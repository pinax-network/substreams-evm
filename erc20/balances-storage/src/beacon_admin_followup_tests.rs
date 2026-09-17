use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/beacon-admin-followup/cases.json")).unwrap()
}
fn load(case: &Value) -> (eth::Block, layout::VerifiedLayout) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/beacon-admin-followup")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    let mut layouts = layout::parse(&serde_json::json!([case["layout"]]).to_string()).unwrap();
    assert_eq!(layouts.len(), 1);
    (block, layouts.remove(0))
}

#[test]
fn five_additional_profiles_match_captured_independent_rpc() {
    let cases = cases();
    assert_eq!(cases.len(), 5);
    let mut tokens = BTreeSet::new();
    for case in cases {
        let (block, layout) = load(&case);
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (hex_bytes(row["address"].as_str().unwrap()).unwrap(), row["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        let events = project(&block, std::slice::from_ref(&layout)).unwrap();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|row| row.contract.as_ref() == Some(&layout.contract)));
        assert_eq!(
            events.balances.into_iter().map(|row| (row.address, row.amount)).collect::<BTreeMap<_, _>>(),
            expected
        );
        tokens.insert(layout.contract);
    }
    assert_eq!(tokens.len(), 5);
}

#[test]
fn captured_ald_and_by_activity_requires_reviewed_transaction_flags() {
    for (rank, slot) in [(224, 7), (239, 10)] {
        let case = cases().into_iter().find(|case| case["rank"] == rank).unwrap();
        let (block, mut layout) = load(&case);
        assert!(!project(&block, std::slice::from_ref(&layout)).unwrap().balances.is_empty());
        assert!(layout.other_slots.remove(&word(&hex_bytes(&format!("0x{slot:064x}")).unwrap()).unwrap()));
        assert!(project(&block, &[layout])
            .unwrap_err()
            .to_string()
            .contains("unresolved storage for configured token"));
    }
}

#[test]
fn bless_beacon_admin_change_cannot_hide_behind_unchanged_implementation_pointers() {
    let case = cases().into_iter().find(|case| case["rank"] == 204).unwrap();
    let (mut block, layout) = load(&case);
    let expected = project(&block, std::slice::from_ref(&layout)).unwrap();
    let beacon = layout.beacon_proxy.as_ref().unwrap();
    let admin = beacon.proxy_admin.as_ref().unwrap();
    block.system_calls.push(eth::Call {
        address: beacon.beacon.clone(),
        storage_changes: vec![eth::StorageChange {
            address: beacon.beacon.clone(),
            key: admin.slot.to_vec(),
            old_value: admin.address.clone(),
            new_value: layout.contract.clone(),
            ordinal: u64::MAX - 1,
        }],
        ..Default::default()
    });
    assert!(project(&block, std::slice::from_ref(&layout))
        .unwrap_err()
        .to_string()
        .contains("beacon proxy admin changed"));
    let mut unpinned = layout;
    unpinned.beacon_proxy.as_mut().unwrap().proxy_admin = None;
    assert_eq!(
        project(&block, &[unpinned]).unwrap(),
        expected,
        "the prior configuration missed this getter dependency"
    );
}
