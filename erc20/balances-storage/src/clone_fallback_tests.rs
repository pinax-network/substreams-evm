use super::*;
use prost::Message;
use serde_json::Value;

fn layout() -> layout::VerifiedLayout {
    let mut layouts = layout::parse(include_str!("../tests/fixtures/clone-fallback/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 1);
    layouts.remove(0)
}
fn captured() -> (eth::Block, Value) {
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/clone-fallback/cases.json")).unwrap();
    assert_eq!(cases.len(), 1);
    let case = cases.into_iter().next().unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/clone-fallback")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    (block, case)
}
#[test]
fn captured_clone_events_match_rpc_with_its_pinned_zero_word_fallback() {
    let l = layout();
    let (block, case) = captured();
    let actual = project(&block, std::slice::from_ref(&l)).unwrap();
    let expected = case["balances"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(expected.len(), 2);
    assert!(actual.balances.iter().all(|r| r.contract.as_ref() == Some(&l.contract)));
    assert_eq!(actual.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
}
#[test]
fn captured_clone_holders_need_fallback_projection_at_initialization() {
    let l = layout();
    let mut raw_only = l.clone();
    raw_only.zero_balance = None;
    let mut fallback_count = 0;
    let mut holders = BTreeSet::new();
    for line in include_str!("../tests/fixtures/clone-fallback/checkpoint.jsonl").lines() {
        let row: Value = serde_json::from_str(line).unwrap();
        assert_eq!(row["contract"], format!("0x{}", hex::encode(&l.contract)));
        assert_eq!(row["hash"], "0xdf9d02ee1c9e345bfcdb971e488a20f7f51dc03d3d8e2e7bbc8fc9f0bb0814cf");
        let holder = hex_bytes(row["address"].as_str().unwrap()).unwrap();
        assert!(holders.insert(holder.clone()));
        let raw = row["storage"].as_str().unwrap();
        assert_eq!(l.project_amount(&holder, raw), row["rpc"]);
        assert_eq!(row["projected"], row["rpc"]);
        if raw == "0" {
            assert_eq!(row["rpc"], "1000000000");
            assert_ne!(raw_only.project_amount(&holder, raw), row["rpc"]);
            fallback_count += 1;
        }
    }
    assert_eq!(holders.len(), 84);
    assert_eq!(fallback_count, 81);
}
#[test]
fn clone_fallback_dependency_is_protected_without_any_balance_write() {
    let l = layout();
    let (mut block, _) = captured();
    block.transaction_traces.clear();
    block.system_calls.clear();
    let rule = l.zero_balance.as_ref().unwrap();
    block.system_calls.push(eth::Call {
        address: l.contract.clone(),
        storage_changes: vec![eth::StorageChange {
            address: l.contract.clone(),
            key: rule.storage_slot.unwrap().to_vec(),
            old_value: rule.value.to_vec(),
            new_value: vec![0; 32],
            ordinal: 1,
        }],
        ..Default::default()
    });
    assert!(project(&block, std::slice::from_ref(&l))
        .unwrap_err()
        .to_string()
        .contains("zero-balance dependency"));
    let mut incomplete = l.clone();
    incomplete.zero_balance.as_mut().unwrap().storage_slot = None;
    assert!(project(&block, &[incomplete]).is_err(), "unknown writes must still fail closed");
    block.system_calls[0].state_reverted = true;
    assert!(project(&block, &[l]).unwrap().balances.is_empty());
}
