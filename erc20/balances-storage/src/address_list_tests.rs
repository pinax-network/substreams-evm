use super::*;
use serde_json::{json, Value};

fn scalar(n: u64) -> [u8; 32] {
    word(&n.to_be_bytes()).unwrap()
}
fn key(index: u64) -> [u8; 32] {
    let mut key = hash(&scalar(35));
    let mut carry = u128::from(index);
    for byte in key.iter_mut().rev() {
        carry += u128::from(*byte);
        *byte = carry as u8;
        carry >>= 8;
    }
    key
}
fn params() -> Value {
    json!([{"contract":format!("0x{}", "aa".repeat(20)), "balance_slot":format!("0x{:064x}", 1),
        "code_hash":format!("0x{}", "bb".repeat(32)), "address_lists":[format!("0x{:064x}", 35)]}])
}
fn case() -> (eth::Block, Vec<VerifiedLayout>) {
    let layouts = layout::parse(&params().to_string()).unwrap();
    let row = |key: [u8; 32], old: Vec<u8>, new: Vec<u8>, ordinal| eth::StorageChange {
        address: layouts[0].contract.clone(),
        key: key.to_vec(),
        old_value: old,
        new_value: new,
        ordinal,
    };
    let holder = vec![6; 20];
    let call = eth::Call {
        address: layouts[0].contract.clone(),
        storage_changes: vec![
            row(scalar(35), vec![2], vec![3], 10),
            row(key(2), vec![], holder.clone(), 20),
            row(mapping(&holder, &scalar(1)), vec![8], vec![9], 30),
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
                ..Default::default()
            }),
            transaction_traces: vec![eth::TransactionTrace {
                status: 1,
                from: holder,
                calls: vec![call],
                ..Default::default()
            }],
            ..Default::default()
        },
        layouts,
    )
}

#[test]
fn witnessed_address_appends_preserve_balances_without_array_preimages() {
    let (mut block, layouts) = case();
    let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
    let mut next = rows[..2].to_vec();
    next[0].old_value = vec![3];
    next[0].new_value = vec![4];
    next[0].ordinal = 40;
    next[1].key = key(3).to_vec();
    next[1].new_value = vec![7; 20];
    next[1].ordinal = 50;
    rows.extend(next);
    let events = project(&block, &layouts).unwrap();
    assert_eq!(events.balances.len(), 1);
    assert_eq!(events.balances[0].amount, "9");
    assert_eq!(events.balances[0].address, vec![6; 20]);
}

#[test]
fn null_address_append_keeps_its_noop_witness_without_emitting_balance_noops() {
    let (mut block, layouts) = case();
    let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
    rows[1].new_value = vec![];
    rows[2].new_value = rows[2].old_value.clone();
    assert!(project(&block, &layouts).unwrap().balances.is_empty());
    block.transaction_traces[0].calls[0].storage_changes.remove(1);
    assert!(project(&block, &layouts).unwrap_err().to_string().contains("missing its element"));
}

#[test]
fn last_u64_index_can_append_to_length_two_to_the_64() {
    let (mut block, layouts) = case();
    let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
    rows[0].old_value = scalar(u64::MAX).to_vec();
    let mut length = [0; 32];
    length[23] = 1;
    rows[0].new_value = length.to_vec();
    rows[1].key = key(u64::MAX).to_vec();
    assert_eq!(project(&block, &layouts).unwrap().balances.len(), 1);
    let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
    rows[0].old_value = length.to_vec();
    length[31] = 1;
    rows[0].new_value = length.to_vec();
    assert!(project(&block, &layouts).unwrap_err().to_string().contains("exceeds u64"));
}

#[test]
fn malformed_address_list_lengths_elements_and_order_fail_closed() {
    for variant in 0..16 {
        let (mut block, layouts) = case();
        let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
        match variant {
            0 => rows[0].new_value = vec![1],
            1 => rows[0].new_value = vec![4],
            2 => rows[0].new_value = vec![2],
            3 => rows[1].key = key(3).to_vec(),
            4 => rows[1].new_value = vec![1; 21],
            5 => rows[1].old_value = vec![1],
            6 => rows[1].ordinal = 9,
            7 => rows[1].ordinal = 10,
            8 => {
                rows.remove(0);
            }
            9 => {
                rows.remove(1);
            }
            10 => rows[0].ordinal = 0,
            11 => rows[1].ordinal = 0,
            12 => {
                let mut extra = rows[1].clone();
                extra.old_value = extra.new_value.clone();
                extra.ordinal = 40;
                rows.push(extra);
            }
            13 => {
                let mut extra = rows[0].clone();
                extra.ordinal = 40;
                rows.push(extra);
            }
            14 => rows[1].address = vec![0xcc; 20],
            15 => rows[1].new_value = vec![0; 33],
            _ => unreachable!(),
        }
        assert!(project(&block, &layouts).is_err(), "variant {variant}");
    }
}

#[test]
fn reverted_and_failed_appends_do_not_authorize_persisted_elements() {
    for failed_tx in [false, true] {
        let (mut block, layouts) = case();
        let mut witness = block.transaction_traces[0].calls[0].clone();
        witness.storage_changes.truncate(1);
        block.transaction_traces[0].calls[0].storage_changes.remove(0);
        if failed_tx {
            block.transaction_traces.push(eth::TransactionTrace {
                status: 2,
                calls: vec![witness],
                ..Default::default()
            });
        } else {
            witness.state_reverted = true;
            block.transaction_traces[0].calls.push(witness);
        }
        assert!(project(&block, &layouts).is_err());
        block.transaction_traces[0].status = 2;
        assert!(project(&block, &layouts).unwrap().balances.is_empty());
    }
}

