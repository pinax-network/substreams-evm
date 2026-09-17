use super::*;
use serde_json::{json, Value};

fn scalar(n: u64) -> [u8; 32] {
    word(&n.to_be_bytes()).unwrap()
}
fn plus(mut key: [u8; 32], n: u64) -> [u8; 32] {
    let mut carry = n as u128;
    for b in key.iter_mut().rev() {
        carry += u128::from(*b);
        *b = carry as u8;
        carry >>= 8;
    }
    key
}
fn packed(clock: u64, votes: u64) -> Vec<u8> {
    let mut value = scalar(clock);
    value[18..26].copy_from_slice(&votes.to_be_bytes());
    value.to_vec()
}
fn params(clock: &str, mapped: bool) -> Value {
    json!([{"contract":format!("0x{}","aa".repeat(20)),"balance_slot":format!("0x{:064x}",5),"code_hash":format!("0x{}","bb".repeat(32)),
        "voting_checkpoints":{"clock":clock,"slots":if mapped{vec![]}else{vec![format!("0x{:064x}",15)]},"mapping_slots":if mapped{vec![format!("0x{:064x}",14)]}else{vec![]}}}])
}
fn case(clock: &str, mapped: bool) -> (eth::Block, Vec<VerifiedLayout>) {
    let layouts = layout::parse(&params(clock, mapped).to_string()).unwrap();
    let l = &layouts[0];
    let mut preimages = std::collections::HashMap::new();
    let root = if mapped {
        let mut preimage = [0; 64];
        preimage[12..32].copy_from_slice(&[8; 20]);
        preimage[32..].copy_from_slice(&scalar(14));
        preimages.insert(hex::encode(hash(&preimage)), hex::encode(preimage));
        hash(&preimage)
    } else {
        scalar(15)
    };
    let owner = [6; 20];
    let mut p = [0; 64];
    p[12..32].copy_from_slice(&owner);
    p[32..].copy_from_slice(&l.balance_slot);
    preimages.insert(hex::encode(hash(&p)), hex::encode(p));
    let row = |key: [u8; 32], old: Vec<u8>, new: Vec<u8>, ordinal| eth::StorageChange {
        address: l.contract.clone(),
        key: key.to_vec(),
        old_value: old,
        new_value: new,
        ordinal,
    };
    let call = eth::Call {
        address: l.contract.clone(),
        keccak_preimages: preimages,
        storage_changes: vec![
            row(root, vec![2], vec![3], 10),
            row(plus(hash(&root), 2), vec![], packed(100, 7), 20),
            row(hash(&p), vec![8], vec![9], 30),
        ],
        ..Default::default()
    };
    (
        eth::Block {
            ver: 5,
            number: 100,
            hash: vec![1; 32],
            detail_level: eth::block::DetailLevel::DetaillevelExtended as i32,
            header: Some(eth::BlockHeader {
                number: 100,
                parent_hash: vec![2; 32],
                state_root: vec![3; 32],
                timestamp: Some(prost_types::Timestamp { seconds: 100, nanos: 0 }),
                ..Default::default()
            }),
            transaction_traces: vec![eth::TransactionTrace {
                status: 1,
                calls: vec![call],
                ..Default::default()
            }],
            ..Default::default()
        },
        layouts,
    )
}

#[test]
fn direct_and_delegated_vote_appends_and_same_clock_updates_keep_balances() {
    for clock in ["block_number", "timestamp"] {
        for mapped in [false, true] {
            let (mut b, l) = case(clock, mapped);
            let rows = &mut b.transaction_traces[0].calls[0].storage_changes;
            let mut update = rows[1].clone();
            update.old_value = update.new_value.clone();
            update.new_value = packed(100, 8);
            update.ordinal = 40;
            rows.push(update);
            let events = project(&b, &l).unwrap();
            assert_eq!(events.balances.len(), 1);
            assert_eq!(events.balances[0].address, vec![6; 20]);
            assert_eq!(events.balances[0].amount, "9");
        }
    }
}

#[test]
fn timestamp_history_can_update_without_a_length_write_across_blocks_in_one_second() {
    let (mut b, l) = case("timestamp", true);
    let rows = &mut b.transaction_traces[0].calls[0].storage_changes;
    rows.remove(0);
    rows[0].old_value = packed(100, 3);
    b.number = 101;
    b.header.as_mut().unwrap().number = 101;
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "9");
    b.header.as_mut().unwrap().timestamp.as_mut().unwrap().seconds = 101;
    assert!(project(&b, &l).unwrap_err().to_string().contains("another clock"));
}

