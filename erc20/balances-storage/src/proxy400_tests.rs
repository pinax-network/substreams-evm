use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/proxy400/layouts.json")).unwrap()
}

fn controls() -> Vec<Value> {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/proxy400/controls.json")).unwrap();
    fixture["tokens"].as_array().unwrap().clone()
}

#[test]
fn dood_and_deep_captured_blocks_match_independent_rpc() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/proxy400");
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/proxy400/cases.json")).unwrap();
    let layouts = layouts();
    assert_eq!(cases.len(), 2);
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let block = eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap();
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        let events = project(&block, std::slice::from_ref(layout)).unwrap();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&contract)));
        checked += expected.len();
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        assert!(tokens.insert(contract));
    }
    assert_eq!(tokens.len(), 2);
    assert_eq!(checked, 4);
}

// Translate independently captured RPC overrides into Extended storage writes.
// Include actual mapping preimages; metadata never seeds or repairs a balance.
fn control_block(token: &Value, control: &Value, layout: &VerifiedLayout) -> eth::Block {
    let holder = hex_bytes(token["address"].as_str().unwrap()).unwrap();
    let mut preimages = std::collections::HashMap::new();
    let mut roots = vec![layout.balance_slot];
    for ns in token["namespaces"].as_array().unwrap() {
        if ns["namespace"] == "deepnode.storage.BanlistControl" {
            roots.push(word(&hex_bytes(ns["slot"].as_str().unwrap()).unwrap()).unwrap());
        }
    }
    for root in roots {
        let mut preimage = [0; 64];
        preimage[12..32].copy_from_slice(&holder);
        preimage[32..].copy_from_slice(&root);
        preimages.insert(hex::encode(hash(&preimage)), hex::encode(preimage));
    }
    let changes = control["state_diff"]
        .as_object()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, (key, value))| {
            let new_value = hex_bytes(value.as_str().unwrap()).unwrap();
            let old_value = if new_value.iter().all(|b| *b == 0) { vec![1] } else { vec![] };
            eth::StorageChange {
                address: layout.contract.clone(),
                key: hex_bytes(key).unwrap(),
                old_value,
                new_value,
                ordinal: i as u64 + 1,
            }
        })
        .collect();
    let number = token["block"].as_u64().unwrap();
    eth::Block {
        ver: 5,
        number,
        hash: hex_bytes(token["hash"].as_str().unwrap()).unwrap(),
        detail_level: eth::block::DetailLevel::DetaillevelExtended as i32,
        header: Some(eth::BlockHeader {
            number,
            parent_hash: vec![2; 32],
            state_root: vec![3; 32],
            ..Default::default()
        }),
        transaction_traces: vec![eth::TransactionTrace {
            status: eth::TransactionTraceStatus::Succeeded as i32,
            calls: vec![eth::Call {
                address: layout.contract.clone(),
                storage_changes: changes,
                keccak_preimages: preimages,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn reviewed_owner_minter_pause_and_ban_writes_match_rpc_controls() {
    let layouts = layouts();
    let mut checked = 0;
    for token in controls() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        for control in token["controls"].as_array().unwrap() {
            if control["expected_revert"] == true || control["case"] == "zero-admin-storage-mirror" {
                continue;
            }
            let expected = amount(&hex_bytes(control["response"]["result"].as_str().unwrap()).unwrap()).unwrap();
            let events = project(&control_block(&token, control, layout), std::slice::from_ref(layout)).unwrap();
            assert_eq!(events.balances.len(), 1, "{}", control["case"]);
            assert_eq!(events.balances[0].contract.as_ref(), Some(&contract));
            assert_eq!(events.balances[0].address, hex_bytes(token["address"].as_str().unwrap()).unwrap());
            assert_eq!(events.balances[0].amount, expected, "{}", control["case"]);
            checked += 1;
        }
    }
    assert_eq!(checked, 8);
}

#[test]
fn unreviewed_deep_admin_mirror_write_still_stops_processing() {
    let layouts = layouts();
    let token = controls()
        .into_iter()
        .find(|t| t["contract"] == "0x9b6a1d4fa5d90e5f2d34130053978d14cd301d58")
        .unwrap();
    let layout = layouts
        .iter()
        .find(|l| l.contract == hex_bytes(token["contract"].as_str().unwrap()).unwrap())
        .unwrap();
    let control = token["controls"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["case"] == "zero-admin-storage-mirror")
        .unwrap();
    // Although the immutable-admin proxy's getter is unaffected by this slot,
    // the qualified configuration deliberately leaves its administrative writes unresolved.
    assert!(project(&control_block(&token, control, layout), std::slice::from_ref(layout))
        .unwrap_err()
        .to_string()
        .contains("unresolved storage"));
}
