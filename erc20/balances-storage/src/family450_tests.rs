use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/family450/layouts.json")).unwrap()
}

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/family450/cases.json")).unwrap()
}

fn controls() -> Value {
    serde_json::from_str(include_str!("../tests/fixtures/family450/controls.json")).unwrap()
}

fn captured(case: &Value) -> eth::Block {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/family450");
    eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap()
}

fn blank(token: &Value) -> eth::Block {
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
        ..Default::default()
    }
}

fn write(block: &mut eth::Block, address: &[u8], key: &[u8], old: &[u8], new: &[u8], ordinal: u64) {
    block.transaction_traces.push(eth::TransactionTrace {
        status: eth::TransactionTraceStatus::Succeeded as i32,
        calls: vec![eth::Call {
            address: address.to_vec(),
            storage_changes: vec![eth::StorageChange {
                address: address.to_vec(),
                key: key.to_vec(),
                old_value: old.to_vec(),
                new_value: new.to_vec(),
                ordinal,
            }],
            ..Default::default()
        }],
        ..Default::default()
    });
}

#[test]
fn proxy_captured_blocks_match_independent_rpc_balances() {
    let layouts = layouts();
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases() {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        assert!(layout.deployment.is_none());
        let block = captured(&case);
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
    assert_eq!(tokens.len(), 13);
    assert_eq!(checked, 39);
}

#[test]
fn exact_raw_override_payloads_match_rpc_and_preserve_null_filtering() {
    let layouts = layouts();
    let mut checked = 0;
    let mut nulls = 0;
    for token in controls()["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        for control in token["raw_controls"].as_array().unwrap() {
            let holder = hex_bytes(control["address"].as_str().unwrap()).unwrap();
            let mut preimage = [0; 64];
            preimage[12..32].copy_from_slice(&holder);
            preimage[32..].copy_from_slice(&layout.balance_slot);
            let key = hash(&preimage);
            assert_eq!(key.to_vec(), hex_bytes(control["storage_key"].as_str().unwrap()).unwrap());
            let value = hex_bytes(control["state_diff"][control["storage_key"].as_str().unwrap()].as_str().unwrap()).unwrap();
            assert_eq!(BigInt::from_unsigned_bytes_be(&value).to_string(), control["raw"].as_str().unwrap());
            assert_eq!(control["state_diff"].as_object().unwrap().len(), 1);
            let mut block = blank(token);
            write(&mut block, &contract, &key, if value.iter().all(|b| *b == 0) { &[1] } else { &[] }, &value, 1);
            block.transaction_traces[0].calls[0]
                .keccak_preimages
                .insert(hex::encode(key), hex::encode(preimage));
            let raw = changes(&block, std::slice::from_ref(layout)).unwrap();
            assert_eq!(raw.len(), 1);
            assert_eq!(raw[0].address, holder);
            assert_eq!(raw[0].amount, control["rpc"].as_str().unwrap());
            let events = project(&block, std::slice::from_ref(layout)).unwrap();
            if holder.iter().all(|b| *b == 0) {
                assert!(events.balances.is_empty());
                nulls += 1;
            } else {
                assert_eq!(events.balances.len(), 1);
                assert_eq!(events.balances[0].address, holder);
                assert_eq!(events.balances[0].amount, control["rpc"].as_str().unwrap());
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 244);
    assert_eq!(nulls, 52);
}

#[test]
fn exact_fixed_metadata_controls_preserve_rpc_balances() {
    let layouts = layouts();
    let mut checked = 0;
    for token in controls()["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let holder = hex_bytes(token["holder"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let mut preimage = [0; 64];
        preimage[12..32].copy_from_slice(&holder);
        preimage[32..].copy_from_slice(&layout.balance_slot);
        for control in token["metadata_controls"].as_array().unwrap() {
            let mut block = blank(token);
            for (i, (key, value)) in control["state_diff"].as_object().unwrap().iter().enumerate() {
                let value = hex_bytes(value.as_str().unwrap()).unwrap();
                write(
                    &mut block,
                    &contract,
                    &hex_bytes(key).unwrap(),
                    if value.iter().all(|b| *b == 0) { &[1] } else { &[] },
                    &value,
                    i as u64 + 1,
                );
            }
            for tx in &mut block.transaction_traces {
                tx.calls[0].keccak_preimages.insert(hex::encode(hash(&preimage)), hex::encode(preimage));
            }
            let events = project(&block, std::slice::from_ref(layout)).unwrap();
            assert_eq!(events.balances.len(), 1);
            assert_eq!(events.balances[0].address, holder);
            assert_eq!(events.balances[0].amount, control["balance_of"].as_str().unwrap());
            checked += 1;
        }
    }
    assert_eq!(checked, 26);
}

#[test]
fn unknown_mapping_writes_remain_errors_without_holder_writes() {
    for token in controls()["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts().into_iter().find(|l| l.contract == contract).unwrap();
        let mut preimage = [0; 64];
        preimage[12..32].fill(0xaa);
        preimage[32..].fill(0xfe);
        let key = hash(&preimage);
        let mut block = blank(token);
        write(&mut block, &contract, &key, &[], &[1], 1);
        block.transaction_traces[0].calls[0]
            .keccak_preimages
            .insert(hex::encode(key), hex::encode(preimage));
        assert!(project(&block, &[layout]).unwrap_err().to_string().contains("unresolved storage"));
    }
}

#[test]
fn every_effective_implementation_code_change_is_rejected_without_holder_writes() {
    for token in controls()["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts().into_iter().find(|l| l.contract == contract).unwrap();
        let (address, hash) = if let Some(p) = &layout.minimal_proxy {
            (&p.implementation, &p.code_hash)
        } else if let Some(p) = &layout.proxy {
            (&p.implementation, &p.code_hash)
        } else {
            let p = layout.beacon_proxy.as_ref().unwrap();
            (&p.implementation, &p.implementation_code_hash)
        };
        for restore in [false, true] {
            let mut block = blank(token);
            block.code_changes = vec![eth::CodeChange {
                address: address.clone(),
                old_hash: hash.to_vec(),
                new_hash: vec![0x42; 32],
                ordinal: 1,
                ..Default::default()
            }];
            if restore {
                block.code_changes.push(eth::CodeChange {
                    address: address.clone(),
                    old_hash: vec![0x42; 32],
                    new_hash: hash.to_vec(),
                    ordinal: 2,
                    ..Default::default()
                });
            }
            assert!(project(&block, std::slice::from_ref(&layout)).is_err());
        }
    }
}

#[test]
fn aave_admin_changes_and_restoration_are_rejected_without_holder_writes() {
    let fixture = controls();
    let token = fixture["tokens"].as_array().unwrap().iter().find(|t| t["rank"] == 439).unwrap();
    let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
    let layout = layouts().into_iter().find(|l| l.contract == contract).unwrap();
    let admin = token["identity"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["role"] == "transparent admin qualification")
        .unwrap();
    let key = hex_bytes(admin["slot"].as_str().unwrap()).unwrap();
    let admin_key: [u8; 32] = key.clone().try_into().unwrap();
    assert!(!layout.other_slots.contains(&admin_key));
    assert_eq!(admin["actual_admin_response"]["error"]["code"], 3);
    assert_eq!(admin["zero_admin_response"]["error"]["code"], 3);
    let original = hex_bytes(admin["boundaries"][0]["admin_word"].as_str().unwrap()).unwrap();
    assert!(original.iter().any(|b| *b != 0));
    assert_eq!(admin["boundaries"][0]["admin_word"], admin["boundaries"][1]["admin_word"]);
    for restore in [false, true] {
        let mut block = blank(token);
        write(&mut block, &contract, &key, &original, &[0; 32], 1);
        if restore {
            write(&mut block, &contract, &key, &[0; 32], &original, 2);
        }
        let error = project(&block, std::slice::from_ref(&layout)).unwrap_err().to_string();
        assert!(error.contains("unresolved storage"));
        assert!(error.contains(&hex::encode(&key)));
    }
}
