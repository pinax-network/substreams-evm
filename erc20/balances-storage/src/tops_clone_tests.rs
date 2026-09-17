use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/tops-clone-followup/cases.json")).unwrap()
}
fn load(case: &Value) -> (eth::Block, layout::VerifiedLayout) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tops-clone-followup")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    let mut layouts = layout::parse(&serde_json::json!([case["layout"]]).to_string()).unwrap();
    assert_eq!(layouts.len(), 1);
    (block, layouts.remove(0))
}

#[test]
fn tops_and_standard_clone_match_captured_independent_rpc() {
    let cases = cases();
    assert_eq!(cases.len(), 3);
    assert_eq!(cases.iter().map(|case| case["balances"].as_array().unwrap().len()).sum::<usize>(), 6);
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
    assert_eq!(tokens.len(), 2);
}

#[test]
fn captured_tops_provider_append_requires_the_address_list_rule() {
    let cases = cases();
    let active = cases.iter().filter(|case| case["required_rule"] == "address_list").collect::<Vec<_>>();
    assert_eq!(active.len(), 1);
    let (block, mut layout) = load(active[0]);
    assert_eq!(block.number, 119571736);
    assert_eq!(project(&block, std::slice::from_ref(&layout)).unwrap().balances.len(), 2);
    layout.address_lists.clear();
    let error = project(&block, &[layout]).unwrap_err().to_string();
    assert_eq!(error, active[0]["without_special_rule"].as_str().unwrap());
    assert!(error.contains("unresolved storage for configured token"));
}

#[test]
fn tops_lp_record_array_writes_are_not_silently_ignored() {
    let case = cases().into_iter().find(|case| case["rank"] == 236).unwrap();
    let (mut block, layout) = load(&case);
    block.transaction_traces.clear();
    block.system_calls.clear();
    let mut preimage = vec![0; 12];
    preimage.extend_from_slice(&[0x35; 20]);
    preimage.extend_from_slice(&hex_bytes(&format!("0x{:064x}", 31)).unwrap());
    let key = hash(&preimage);
    block.system_calls.push(eth::Call {
        address: layout.contract.clone(),
        keccak_preimages: [(hex::encode(key), hex::encode(preimage))].into(),
        storage_changes: vec![eth::StorageChange {
            address: layout.contract.clone(),
            key: key.to_vec(),
            old_value: vec![0; 32],
            new_value: vec![1],
            ordinal: 1,
        }],
        ..Default::default()
    });
    assert!(project(&block, std::slice::from_ref(&layout))
        .unwrap_err()
        .to_string()
        .contains("unresolved storage for configured token"));
    block.system_calls[0].state_reverted = true;
    assert!(project(&block, &[layout]).unwrap().balances.is_empty());
}

#[test]
fn standard_clone_dependency_code_remains_protected_without_holder_writes() {
    let case = cases().into_iter().find(|case| case["rank"] == 220).unwrap();
    let (mut block, layout) = load(&case);
    block.transaction_traces.clear();
    block.system_calls.clear();
    let delegate = layout.minimal_proxy.as_ref().unwrap();
    block.code_changes = vec![eth::CodeChange {
        address: delegate.implementation.clone(),
        new_hash: delegate.code_hash.to_vec(),
        ordinal: 1,
        ..Default::default()
    }];
    assert!(project(&block, std::slice::from_ref(&layout)).is_err());
    let mut missing = layout.clone();
    missing.minimal_proxy = None;
    assert!(project(&block, &[missing]).unwrap().balances.is_empty());
}
