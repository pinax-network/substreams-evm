use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/top50-final/cases.json")).unwrap()
}

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/bsc-top50-layouts.json")).unwrap()
}

fn captured(case: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/top50-final")
        .join(case["fixture"].as_str().unwrap());
    eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap()
}

#[test]
fn eight_remaining_ranked_tokens_match_captured_historical_rpc_balances() {
    let cases = cases();
    assert_eq!(cases.len(), 8);
    let layouts = layouts();
    let mut contracts = BTreeSet::new();
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        assert!(contracts.insert(contract.clone()));
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        // Runtime-identical siblings were deployed earlier than the previously
        // qualified family member; they must not inherit its zero initialization.
        assert!(layout.deployment.is_none());
        let block = captured(&case);
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let events = project(&block, std::slice::from_ref(layout)).unwrap();
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (hex_bytes(row["address"].as_str().unwrap()).unwrap(), row["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        assert!(events.balances.iter().all(|b| b.contract.as_ref() == Some(&contract)));
        let actual = events.balances.into_iter().map(|b| (b.address, b.amount)).collect::<BTreeMap<_, _>>();
        assert_eq!(actual, expected, "contract {}", case["contract"]);
    }
}

#[test]
fn packed_pause_and_owner_fields_do_not_change_raw_holder_outputs() {
    let layouts = layouts();
    for case in cases().iter().filter(|c| c["rank"] == 19 || c["rank"] == 46) {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let original = captured(case);
        let expected = project(&original, std::slice::from_ref(layout)).unwrap();
        let (slot, owner_byte, pause_byte) = if case["rank"] == 19 { (6, 31, 11) } else { (5, 30, 31) };
        for paused in [false, true] {
            let mut packed = [0; 32];
            packed[owner_byte] = 123;
            packed[pause_byte] = u8::from(paused);
            let mut block = original.clone();
            block.transaction_traces.push(eth::TransactionTrace {
                status: 1,
                calls: vec![eth::Call {
                    storage_changes: vec![eth::StorageChange {
                        address: contract.clone(),
                        key: word(&[slot]).unwrap().to_vec(),
                        old_value: vec![],
                        new_value: packed.to_vec(),
                        ordinal: 1_000_000,
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            });
            assert_eq!(project(&block, std::slice::from_ref(layout)).unwrap(), expected);
        }
    }
}
