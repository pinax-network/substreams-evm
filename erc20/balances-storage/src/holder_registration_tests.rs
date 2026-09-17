use super::*;
use prost::Message;
use serde_json::Value;

fn fixture() -> (eth::Block, Vec<VerifiedLayout>, Value) {
    let block = eth::Block::decode(include_bytes!("../tests/fixtures/holder-registration/creation.pb").as_slice()).unwrap();
    let layouts = layout::parse(include_str!("../tests/fixtures/4stock-registration-layout.json")).unwrap();
    let expected = serde_json::from_str(include_str!("../tests/fixtures/holder-registration/creation.json")).unwrap();
    (block, layouts, expected)
}

#[test]
fn complete_captured_deployment_preserves_initial_holders_and_supply() {
    let (block, layouts, expected) = fixture();
    assert_eq!(block.number, expected["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(expected["hash"].as_str().unwrap()).unwrap());
    let events = project(&block, &layouts).unwrap();
    let actual = events
        .balances
        .into_iter()
        .map(|r| (format!("0x{}", hex::encode(r.address)), r.amount))
        .collect::<BTreeMap<_, _>>();
    let expected_rows = expected["balances"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["address"].as_str().unwrap().to_string(), r["rpc"].as_str().unwrap().to_string()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected_rows);
    assert_eq!(
        actual.values().fold(BigInt::from(0), |sum, s| sum + s.parse::<BigInt>().unwrap()).to_string(),
        expected["total_supply_rpc"]
    );
    let array_base = hex::encode(hash(&word(&[35]).unwrap()));
    assert!(block
        .transaction_traces
        .iter()
        .flat_map(|t| &t.calls)
        .all(|c| !c.keccak_preimages.contains_key(&array_base)));
}

#[test]
fn captured_registration_needs_qualified_deployment_initialization_and_membership() {
    for variant in 0..6 {
        let (block, mut layouts, _) = fixture();
        let l = &mut layouts[0];
        match variant {
            0 => l.deployment = None,
            1 => l.address_lists.clear(),
            2 => {
                l.other_mapping_words.insert(word(&[36]).unwrap(), 4);
            }
            3 => {
                l.other_mapping_slots.remove(&word(&[17]).unwrap());
            }
            4 => {
                l.other_slots.remove(&word(&[0]).unwrap());
            }
            5 => {
                l.other_slots.remove(&hash(&word(&[8]).unwrap()));
            }
            _ => unreachable!(),
        }
        assert!(project(&block, &layouts).is_err(), "variant {variant}");
    }
}

#[test]
fn captured_first_member_cannot_bypass_length_or_element_validation() {
    for variant in 0..4 {
        let (mut block, layouts, _) = fixture();
        let rows = &mut block
            .transaction_traces
            .iter_mut()
            .flat_map(|t| &mut t.calls)
            .find(|c| c.index == 193 && c.storage_changes.iter().any(|s| s.address == layouts[0].contract))
            .unwrap()
            .storage_changes;
        let root = word(&[35]).unwrap();
        let base = hash(&root);
        match variant {
            0 => rows.retain(|s| s.key != root),
            1 => rows.retain(|s| s.key != base),
            2 => rows.iter_mut().find(|s| s.key == base).unwrap().ordinal = 5672,
            3 => rows.iter_mut().find(|s| s.key == base).unwrap().new_value = vec![1; 21],
            _ => unreachable!(),
        }
        assert!(project(&block, &layouts).is_err(), "variant {variant}");
    }
}

#[test]
fn captured_launch_trading_matches_rpc_and_requires_each_reward_field() {
    let block = eth::Block::decode(include_bytes!("../tests/fixtures/holder-registration/first-trading.pb").as_slice()).unwrap();
    let layouts = layout::parse(include_str!("../tests/fixtures/4stock-registration-layout.json")).unwrap();
    let expected: Value = serde_json::from_str(include_str!("../tests/fixtures/holder-registration/first-trading.json")).unwrap();
    assert_eq!(block.number, expected["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(expected["hash"].as_str().unwrap()).unwrap());
    let events = project(&block, &layouts).unwrap();
    assert_eq!(events.balances.len(), 5);
    let actual = events
        .balances
        .into_iter()
        .map(|r| (format!("0x{}", hex::encode(r.address)), r.amount))
        .collect::<BTreeMap<_, _>>();
    let expected_rows = expected["balances"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["address"].as_str().unwrap().to_string(), r["rpc"].as_str().unwrap().to_string()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(actual, expected_rows);
    for slot in [38, 41, 44] {
        let mut missing = layouts.clone();
        assert!(missing[0].other_slots.remove(&word(&[slot]).unwrap()));
        assert!(project(&block, &missing).unwrap_err().to_string().contains("unresolved storage"));
    }
}
