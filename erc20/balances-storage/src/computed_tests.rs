use super::*;
use prost::Message;
use serde_json::{json, Value};

fn params() -> Value {
    serde_json::from_str(include_str!("../tests/fixtures/bsc-computed-layout.json")).unwrap()
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(&params().to_string()).unwrap()
}
fn block() -> eth::Block {
    eth::Block {
        ver: 5,
        number: 80,
        hash: vec![1; 32],
        detail_level: eth::block::DetailLevel::DetaillevelExtended as i32,
        header: Some(eth::BlockHeader {
            number: 80,
            parent_hash: vec![2; 32],
            state_root: vec![3; 32],
            ..Default::default()
        }),
        ..Default::default()
    }
}
fn tx(call: eth::Call) -> eth::TransactionTrace {
    eth::TransactionTrace {
        status: 1,
        from: vec![8; 20],
        calls: vec![call],
        ..Default::default()
    }
}
fn log(token: &[u8], signature: &str, from: &[u8], to: &[u8]) -> eth::Log {
    eth::Log {
        address: token.to_vec(),
        topics: vec![hash(signature.as_bytes()).to_vec(), word(from).unwrap().to_vec(), word(to).unwrap().to_vec()],
        data: word(&[1]).unwrap().to_vec(),
        ordinal: 10,
        ..Default::default()
    }
}

