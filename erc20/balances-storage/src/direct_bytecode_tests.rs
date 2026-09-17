use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/direct-bytecode/cases.json")).unwrap()
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/bsc-direct-bytecode-layouts.json")).unwrap()
}
fn captured(c: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/direct-bytecode")
        .join(c["fixture"].as_str().unwrap());
    eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap()
}

#[test]
fn eight_direct_bytecode_layouts_match_captured_historical_rpc() {
    let layouts = layouts();
    assert_eq!(layouts.len(), 97);
    let cases = cases();
    assert_eq!(cases.len(), 11);
    let mut contracts = BTreeSet::new();
    for c in cases {
        let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
        contracts.insert(contract.clone());
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let b = captured(&c);
        assert_eq!(b.number, c["block"].as_u64().unwrap());
        assert_eq!(b.hash, hex_bytes(c["hash"].as_str().unwrap()).unwrap());
        let events = project(&b, std::slice::from_ref(l)).unwrap();
        let expected = c["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| (hex_bytes(v["address"].as_str().unwrap()).unwrap(), v["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&contract)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
    }
    assert_eq!(contracts.len(), 8);
}

#[test]
fn burn_counter_block_tracking_and_temporary_flag_require_explicit_review() {
    let layouts = layouts();
    let cases = cases();
    for (rank, height, scalar, mapping_slot) in [(55, 122288579, Some(0), None), (62, 122288270, None, Some(15)), (62, 122288404, Some(19), None)] {
        let c = cases.iter().find(|c| c["rank"] == rank && c["block"] == height).unwrap();
        let contract = hex_bytes(c["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let b = captured(c);
        assert!(project(&b, std::slice::from_ref(l)).is_ok());
        let mut unsupported = l.clone();
        if let Some(slot) = scalar {
            assert!(unsupported.other_slots.remove(&word(&[slot]).unwrap()));
        }
        if let Some(slot) = mapping_slot {
            assert!(unsupported.other_mapping_slots.remove(&word(&[slot]).unwrap()));
        }
        assert!(project(&b, &[unsupported]).unwrap_err().to_string().contains("unresolved storage"));
    }
}

#[test]
fn captured_bookkeeping_writes_have_the_reviewed_counter_clock_and_flag_semantics() {
    let cases = cases();
    let sac = cases.iter().find(|c| c["rank"] == 55 && c["block"] == 122288579).unwrap();
    let contract = hex_bytes(sac["contract"].as_str().unwrap()).unwrap();
    let block = captured(sac);
    let burn = block
        .transaction_traces
        .iter()
        .flat_map(|tx| &tx.calls)
        .find(|c| c.address == contract && !c.state_reverted && c.input.starts_with(&[0x42, 0x96, 0x6c, 0x68]))
        .unwrap();
    let burned = BigInt::from_unsigned_bytes_be(&burn.input[4..36]);
    let counter = burn.storage_changes.iter().find(|s| word(&s.key).unwrap() == word(&[0]).unwrap()).unwrap();
    let supply = burn.storage_changes.iter().find(|s| word(&s.key).unwrap() == word(&[3]).unwrap()).unwrap();
    assert_eq!(
        BigInt::from_unsigned_bytes_be(&counter.new_value) - BigInt::from_unsigned_bytes_be(&counter.old_value),
        burned
    );
    assert_eq!(
        BigInt::from_unsigned_bytes_be(&supply.old_value) - BigInt::from_unsigned_bytes_be(&supply.new_value),
        burned
    );

    let au = cases.iter().find(|c| c["rank"] == 62 && c["block"] == 122288270).unwrap();
    let contract = hex_bytes(au["contract"].as_str().unwrap()).unwrap();
    let block = captured(au);
    let key = mapping(&hex_bytes("0xb197bc3fac512b24855d72194030d09167f0080f").unwrap(), &word(&[15]).unwrap());
    let clock = block
        .transaction_traces
        .iter()
        .flat_map(|tx| &tx.calls)
        .filter(|c| !c.state_reverted)
        .flat_map(|c| &c.storage_changes)
        .find(|s| s.address == contract && s.key == key)
        .unwrap();
    assert_eq!(BigInt::from_unsigned_bytes_be(&clock.new_value).to_string(), block.number.to_string());

    let flags = cases.iter().find(|c| c["rank"] == 62 && c["block"] == 122288404).unwrap();
    let block = captured(flags);
    let mut writes = block
        .transaction_traces
        .iter()
        .flat_map(|tx| &tx.calls)
        .filter(|c| !c.state_reverted)
        .flat_map(|c| &c.storage_changes)
        .filter(|s| s.address == contract && word(&s.key).unwrap() == word(&[19]).unwrap())
        .collect::<Vec<_>>();
    writes.sort_by_key(|s| s.ordinal);
    assert_eq!(writes.len(), 2);
    assert_eq!(word(&writes[0].old_value).unwrap(), word(&writes[1].new_value).unwrap());
    assert_eq!(word(&writes[0].new_value).unwrap(), word(&writes[1].old_value).unwrap());
    for (s, expected_flag) in writes.into_iter().zip([1, 0]) {
        let old = word(&s.old_value).unwrap();
        let new = word(&s.new_value).unwrap();
        assert_eq!(old[..11], new[..11]);
        assert_eq!(old[12..], new[12..]);
        assert_eq!(new[11], expected_flag);
    }
}
