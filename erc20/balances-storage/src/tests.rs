use super::*;
use prost::Message;

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
            timestamp: Some(prost_types::Timestamp { seconds: 1789570000, nanos: 0 }),
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
fn native(ordinal: u64, old: u8, new: u8) -> eth::BalanceChange {
    eth::BalanceChange {
        address: vec![4; 20],
        old_value: Some(eth::BigInt { bytes: vec![old] }),
        new_value: Some(eth::BigInt { bytes: vec![new] }),
        ordinal,
        reason: eth::balance_change::Reason::Transfer as i32,
    }
}
fn wbnb_call(owner: &[u8], old: u8, new: u8) -> eth::Call {
    let mut preimage = vec![0; 64];
    preimage[12..32].copy_from_slice(owner);
    preimage[63] = 3;
    let key = hash(&preimage);
    eth::Call {
        address: hex::decode(WBNB).unwrap(),
        keccak_preimages: [(hex::encode(key), hex::encode(preimage))].into(),
        storage_changes: vec![eth::StorageChange {
            address: hex::decode(WBNB).unwrap(),
            key: key.to_vec(),
            old_value: vec![old],
            new_value: vec![new],
            ordinal: 10,
        }],
        ..Default::default()
    }
}
#[test]
fn takes_last_persisted_value_by_ordinal_and_preserves_first_old() {
    let mut b = block();
    let mut c = wbnb_call(&[6; 20], 5, 9);
    let mut last = c.storage_changes[0].clone();
    last.ordinal = 30;
    last.old_value = vec![9];
    last.new_value = vec![0];
    c.storage_changes.insert(0, last);
    b.transaction_traces = vec![tx(c)];
    let out = project(&b).unwrap();
    assert_eq!(out.balances.len(), 1);
    assert_eq!((&*out.balances[0].old_amount, &*out.balances[0].amount), ("5", "0"));
    assert_eq!(out.balances[0].ordinal, 30);
}
#[test]
fn reverted_child_does_not_leak_state() {
    let mut b = block();
    let mut c = wbnb_call(&[6; 20], 5, 8);
    c.state_reverted = true;
    c.balance_changes = vec![native(0, 3, 0)];
    b.transaction_traces = vec![tx(c)];
    let out = project(&b).unwrap();
    assert!(out.balances.is_empty());
    assert_eq!(out.wbnb_storage_changes, 0);
}
#[test]
fn discovers_holder_from_preimage_without_transfer_event() {
    let mut b = block();
    b.transaction_traces = vec![tx(wbnb_call(&[6; 20], 8, 0))];
    let out = project(&b).unwrap();
    assert_eq!(out.balances[0].address, vec![6; 20]);
    assert_eq!(out.balances[0].amount, "0");
    assert!(out.unresolved_wbnb_slots.is_empty());
}
#[test]
fn candidate_address_is_only_accepted_when_slot_hash_matches() {
    let mut b = block();
    let mut c = wbnb_call(&[6; 20], 8, 2);
    c.keccak_preimages.clear();
    c.caller = vec![6; 20];
    b.transaction_traces = vec![tx(c)];
    assert_eq!(project(&b).unwrap().balances.len(), 1);
    b.transaction_traces[0].calls[0].caller = vec![7; 20];
    let out = project(&b).unwrap();
    assert!(out.balances.is_empty());
    assert_eq!(out.unresolved_wbnb_slots.len(), 1);
    assert!(events(&out).is_err());
}
#[test]
fn rejects_corrupt_preimage() {
    let mut b = block();
    let mut c = wbnb_call(&[6; 20], 1, 2);
    c.keccak_preimages.values_mut().for_each(|x| *x = "00".into());
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b).is_err());
}
#[test]
fn allowance_is_not_a_token_balance() {
    let mut b = block();
    let mut p1 = vec![0; 64];
    p1[12..32].copy_from_slice(&[6; 20]);
    p1[63] = 4;
    let h1 = hash(&p1);
    let mut p2 = vec![0; 64];
    p2[12..32].copy_from_slice(&[7; 20]);
    p2[32..].copy_from_slice(&h1);
    let h2 = hash(&p2);
    let c = eth::Call {
        address: hex::decode(WBNB).unwrap(),
        keccak_preimages: [(hex::encode(h1), hex::encode(p1)), (hex::encode(h2), hex::encode(p2))].into(),
        storage_changes: vec![eth::StorageChange {
            address: hex::decode(WBNB).unwrap(),
            key: h2.to_vec(),
            old_value: vec![],
            new_value: vec![9],
            ordinal: 10,
        }],
        ..Default::default()
    };
    b.transaction_traces = vec![tx(c)];
    let out = project(&b).unwrap();
    assert!(out.balances.is_empty());
    assert!(out.unresolved_wbnb_slots.is_empty());
}
#[test]
fn preserves_uint256_max() {
    let mut b = block();
    let mut c = wbnb_call(&[6; 20], 0, 1);
    c.storage_changes[0].new_value = vec![255; 32];
    b.transaction_traces = vec![tx(c)];
    assert_eq!(
        project(&b).unwrap().balances[0].amount,
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    );
}
#[test]
fn incomplete_or_ambiguous_input_fails() {
    let mut b = block();
    b.detail_level = 1;
    assert!(project(&b).is_err());
    b = block();
    b.ver = 99;
    assert!(project(&b).is_err());
    b = block();
    let mut c = wbnb_call(&[6; 20], 0, 1);
    c.storage_changes.push(c.storage_changes[0].clone());
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b).is_err());
    b.transaction_traces[0].calls[0].storage_changes[1].ordinal = 20;
    b.transaction_traces[0].calls[0].storage_changes[1].old_value = vec![3];
    assert!(project(&b).is_err());
}
#[test]
fn empty_block_has_the_same_empty_events_encoding_as_reference() {
    let out = project(&block()).unwrap();
    let encoded = events(&out).unwrap().encode_to_vec();
    assert!(encoded.is_empty());
    assert_eq!(balances_pb::Events::decode(encoded.as_slice()).unwrap(), balances_pb::Events::default());
}
#[test]
fn fails_on_wbnb_code_change() {
    let mut b = block();
    b.code_changes = vec![eth::CodeChange {
        address: hex::decode(WBNB).unwrap(),
        new_hash: vec![4; 32],
        new_code: vec![0],
        ordinal: 5,
        ..Default::default()
    }];
    assert!(project(&b).is_err());
    b.code_changes[0].new_hash = hex_bytes(WBNB_CODE_HASH).unwrap();
    b.code_changes[0].new_code = hex_bytes(include_str!("../tests/fixtures/wbnb-runtime.hex").trim()).unwrap();
    // Returning to the expected code cannot qualify earlier writes in this block.
    assert!(project(&b).is_err());
}
#[test]
fn pinned_runtime_matches_previously_proven_wbnb_code_hash() {
    let code = hex_bytes(include_str!("../tests/fixtures/wbnb-runtime.hex").trim()).unwrap();
    assert_eq!(hex::encode(hash(&code)), WBNB_CODE_HASH);
}
#[test]
fn captured_failed_authorization_transactions_project_without_rpc() {
    for bytes in [
        include_bytes!("../tests/fixtures/bsc-121114122-failed-setcode.pb").as_slice(),
        include_bytes!("../tests/fixtures/bsc-121114153-failed-setcode.pb").as_slice(),
    ] {
        let mut b = block();
        b.transaction_traces = vec![eth::TransactionTrace::decode(bytes).unwrap()];
        let out = project(&b).unwrap();
        assert!(out.balances.iter().all(|b| hex::encode(&b.contract) == WBNB));
        assert!(events(&out).is_ok());
    }
}

