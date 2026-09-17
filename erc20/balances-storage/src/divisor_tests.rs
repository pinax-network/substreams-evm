use super::*;
use serde_json::{json, Value};

fn hex_word(n: u64) -> String {
    format!("0x{n:064x}")
}
fn params() -> Value {
    json!([{"contract":format!("0x{}","aa".repeat(20)),"balance_slot":hex_word(12),
        "code_hash":format!("0x{}","11".repeat(32)),"balance_divisor":{"value":hex_word(3),"storage_slot":hex_word(11)}}])
}
fn block(storage: Vec<eth::StorageChange>) -> eth::Block {
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
            from: vec![6; 20],
            calls: vec![eth::Call {
                address: vec![0xaa; 20],
                storage_changes: storage,
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}
fn write(key: Vec<u8>, old: u8, new: u8, ordinal: u64) -> eth::StorageChange {
    eth::StorageChange {
        address: vec![0xaa; 20],
        key,
        old_value: vec![old],
        new_value: vec![new],
        ordinal,
    }
}
#[test]
fn divisor_uses_unsigned_full_width_floor_division() {
    let mut l = layout::parse(&params().to_string()).unwrap();
    for (raw, expected) in [
        ("0", "0"),
        ("1", "0"),
        ("2", "0"),
        ("3", "1"),
        ("4", "1"),
        ("5", "1"),
        ("6", "2"),
        (
            "115792089237316195423570985008687907853269984665640564039457584007913129639935",
            "38597363079105398474523661669562635951089994888546854679819194669304376546645",
        ),
    ] {
        assert_eq!(l[0].project_amount(&[6; 20], raw), expected);
    }
    l[0].balance_divisor.as_mut().unwrap().value = [255; 32];
    assert_eq!(
        l[0].project_amount(&[6; 20], "115792089237316195423570985008687907853269984665640564039457584007913129639934"),
        "0"
    );
    assert_eq!(
        l[0].project_amount(&[6; 20], "115792089237316195423570985008687907853269984665640564039457584007913129639935"),
        "1"
    );
}
#[test]
fn divisor_keeps_raw_continuity_even_when_projected_values_are_equal() {
    let l = layout::parse(&params().to_string()).unwrap();
    let key = mapping(&[6; 20], &l[0].balance_slot).to_vec();
    let mut b = block(vec![write(key.clone(), 0, 4, 10), write(key, 4, 5, 20)]);
    let raw = changes(&b, &l).unwrap();
    assert_eq!((raw[0].old_amount.as_str(), raw[0].amount.as_str()), ("0", "5"));
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "1");
    b.transaction_traces[0].calls[0].storage_changes[1].old_value = vec![3];
    assert!(project(&b, &l).unwrap_err().to_string().contains("discontinuous"));
}
#[test]
fn divisor_change_and_restore_rejects_even_without_holder_writes() {
    let l = layout::parse(&params().to_string()).unwrap();
    let key = l[0].balance_divisor.as_ref().unwrap().storage_slot.to_vec();
    let mut b = block(vec![write(key.clone(), 3, 4, 10)]);
    assert!(project(&b, &l).unwrap_err().to_string().contains("rebuild retained holder balances"));
    b.transaction_traces[0].calls[0].storage_changes.push(write(key, 4, 3, 20));
    assert!(project(&b, &l).is_err());
    let c = b.transaction_traces[0].calls[0].clone();
    b.transaction_traces.clear();
    b.system_calls.push(c);
    assert!(project(&b, &l).is_err());
}
#[test]
fn reverted_failed_and_unchanged_divisor_writes_do_not_invalidate() {
    let l = layout::parse(&params().to_string()).unwrap();
    let key = l[0].balance_divisor.as_ref().unwrap().storage_slot.to_vec();
    let mut b = block(vec![write(key.clone(), 3, 4, 10)]);
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
    b.transaction_traces[0].calls[0].state_reverted = false;
    b.transaction_traces[0].status = 2;
    assert!(project(&b, &l).unwrap().balances.is_empty());
    let b = block(vec![write(key, 3, 3, 10)]);
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn divisor_rejects_zero_missing_and_overlapping_dependencies() {
    for case in 0..13 {
        let mut p = params();
        match case {
            0 => p[0]["balance_divisor"]["value"] = json!(hex_word(0)),
            1 => {
                p[0]["balance_divisor"].as_object_mut().unwrap().remove("storage_slot");
            }
            2 => p[0]["balance_divisor"]["storage_slot"] = json!(hex_word(12)),
            3 => p[0]["other_slots"] = json!([hex_word(11)]),
            4 => p[0]["other_mapping_slots"] = json!([hex_word(11)]),
            5 => p[0]["other_mapping_words"] = json!({hex_word(11):2}),
            6 => p[0]["address_lists"] = json!([hex_word(11)]),
            7 => p[0]["voting_checkpoints"] = json!({"clock":"block_number","slots":[hex_word(11)]}),
            8 => p[0]["zero_balance"] = json!({"value":hex_word(1)}),
            9 => p[0]["address_hash_balance"] = json!({"modulus":hex_word(1),"offset":hex_word(0),"multiplier":hex_word(1),"stored_addresses":[]}),
            10 => p[0]["deployment"] = json!({"block":1,"block_hash":hex_word(1)}),
            11 => p[0]["proxy"] = json!({"implementation_slot":hex_word(11),"implementation":format!("0x{}","bb".repeat(20)),"code_hash":hex_word(1)}),
            12 => p[0]["immutable_zero_mapping"] = json!(true),
            _ => unreachable!(),
        }
        assert!(layout::parse(&p.to_string()).is_err(), "case {case}");
    }
}