#[test]
fn address_list_roots_cannot_overlap_configured_fields() {
    let root = format!("0x{:064x}", 35);
    for variant in 0..9 {
        let mut p = params();
        match variant {
            0 => p[0]["balance_slot"] = json!(root),
            1 => p[0]["other_slots"] = json!([root]),
            2 => p[0]["other_mapping_slots"] = json!([root]),
            3 => p[0]["other_mapping_words"] = json!({root.clone():4}),
            4 => p[0]["address_lists"] = json!([root, root]),
            5 => p[0]["zero_balance"] = json!({"value":format!("0x{:064x}",0),"storage_slot":root}),
            6 => p[0]["voting_checkpoints"] = json!({"clock":"block_number","slots":[root]}),
            7 => p[0]["voting_checkpoints"] = json!({"clock":"block_number","mapping_slots":[root]}),
            8 => {
                p[0]["proxy"] = json!({"implementation":format!("0x{}","cc".repeat(20)),"code_hash":format!("0x{}","dd".repeat(32)),"implementation_slot":root})
            }
            _ => unreachable!(),
        }
        assert!(layout::parse(&p.to_string()).is_err(), "variant {variant}");
    }
}

fn removal(swap: bool) -> (eth::Block, Vec<VerifiedLayout>) {
    let (mut block, layouts) = case();
    let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
    let mut clear = rows[1].clone();
    clear.old_value = vec![7; 20];
    clear.new_value.clear();
    clear.ordinal = 20;
    rows[0].old_value = vec![3];
    rows[0].new_value = vec![2];
    rows[0].ordinal = 21;
    rows[1] = clear.clone();
    if swap {
        let mut copy = clear;
        copy.key = key(0).to_vec();
        copy.old_value = vec![8; 20];
        copy.new_value = vec![7; 20];
        copy.ordinal = 19;
        rows.push(copy);
    }
    (block, layouts)
}

#[test]
fn witnessed_tail_and_swap_removals_preserve_balances() {
    for swap in [false, true] {
        let (block, layouts) = removal(swap);
        let events = project(&block, &layouts).unwrap();
        assert_eq!(events.balances.len(), 1);
        assert_eq!(events.balances[0].amount, "9");
    }
}

#[test]
fn append_pop_and_reappend_can_reuse_an_element_with_distinct_witnesses() {
    let (mut block, layouts) = case();
    let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
    let mut clear = rows[1].clone();
    clear.old_value = clear.new_value.clone();
    clear.new_value.clear();
    clear.ordinal = 40;
    let mut pop = rows[0].clone();
    pop.old_value = vec![3];
    pop.new_value = vec![2];
    pop.ordinal = 41;
    let mut append = rows[0].clone();
    append.ordinal = 50;
    let mut element = rows[1].clone();
    element.ordinal = 51;
    rows.extend([clear, pop, append, element]);
    assert_eq!(project(&block, &layouts).unwrap().balances[0].amount, "9");
    // A missing zero-to-zero clear cannot reuse the append's witness.
    let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
    rows[1].new_value.clear();
    rows.remove(3);
    assert!(project(&block, &layouts).is_err());
}

#[test]
fn malformed_removals_never_authorize_other_element_writes() {
    for variant in 0..14 {
        let (mut block, layouts) = removal(true);
        let rows = &mut block.transaction_traces[0].calls[0].storage_changes;
        match variant {
            0 => rows[0].new_value = vec![1],
            1 => rows[1].new_value = vec![7; 20],
            2 => rows[1].old_value = vec![7; 21],
            3 => rows[1].ordinal = 22,
            4 => rows[1].ordinal = 0,
            5 => rows[3].key = key(3).to_vec(),
            6 => rows[3].new_value = vec![8; 20],
            7 => rows[3].old_value = vec![8; 21],
            8 => rows[3].ordinal = 22,
            9 => rows[3].ordinal = 20,
            10 => {
                rows.remove(1);
            }
            11 => {
                rows.remove(0);
            }
            12 => {
                let mut extra = rows[3].clone();
                extra.key = key(1).to_vec();
                extra.ordinal = 18;
                rows.push(extra);
            }
            13 => {
                let mut extra = rows[1].clone();
                extra.ordinal = 25;
                extra.old_value.clear();
                extra.new_value = vec![9; 20];
                rows.push(extra);
            }
            _ => unreachable!(),
        }
        assert!(project(&block, &layouts).is_err(), "variant {variant}");
    }
}

#[test]
fn null_tail_removal_requires_its_noop_and_failed_calls_cannot_supply_it() {
    let (mut block, layouts) = removal(false);
    block.transaction_traces[0].calls[0].storage_changes[1].old_value.clear();
    assert!(project(&block, &layouts).is_ok());
    let clear = block.transaction_traces[0].calls[0].storage_changes.remove(1);
    block.transaction_traces[0].calls.push(eth::Call {
        state_reverted: true,
        storage_changes: vec![clear],
        ..Default::default()
    });
    assert!(project(&block, &layouts).is_err());
}

#[test]
fn captured_shareholder_removal_matches_all_six_historical_rpc_balances() {
    use prost::Message;
    let layouts = layout::parse(include_str!("../tests/fixtures/shareholder-removal/layouts.json")).unwrap();
    let report: Value = serde_json::from_str(include_str!("../tests/fixtures/shareholder-removal/before.json")).unwrap();
    assert_eq!(report["error"], "address-list length must append one");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shareholder-removal/block.pb");
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.hash, hex_bytes(report["hash"].as_str().unwrap()).unwrap());
    assert_eq!(block.number, report["block"].as_u64().unwrap());
    let expected = report["balance_checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["new_rpc"].as_str().unwrap().to_owned()))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(expected.len(), 6);
    let events = project(&block, &layouts).unwrap();
    assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&layouts[0].contract)));
    assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
}