#[test]
fn captured_complete_bsc_block_matches_18_historical_erc20_rpc_balances() {
    let b = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122260950.pb").as_slice()).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-122260950.json")).unwrap();
    let out = project(&b).unwrap();
    assert_eq!(format!("0x{}", hex::encode(&out.hash)), expected["block_hash"]);
    assert_eq!(out.number, 122260950);
    assert!(out.unresolved_wbnb_slots.is_empty());
    let oracle = expected["rpc_checks"].as_array().unwrap();
    assert_eq!(oracle.len(), 100);
    assert_eq!(out.balances.len(), 18);
    for row in oracle.iter().filter(|row| row["contract"] != "") {
        let address = hex_bytes(row["address"].as_str().unwrap()).unwrap();
        let contract = hex_bytes(row["contract"].as_str().unwrap()).unwrap();
        let actual = out.balances.iter().find(|b| b.address == address && b.contract == contract).unwrap();
        assert_eq!(actual.amount, row["balance"].as_str().unwrap(), "address {}", row["address"]);
    }
    let public = events(&out).unwrap();
    assert_eq!(public.balances.len(), 18);
    assert!(public.balances.iter().all(|b| b.contract.is_some()));
}

#[test]
fn event_wire_format_is_exactly_the_shared_reference_protobuf() {
    let mut b = block();
    b.balance_changes = vec![native(1, 0, 99)];
    b.transaction_traces = vec![tx(wbnb_call(&[6; 20], 8, 0))];
    let storage = project(&b).unwrap();
    let expected = balances_pb::Events {
        balances: vec![balances_pb::Balance {
            contract: Some(hex_bytes(WBNB).unwrap()),
            address: vec![6; 20],
            amount: "0".into(),
        }],
    };
    assert_eq!(events(&storage).unwrap().encode_to_vec(), expected.encode_to_vec());
    assert_eq!(
        balance_changes(&storage).unwrap(),
        balances_pb::BalanceChanges {
            balance_changes: vec![balances_pb::BalanceChange {
                contract: expected.balances[0].contract.clone(),
                address: vec![6; 20],
            }]
        }
    );
}
