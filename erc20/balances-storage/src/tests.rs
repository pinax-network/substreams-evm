use super::*;
use prost::Message;
use serde_json::json;

fn slot(n: u8) -> [u8; 32] {
    let mut value = [0; 32];
    value[31] = n;
    value
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(
        &json!([
            {"contract":format!("0x{}", "aa".repeat(20)),"balance_slot":format!("0x{}",hex::encode(slot(7))),
             "code_hash":format!("0x{}","11".repeat(32)),"other_mapping_slots":[format!("0x{}",hex::encode(slot(8)))]},
            {"contract":format!("0x{}", "bb".repeat(20)),"balance_slot":format!("0x{}","ff".repeat(32)),
             "code_hash":format!("0x{}","22".repeat(32))}
        ])
        .to_string(),
    )
    .unwrap()
}
fn block() -> eth::Block {
    eth::Block {
        ver: 5,
        number: 122260950,
        hash: vec![1; 32],
        detail_level: eth::block::DetailLevel::DetaillevelExtended as i32,
        header: Some(eth::BlockHeader {
            number: 122260950,
            parent_hash: vec![2; 32],
            state_root: vec![3; 32],
            ..Default::default()
        }),
        ..Default::default()
    }
}
fn tx(call: eth::Call) -> eth::TransactionTrace {
    eth::TransactionTrace {
        status: eth::TransactionTraceStatus::Succeeded as i32,
        calls: vec![call],
        ..Default::default()
    }
}
fn token_call(layout: &VerifiedLayout, owner: &[u8], old: u8, new: u8) -> eth::Call {
    let mut preimage = vec![0; 64];
    preimage[12..32].copy_from_slice(owner);
    preimage[32..].copy_from_slice(&layout.balance_slot);
    let key = hash(&preimage);
    eth::Call {
        address: layout.contract.clone(),
        keccak_preimages: [(hex::encode(key), hex::encode(preimage))].into(),
        storage_changes: vec![eth::StorageChange {
            address: layout.contract.clone(),
            key: key.to_vec(),
            old_value: vec![old],
            new_value: vec![new],
            ordinal: 10,
        }],
        ..Default::default()
    }
}
#[test]
fn two_arbitrary_tokens_and_full_width_slots_use_same_shared_events() {
    let l = layouts();
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 5, 0)), tx(token_call(&l[1], &[9; 20], 2, 8))];
    let expected = balances_pb::Events {
        balances: vec![
            balances_pb::Balance {
                contract: Some(l[0].contract.clone()),
                address: vec![6; 20],
                amount: "0".into(),
            },
            balances_pb::Balance {
                contract: Some(l[1].contract.clone()),
                address: vec![9; 20],
                amount: "8".into(),
            },
        ],
    };
    assert_eq!(project(&b, &l).unwrap().encode_to_vec(), expected.encode_to_vec());
    assert_eq!(changes(&b, &l).unwrap()[0].old_amount, "5");
}
#[test]
fn last_persisted_value_wins_and_first_old_value_is_retained() {
    let l = layouts();
    let mut c = token_call(&l[0], &[6; 20], 5, 9);
    let mut last = c.storage_changes[0].clone();
    last.ordinal = 30;
    last.old_value = vec![9];
    last.new_value = vec![0];
    c.storage_changes.insert(0, last);
    let mut b = block();
    b.transaction_traces = vec![tx(c)];
    let rows = changes(&b, &l).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!((&*rows[0].old_amount, &*rows[0].amount, rows[0].ordinal), ("5", "0", 30));
}
#[test]
fn reverted_execution_does_not_emit_balances() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 5, 8);
    c.state_reverted = true;
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn holder_preimage_does_not_require_a_transfer_log() {
    let l = layouts();
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 8, 0))];
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "0");
}
#[test]
fn fallback_address_must_match_the_configured_mapping_hash() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 8, 2);
    c.keccak_preimages.clear();
    c.caller = vec![6; 20];
    b.transaction_traces = vec![tx(c)];
    assert_eq!(project(&b, &l).unwrap().balances.len(), 1);
    b.transaction_traces[0].calls[0].caller = vec![7; 20];
    assert!(project(&b, &l).is_err());
}
#[test]
fn corrupt_preimage_is_rejected() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 1, 2);
    c.keccak_preimages.values_mut().for_each(|v| *v = "00".into());
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).is_err());
}
#[test]
fn only_explicitly_configured_other_storage_is_ignored() {
    let mut l = layouts();
    let mut b = block();
    let mut p1 = vec![0; 64];
    p1[12..32].copy_from_slice(&[6; 20]);
    p1[32..].copy_from_slice(&slot(8));
    let h1 = hash(&p1);
    let mut p2 = vec![0; 64];
    p2[12..32].copy_from_slice(&[7; 20]);
    p2[32..].copy_from_slice(&h1);
    let h2 = hash(&p2);
    b.transaction_traces = vec![tx(eth::Call {
        address: l[0].contract.clone(),
        keccak_preimages: [(hex::encode(h1), hex::encode(p1)), (hex::encode(h2), hex::encode(p2))].into(),
        storage_changes: vec![eth::StorageChange {
            address: l[0].contract.clone(),
            key: h2.to_vec(),
            new_value: vec![9],
            ordinal: 10,
            ..Default::default()
        }],
        ..Default::default()
    })];
    assert!(project(&b, &l).unwrap().balances.is_empty());
    l[0].other_mapping_slots.clear();
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].storage_changes[0].key = slot(2).to_vec();
    assert!(project(&b, &l).is_err());
    l[0].other_slots.insert(slot(2));
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn preserves_uint256_max() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 0, 1);
    c.storage_changes[0].new_value = vec![255; 32];
    b.transaction_traces = vec![tx(c)];
    assert_eq!(
        project(&b, &l).unwrap().balances[0].amount,
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    );
}
#[test]
fn incomplete_or_ambiguous_input_fails() {
    let l = layouts();
    let mut b = block();
    b.detail_level = 1;
    assert!(project(&b, &l).is_err());
    b = block();
    b.ver = 99;
    assert!(project(&b, &l).is_err());
    b = block();
    let mut c = token_call(&l[0], &[6; 20], 0, 1);
    c.storage_changes.push(c.storage_changes[0].clone());
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].storage_changes[1].ordinal = 20;
    b.transaction_traces[0].calls[0].storage_changes[1].old_value = vec![3];
    assert!(project(&b, &l).is_err());
}
#[test]
fn no_configured_tokens_or_no_changes_emits_empty_shared_events() {
    let l = layouts();
    let mut b = block();
    assert!(project(&b, &l).unwrap().encode_to_vec().is_empty());
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 1, 2))];
    assert!(project(&b, &[]).unwrap().balances.is_empty());
    assert!(project(&b, &l[1..]).unwrap().balances.is_empty());
}
#[test]
fn configured_code_changes_fail_even_when_returning_to_pinned_code() {
    let l = layouts();
    let mut b = block();
    b.code_changes = vec![eth::CodeChange {
        address: l[0].contract.clone(),
        new_hash: l[0].code_hash.to_vec(),
        ordinal: 5,
        ..Default::default()
    }];
    assert!(project(&b, &l).is_err());
    b.code_changes[0].address = vec![0x77; 20];
    assert!(project(&b, &l).is_ok());
}