#[test]
fn malformed_vote_lengths_elements_and_execution_order_are_rejected() {
    for case_id in 0..14 {
        let (mut b, l) = case("block_number", false);
        let rows = &mut b.transaction_traces[0].calls[0].storage_changes;
        match case_id {
            0 => rows[0].new_value = vec![1],
            1 => rows[0].new_value = vec![4],
            2 => rows[0].new_value = vec![102],
            3 => rows[1].key = plus(hash(&scalar(15)), 3).to_vec(),
            4 => rows[1].new_value = packed(99, 7),
            5 => rows[1].old_value = packed(99, 7),
            6 => rows[1].ordinal = 9,
            7 => {
                rows.remove(1);
            }
            8 => {
                rows.remove(0);
            }
            9 => {
                let mut bad = rows[1].clone();
                bad.ordinal = 40;
                bad.old_value = packed(100, 999);
                rows.push(bad);
            }
            10 => {
                let mut bad = rows[0].clone();
                bad.ordinal = 40;
                bad.old_value = vec![3];
                bad.new_value = vec![4];
                rows.push(bad);
            }
            11 => rows[1].key = plus(hash(&scalar(15)), 101).to_vec(),
            12 => rows[0].ordinal = 0,
            13 => rows[1].ordinal = 0,
            _ => unreachable!(),
        }
        assert!(project(&b, &l).is_err(), "case {case_id}");
    }
}

#[test]
fn checkpoint_membership_requires_the_configured_contract_and_mapping_preimage() {
    for variant in 0..3 {
        let (mut b, l) = case("block_number", true);
        let c = &mut b.transaction_traces[0].calls[0];
        match variant {
            0 => c.keccak_preimages.retain(|_, p| !p.ends_with(&hex::encode(scalar(14)))),
            1 => c.storage_changes[0].key = scalar(15).to_vec(),
            2 => c.storage_changes[0].address = vec![0xcc; 20],
            _ => unreachable!(),
        }
        assert!(project(&b, &l).is_err());
    }
}

#[test]
fn reverted_checkpoint_writes_never_authorize_persisted_array_elements() {
    let (mut b, l) = case("block_number", true);
    let mut reverted = b.transaction_traces[0].calls[0].clone();
    reverted.state_reverted = true;
    reverted.storage_changes.truncate(1);
    b.transaction_traces[0].calls[0].storage_changes.remove(0);
    b.transaction_traces[0].calls.push(reverted);
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}

#[test]
fn timestamp_overwrites_reject_two_tails_and_unbounded_indices() {
    for invalid in [false, true] {
        let (mut b, l) = case("timestamp", false);
        let rows = &mut b.transaction_traces[0].calls[0].storage_changes;
        rows.remove(0);
        rows[0].old_value = packed(100, 1);
        let mut second = rows[0].clone();
        second.ordinal = 40;
        second.key = plus(hash(&scalar(15)), if invalid { 101 } else { 3 }).to_vec();
        rows.push(second);
        assert!(project(&b, &l).is_err());
    }
}

#[test]
fn checkpoint_configuration_rejects_overlapping_fields_unknown_clocks_and_empty_roots() {
    for variant in 0..8 {
        let mut p = params("block_number", false);
        match variant {
            0 => p[0]["voting_checkpoints"]["clock"] = json!("automatic"),
            1 => p[0]["voting_checkpoints"]["slots"] = json!([]),
            2 => p[0]["voting_checkpoints"]["slots"] = json!([p[0]["balance_slot"].clone()]),
            3 => p[0]["voting_checkpoints"]["mapping_slots"] = p[0]["voting_checkpoints"]["slots"].clone(),
            4 => p[0]["other_slots"] = p[0]["voting_checkpoints"]["slots"].clone(),
            5 => p[0]["other_mapping_slots"] = p[0]["voting_checkpoints"]["slots"].clone(),
            6 => p[0]["other_mapping_words"] = json!({format!("0x{:064x}",15):2}),
            7 => p[0]["zero_balance"] = json!({"value":format!("0x{:064x}",100),"storage_slot":format!("0x{:064x}",15)}),
            _ => unreachable!(),
        }
        assert!(layout::parse(&p.to_string()).is_err(), "case {variant}");
    }
}
