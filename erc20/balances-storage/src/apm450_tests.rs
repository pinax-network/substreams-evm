use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/apm450/layouts.json")).unwrap()
}
fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/apm450/cases.json")).unwrap()
}
fn captured(case: &Value) -> eth::Block {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/apm450");
    eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap()
}
fn empty_block() -> eth::Block {
    let mut block = captured(&cases()[0]);
    block.transaction_traces.clear();
    block.system_calls.clear();
    block
}
fn preimage(holder: &[u8], root: &[u8; 32]) -> [u8; 64] {
    let mut value = [0; 64];
    value[12..32].copy_from_slice(holder);
    value[32..].copy_from_slice(root);
    value
}
fn storage(layout: &VerifiedLayout, key: &[u8], new_value: Vec<u8>, ordinal: u64) -> eth::StorageChange {
    eth::StorageChange {
        address: layout.contract.clone(),
        key: key.to_vec(),
        old_value: if new_value.iter().all(|b| *b == 0) { vec![1] } else { vec![] },
        new_value,
        ordinal,
    }
}

#[test]
fn apm_captured_blocks_match_independent_rpc_and_require_reviewed_allowance() {
    let layout = layouts().remove(0);
    let cases = cases();
    assert_eq!(cases.len(), 4);
    let mut checked = 0;
    for case in cases {
        let block = captured(&case);
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        let events = project(&block, std::slice::from_ref(&layout)).unwrap();
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&layout.contract)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        checked += expected.len();
        if block.number == 122288187 {
            let mut unreviewed = layout.clone();
            unreviewed.other_mapping_slots.clear();
            assert!(project(&block, &[unreviewed]).unwrap_err().to_string().contains("unresolved storage"));
        }
    }
    assert_eq!(checked, 8);
}

#[test]
fn apm_rpc_checkpoint_and_original_mismatches_require_the_zero_word_fallback() {
    let layout = layouts().remove(0);
    let mut raw_only = layout.clone();
    raw_only.zero_balance = None;
    let mut checked = 0;
    let mut fallback = 0;
    for line in include_str!("../tests/fixtures/apm450/checkpoint.jsonl").lines() {
        let row: Value = serde_json::from_str(line).unwrap();
        let holder = hex_bytes(row["address"].as_str().unwrap()).unwrap();
        let raw = row["storage"].as_str().unwrap();
        assert_eq!(layout.project_amount(&holder, raw), row["rpc"].as_str().unwrap());
        if raw == "0" {
            assert_eq!(row["rpc"], "7000000000");
            assert_ne!(raw_only.project_amount(&holder, raw), row["rpc"]);
            fallback += 1;
        }
        checked += 1;
    }
    assert_eq!((checked, fallback), (11, 4));
    let old: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/apm450/prestate.json")).unwrap();
    assert_eq!(old.len(), 2);
    for row in old {
        let holder = hex_bytes(row["holder"].as_str().unwrap()).unwrap();
        assert_eq!(row["raw"], "0");
        assert_eq!(layout.project_amount(&holder, "0"), row["rpc"].as_str().unwrap());
        assert_eq!(raw_only.project_amount(&holder, "0"), row["plain_mapping"].as_str().unwrap());
        assert_ne!(row["plain_mapping"], row["rpc"]);
    }
}

#[test]
fn apm_raw_and_fallback_words_match_all_independent_rpc_controls() {
    let controls: Value = serde_json::from_str(include_str!("../tests/fixtures/apm450/controls.json")).unwrap();
    let mut layout = layouts().remove(0);
    let rows = controls["tokens"][0]["raw_controls"].as_array().unwrap();
    assert_eq!(rows.len(), 36);
    for control in rows {
        let holder = hex_bytes(control["holder"].as_str().unwrap()).unwrap();
        let raw: BigInt = control["raw"].as_str().unwrap().parse().unwrap();
        let fallback: BigInt = control["fallback"].as_str().unwrap().parse().unwrap();
        let bytes = fallback.to_bytes_be().1;
        let mut value = [0; 32];
        value[32 - bytes.len()..].copy_from_slice(&bytes);
        layout.zero_balance.as_mut().unwrap().value = value;
        let preimage = preimage(&holder, &layout.balance_slot);
        let key = hash(&preimage);
        let mut block = empty_block();
        block.system_calls.push(eth::Call {
            address: layout.contract.clone(),
            storage_changes: vec![storage(&layout, &key, raw.to_bytes_be().1, 1)],
            keccak_preimages: [(hex::encode(key), hex::encode(preimage))].into(),
            ..Default::default()
        });
        let events = project(&block, std::slice::from_ref(&layout)).unwrap();
        assert_eq!(events.balances.len(), 1);
        assert_eq!(events.balances[0].address, holder);
        assert_eq!(events.balances[0].amount, control["actual"].as_str().unwrap());
    }
}