fn proxy_layouts() -> Vec<VerifiedLayout> {
    let mut l = layouts();
    l[0].proxy = Some(layout::VerifiedProxy {
        implementation_slot: slot(99),
        implementation: vec![0xcc; 20],
        code_hash: [0xdd; 32],
    });
    l
}
#[test]
fn pinned_proxy_emits_balances_for_proxy_address() {
    let l = proxy_layouts();
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 1, 2))];
    let events = project(&b, &l).unwrap();
    assert_eq!(events.balances[0].contract, Some(l[0].contract.clone()));
    assert_eq!(events.balances[0].amount, "2");
}
#[test]
fn proxy_upgrade_and_upgrade_back_require_requalification() {
    let l = proxy_layouts();
    let mut call = token_call(&l[0], &[6; 20], 1, 2);
    for (ordinal, old, new) in [(20, 0xcc, 0xee), (30, 0xee, 0xcc)] {
        call.storage_changes.push(eth::StorageChange {
            address: l[0].contract.clone(),
            key: slot(99).to_vec(),
            old_value: vec![old; 20],
            new_value: vec![new; 20],
            ordinal,
        });
    }
    let mut b = block();
    b.transaction_traces = vec![tx(call)];
    assert!(project(&b, &l).unwrap_err().to_string().contains("proxy implementation slot changed"));
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn implementation_code_changes_are_rejected_without_proxy_code_change() {
    let l = proxy_layouts();
    let mut b = block();
    b.code_changes = vec![eth::CodeChange {
        address: l[0].proxy.as_ref().unwrap().implementation.clone(),
        new_hash: vec![0xdd; 32],
        ordinal: 5,
        ..Default::default()
    }];
    assert!(project(&b, &l).unwrap_err().to_string().contains("implementation code changed"));
}
#[test]
fn proxy_layout_cannot_ignore_upgrade_slot_or_pin_zero_self_implementation() {
    let value: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-reviewed-layouts.json")).unwrap();
    for field in ["other_slots", "other_mapping_slots"] {
        let mut bad = value.clone();
        bad[2][field] = json!([bad[2]["proxy"]["implementation_slot"]]);
        assert!(layout::parse(&bad.to_string()).is_err());
    }
    for implementation in [format!("0x{}", "00".repeat(20)), value[2]["contract"].as_str().unwrap().into()] {
        let mut bad = value.clone();
        bad[2]["proxy"]["implementation"] = json!(implementation);
        assert!(layout::parse(&bad.to_string()).is_err());
    }
}
#[test]
fn layout_parameters_reject_ambiguity_and_malformed_values() {
    assert!(layout::parse("[]").unwrap().is_empty());
    let good = include_str!("../tests/fixtures/verified-layouts.json");
    let mut v: serde_json::Value = serde_json::from_str(good).unwrap();
    let duplicate = v[0].clone();
    v.as_array_mut().unwrap().push(duplicate);
    assert!(layout::parse(&v.to_string()).is_err());
    for field in ["contract", "balance_slot", "code_hash"] {
        let mut v: serde_json::Value = serde_json::from_str(good).unwrap();
        v[0][field] = json!("0x01");
        assert!(layout::parse(&v.to_string()).is_err());
    }
    let mut v: serde_json::Value = serde_json::from_str(good).unwrap();
    v[0]["other_mapping_slots"] = json!([v[0]["balance_slot"]]);
    assert!(layout::parse(&v.to_string()).is_err());
}

#[test]
fn package_has_exactly_one_map_and_only_the_shared_protobuf() {
    let manifest = include_str!("../substreams.yaml");
    let modules = manifest.lines().filter(|line| line.starts_with("  - name:")).collect::<Vec<_>>();
    assert_eq!(modules, vec!["  - name: map_events"]);
    assert!(manifest.contains("files: [balances.proto]"));
    assert!(manifest.contains("type: proto:evm.balances.v1.Events"));
}
#[test]
fn captured_failed_authorizations_are_processed_without_rpc() {
    let l = layouts();
    for bytes in [
        include_bytes!("../tests/fixtures/bsc-121114122-failed-setcode.pb").as_slice(),
        include_bytes!("../tests/fixtures/bsc-121114153-failed-setcode.pb").as_slice(),
    ] {
        let mut b = block();
        b.transaction_traces = vec![eth::TransactionTrace::decode(bytes).unwrap()];
        assert!(project(&b, &l).is_ok());
    }
}
#[test]
fn captured_block_matches_all_18_recorded_erc20_balances_with_external_layout() {
    let b = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122260950.pb").as_slice()).unwrap();
    let l = layout::parse(include_str!("../tests/fixtures/verified-layouts.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-122260950.json")).unwrap();
    let out = project(&b, &l).unwrap();
    assert_eq!(out.balances.len(), 18);
    for row in expected["rpc_checks"].as_array().unwrap().iter().filter(|r| r["contract"] != "") {
        let address = hex_bytes(row["address"].as_str().unwrap()).unwrap();
        let contract = hex_bytes(row["contract"].as_str().unwrap()).unwrap();
        let actual = out
            .balances
            .iter()
            .find(|v| v.address == address && v.contract.as_ref() == Some(&contract))
            .unwrap();
        assert_eq!(actual.amount, row["balance"].as_str().unwrap());
    }
    assert_eq!(
        hash(&hex_bytes(include_str!("../tests/fixtures/wbnb-runtime.hex").trim()).unwrap()),
        l[0].code_hash
    );
}