#[test]
fn captured_no_write_block_emits_3001_computed_holders_without_guessing_the_stored_holder() {
    let b = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122288080-computed-tx.pb").as_slice()).unwrap();
    let l = layouts();
    assert!(changes(&b, &l).unwrap().is_empty());
    let events = project(&b, &l).unwrap();
    assert_eq!(events.balances.len(), 3001);
    let rule = l[0].address_hash_balance.as_ref().unwrap();
    assert!(events.balances.iter().all(|r| rule.amount(&r.address).as_ref() == Some(&r.amount)));
    assert!(!events.balances.iter().any(|r| rule.stored_addresses.values().any(|a| a == &r.address)));
}
#[test]
fn address_formula_matches_independently_captured_rpc_and_ignores_raw_words() {
    let report: Value = serde_json::from_str(include_str!("../docs/evidence/computed-address-characterization.json")).unwrap();
    let l = layouts();
    for check in report["independent_rpc_checks"].as_array().unwrap() {
        let address = hex_bytes(check["address"].as_str().unwrap()).unwrap();
        for raw in ["0", "1", "123"] {
            assert_eq!(l[0].project_amount(&address, raw), check["rpc"].as_str().unwrap());
        }
    }
    for address in l[0].address_hash_balance.as_ref().unwrap().stored_addresses.values() {
        assert_eq!(l[0].project_amount(address, "123"), "123");
    }
}
#[test]
fn computed_holder_selection_matches_reference_events_and_deduplicates() {
    let l = layouts();
    let token = &l[0].contract;
    let mut b = block();
    let mut ownership = log(token, "OwnershipTransferred(address,address)", &[6; 20], &[9; 20]);
    // The reference uses FiatToken's non-indexed ownership event ABI.
    ownership.data = ownership.topics[1..].concat();
    ownership.topics.truncate(1);
    b.transaction_traces = vec![tx(eth::Call {
        logs: vec![
            log(token, "Transfer(address,address,uint256)", &[6; 20], &[7; 20]),
            log(token, "Approval(address,address,uint256)", &[6; 20], &[7; 20]),
            ownership,
        ],
        ..Default::default()
    })];
    let events = project(&b, &l).unwrap();
    assert_eq!(events.balances.len(), 5);
    let observed = events.balances.iter().map(|r| r.address.clone()).collect::<BTreeSet<_>>();
    assert_eq!(observed, [token.clone(), vec![6; 20], vec![7; 20], vec![8; 20], vec![9; 20]].into());
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
    b.transaction_traces[0].calls[0].state_reverted = false;
    b.transaction_traces[0].status = 2;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn computed_unknown_and_malformed_events_stop_instead_of_silently_losing_holders() {
    let l = layouts();
    let mut b = block();
    let mut event = log(&l[0].contract, "Transfer(address,address,uint256)", &[6; 20], &[7; 20]);
    event.data.clear();
    b.transaction_traces = vec![tx(eth::Call {
        logs: vec![event],
        ..Default::default()
    })];
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].logs[0].topics[0] = vec![1; 32];
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].logs[0].address = vec![0xbb; 20];
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn computed_selector_changes_and_restore_fail_but_packed_flags_and_reverts_do_not() {
    let l = layouts();
    let rule = l[0].address_hash_balance.as_ref().unwrap();
    let (slot, address) = rule.stored_addresses.first_key_value().unwrap();
    let old = word(address).unwrap().to_vec();
    let mut flags = old.clone();
    flags[..12].fill(255);
    let mut b = block();
    b.transaction_traces = vec![tx(eth::Call {
        storage_changes: vec![eth::StorageChange {
            address: l[0].contract.clone(),
            key: slot.to_vec(),
            old_value: old.clone(),
            new_value: flags,
            ordinal: 10,
        }],
        ..Default::default()
    })];
    assert!(project(&b, &l).unwrap().balances.is_empty());
    b.transaction_traces[0].calls[0].storage_changes[0].new_value = word(&[7; 20]).unwrap().to_vec();
    assert!(project(&b, &l).is_err());
    let first = b.transaction_traces[0].calls[0].storage_changes[0].clone();
    b.transaction_traces[0].calls[0].storage_changes.push(eth::StorageChange {
        old_value: first.new_value.clone(),
        new_value: old,
        ordinal: 20,
        ..first
    });
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn computed_and_stored_write_projection_preserves_raw_continuity_and_zero_balances() {
    let l = layouts();
    let mut b = block();
    let stored = l[0]
        .address_hash_balance
        .as_ref()
        .unwrap()
        .stored_addresses
        .values()
        .find(|a| a.iter().any(|b| *b != 0))
        .unwrap()
        .clone();
    let mut c = eth::Call {
        address: l[0].contract.clone(),
        ..Default::default()
    };
    for (i, address) in [vec![6; 20], stored.clone()].into_iter().enumerate() {
        let mut preimage = word(&address).unwrap().to_vec();
        preimage.extend(l[0].balance_slot);
        let key = hash(&preimage);
        c.keccak_preimages.insert(hex::encode(key), hex::encode(preimage));
        c.storage_changes.push(eth::StorageChange {
            address: l[0].contract.clone(),
            key: key.to_vec(),
            old_value: vec![5],
            new_value: vec![0],
            ordinal: 10 + i as u64,
        });
    }
    c.logs.push(log(&l[0].contract, "Transfer(address,address,uint256)", &[6; 20], &stored));
    b.transaction_traces = vec![tx(c)];
    let events = project(&b, &l).unwrap();
    assert_eq!(events.balances.iter().find(|r| r.address == stored).unwrap().amount, "0");
    assert_ne!(events.balances.iter().find(|r| r.address == vec![6; 20]).unwrap().amount, "0");
    assert_eq!(events.balances.iter().filter(|r| r.address == vec![6; 20]).count(), 1);
    let mut bad = b.transaction_traces[0].calls[0].storage_changes[0].clone();
    bad.ordinal = 30;
    bad.old_value = vec![1];
    bad.new_value = vec![2];
    b.transaction_traces[0].calls[0].storage_changes.push(bad);
    assert!(project(&b, &l).is_err());
}
#[test]
fn computed_configuration_rejects_division_by_zero_overflow_and_hidden_dependencies() {
    for case in 0..7 {
        let mut p = params();
        match case {
            0 => p[0]["address_hash_balance"]["modulus"] = json!(format!("0x{}", "00".repeat(32))),
            1 => p[0]["address_hash_balance"]["offset"] = json!(format!("0x{}", "ff".repeat(32))),
            2 => p[0]["address_hash_balance"]["multiplier"] = json!(format!("0x{}", "ff".repeat(32))),
            3 => p[0]["other_slots"].as_array_mut().unwrap().push(json!(format!("0x{:064x}", 3))),
            4 => {
                let dep = p[0]["address_hash_balance"]["stored_addresses"][0].clone();
                p[0]["address_hash_balance"]["stored_addresses"].as_array_mut().unwrap().push(dep);
            }
            5 => p[0]["zero_balance"] = json!({"value":format!("0x{:064x}",1)}),
            6 => p[0]["deployment"] = json!({"block":80,"block_hash":format!("0x{}","01".repeat(32))}),
            _ => unreachable!(),
        }
        assert!(layout::parse(&p.to_string()).is_err(), "case {case}");
    }
}