#[test]
fn apm_ordinary_metadata_preserves_both_public_balance_branches() {
    let controls: Value = serde_json::from_str(include_str!("../tests/fixtures/apm450/controls.json")).unwrap();
    let layout = layouts().remove(0);
    let token = &controls["tokens"][0];
    let holder = hex_bytes(token["holder"].as_str().unwrap()).unwrap();
    let balance_preimage = preimage(&holder, &layout.balance_slot);
    let rows = token["metadata_controls"].as_array().unwrap();
    assert_eq!(rows.len(), 24);
    for control in rows {
        let mut preimages = std::collections::HashMap::from([(hex::encode(hash(&balance_preimage)), hex::encode(balance_preimage))]);
        if control["kind"] == "allowance" {
            let call = hex_bytes(control["field_call"]["data"].as_str().unwrap()).unwrap();
            let mut owner = [0; 64];
            owner[..32].copy_from_slice(&call[4..36]);
            owner[63] = 1;
            let mut spender = [0; 64];
            spender[..32].copy_from_slice(&call[36..68]);
            spender[32..].copy_from_slice(&hash(&owner));
            preimages.extend([
                (hex::encode(hash(&owner)), hex::encode(owner)),
                (hex::encode(hash(&spender)), hex::encode(spender)),
            ]);
        }
        let changes = control["state_diff"]
            .as_object()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, (key, value))| storage(&layout, &hex_bytes(key).unwrap(), hex_bytes(value.as_str().unwrap()).unwrap(), i as u64 + 1))
            .collect();
        let mut block = empty_block();
        block.system_calls.push(eth::Call {
            address: layout.contract.clone(),
            storage_changes: changes,
            keccak_preimages: preimages,
            ..Default::default()
        });
        let events = project(&block, std::slice::from_ref(&layout)).unwrap();
        assert_eq!(events.balances.len(), 1);
        assert_eq!(events.balances[0].address, holder);
        assert_eq!(events.balances[0].amount, control["balance_of"].as_str().unwrap());
    }
}

#[test]
fn apm_fallback_changes_and_unreviewed_custom_admin_state_remain_protected() {
    let layout = layouts().remove(0);
    let mut block = empty_block();
    let slot = layout.zero_balance.as_ref().unwrap().storage_slot.unwrap();
    let mut change = storage(&layout, &slot, vec![1], 1);
    change.old_value = layout.zero_balance.as_ref().unwrap().value.to_vec();
    block.system_calls.push(eth::Call {
        address: layout.contract.clone(),
        storage_changes: vec![change],
        ..Default::default()
    });
    assert!(project(&block, std::slice::from_ref(&layout))
        .unwrap_err()
        .to_string()
        .contains("zero-balance dependency changed"));
    let mut restored = block.system_calls[0].storage_changes[0].clone();
    restored.old_value = vec![1];
    restored.new_value = layout.zero_balance.as_ref().unwrap().value.to_vec();
    restored.ordinal = 2;
    block.system_calls[0].storage_changes.push(restored);
    assert!(project(&block, std::slice::from_ref(&layout))
        .unwrap_err()
        .to_string()
        .contains("zero-balance dependency changed"));
    let mut root = [0; 32];
    root[31] = 16;
    let preimage = preimage(&[0xaa; 20], &root);
    let key = hash(&preimage);
    block.system_calls[0].storage_changes = vec![storage(&layout, &key, vec![1], 1)];
    block.system_calls[0].keccak_preimages = [(hex::encode(key), hex::encode(preimage))].into();
    assert!(project(&block, &[layout]).unwrap_err().to_string().contains("unresolved storage"));
}
