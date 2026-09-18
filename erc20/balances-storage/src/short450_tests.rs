use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/short450/layouts.json")).unwrap()
}

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/short450/cases.json")).unwrap()
}

fn controls() -> Value {
    serde_json::from_str(include_str!("../tests/fixtures/short450/controls.json")).unwrap()
}

fn captured(case: &Value) -> eth::Block {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/short450");
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
    assert_eq!(tokens.len(), 2);
    assert_eq!(checked, 4);
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
    assert_eq!(checked, 40);
    assert_eq!(nulls, 8);
}

#[test]
fn short450_fixed_fields_allowances_and_packed_controls_preserve_the_independent_rpc_balance() {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/short450/controls.json")).unwrap();
    let layouts = layouts();
    let mut checked = 0;
    for token in fixture["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let holder = hex_bytes(token["holder"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let mut preimage = [0; 64];
        preimage[12..32].copy_from_slice(&holder);
        preimage[32..].copy_from_slice(&layout.balance_slot);
        for control in token["metadata_controls"].as_array().unwrap() {
            let mut preimages: std::collections::HashMap<String, String> = serde_json::from_value(control["preimages"].clone()).unwrap();
            preimages.insert(hex::encode(hash(&preimage)), hex::encode(preimage));
            let changes = control["state_diff"]
                .as_object()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, (key, value))| {
                    let new_value = hex_bytes(value.as_str().unwrap()).unwrap();
                    eth::StorageChange {
                        address: contract.clone(),
                        key: hex_bytes(key).unwrap(),
                        old_value: if new_value.iter().all(|b| *b == 0) { vec![1] } else { vec![] },
                        new_value,
                        ordinal: i as u64 + 1,
                    }
                })
                .collect();
            let number = token["block"].as_u64().unwrap();
            let block = eth::Block {
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
                        address: contract.clone(),
                        storage_changes: changes,
                        keccak_preimages: preimages,
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            };
            let events = project(&block, std::slice::from_ref(layout)).unwrap();
            assert_eq!(events.balances.len(), 1, "{}", control["name"]);
            assert_eq!(events.balances[0].contract.as_ref(), Some(&contract));
            assert_eq!(events.balances[0].address, holder);
            assert_eq!(events.balances[0].amount, control["balance_of"].as_str().unwrap());
            checked += 1;
        }
    }
    assert_eq!(checked, 8);
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
fn proxy_and_implementation_code_changes_reject_without_holder_writes() {
    for token in controls()["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts().into_iter().find(|l| l.contract == contract).unwrap();
        let proxy = layout.proxy.as_ref().unwrap();
        for (address, hash) in [(&layout.contract, &layout.code_hash), (&proxy.implementation, &proxy.code_hash)] {
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
}

#[test]
fn pointer_changes_and_restoration_reject_without_holder_writes() {
    for token in controls()["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts().into_iter().find(|l| l.contract == contract).unwrap();
        let proxy = layout.proxy.as_ref().unwrap();
        let original = word(&proxy.implementation).unwrap();
        for restore in [false, true] {
            let mut block = blank(token);
            write(&mut block, &contract, &proxy.implementation_slot, &original, &[0; 32], 1);
            if restore {
                write(&mut block, &contract, &proxy.implementation_slot, &[0; 32], &original, 2);
            }
            assert!(project(&block, std::slice::from_ref(&layout)).is_err());
        }
    }
}

#[test]
fn packed_field_controls_bind_low_address_and_unknown_fixed_fields_stay_guarded() {
    let fixture = controls();
    let mut packed = 0;
    for token in fixture["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts().into_iter().find(|l| l.contract == contract).unwrap();
        for control in token["metadata_controls"].as_array().unwrap() {
            if control["kind"] != "packed_privileged_address_and_transfer_flag" {
                continue;
            }
            assert_eq!(control["field_call"]["data"], "0x02691bcb");
            let field = hex_bytes(control["state_diff"][control["storage_key"].as_str().unwrap()].as_str().unwrap()).unwrap();
            assert_eq!(
                BigInt::from_unsigned_bytes_be(&field[12..]).to_string(),
                control["field_value"].as_str().unwrap()
            );
            assert_eq!(control["balance_of"], "123");
            packed += 1;
        }
        let mut block = blank(token);
        let key = word(&[99]).unwrap();
        write(&mut block, &contract, &key, &[], &[1], 1);
        assert!(project(&block, std::slice::from_ref(&layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));
    }
    assert_eq!(packed, 4);
}
