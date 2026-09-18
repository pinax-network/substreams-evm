use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/permit450/layouts.json")).unwrap()
}

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/permit450/cases.json")).unwrap()
}

fn captured(case: &Value) -> eth::Block {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/permit450");
    eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap()
}

#[test]
fn permit450_captured_blocks_match_independent_rpc() {
    let layouts = layouts();
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases() {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
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
fn permit450_fixed_fields_allowances_and_nonces_preserve_the_independent_rpc_balance() {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/permit450/controls.json")).unwrap();
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
    assert_eq!(checked, 68);
}

#[test]
fn permit450_raw_word_controls_match_rpc_including_null_address_filtering() {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/permit450/controls.json")).unwrap();
    let layouts = layouts();
    let mut checked = 0;
    let mut excluded_null = 0;
    for token in fixture["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        for control in token["raw_controls"].as_array().unwrap() {
            let holder = hex_bytes(control["address"].as_str().unwrap()).unwrap();
            let mut preimage = [0; 64];
            preimage[12..32].copy_from_slice(&holder);
            preimage[32..].copy_from_slice(&layout.balance_slot);
            let key = hash(&preimage);
            assert_eq!(key.to_vec(), hex_bytes(control["storage_key"].as_str().unwrap()).unwrap());
            let raw: BigInt = control["raw"].as_str().unwrap().parse().unwrap();
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
                        storage_changes: vec![eth::StorageChange {
                            address: contract.clone(),
                            key: key.to_vec(),
                            old_value: if raw == BigInt::zero() { vec![1] } else { vec![] },
                            new_value: raw.to_bytes_be().1,
                            ordinal: 1,
                        }],
                        keccak_preimages: std::collections::HashMap::from([(hex::encode(key), hex::encode(preimage))]),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            };
            let raw_changes = changes(&block, std::slice::from_ref(layout)).unwrap();
            assert_eq!(raw_changes.len(), 1);
            assert_eq!(raw_changes[0].address, holder);
            assert_eq!(raw_changes[0].amount, control["rpc"].as_str().unwrap());
            let events = project(&block, std::slice::from_ref(layout)).unwrap();
            if holder.iter().all(|b| *b == 0) {
                assert!(events.balances.is_empty());
                excluded_null += 1;
            } else {
                assert_eq!(events.balances.len(), 1);
                assert_eq!(events.balances[0].contract.as_ref(), Some(&contract));
                assert_eq!(events.balances[0].address, holder);
                assert_eq!(events.balances[0].amount, control["rpc"].as_str().unwrap());
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 40);
    assert_eq!(excluded_null, 8);
}

#[test]
fn permit450_unreviewed_mapping_writes_still_stop_processing() {
    let layouts = layouts();
    for case in cases() {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let mut block = captured(&case);
        let mut owner = [0; 64];
        owner[12..32].fill(0xaa);
        owner[63] = 13;
        let mut spender = [0; 64];
        spender[12..32].fill(0xbb);
        spender[32..].copy_from_slice(&hash(&owner));
        let key = hash(&spender);
        block.transaction_traces.push(eth::TransactionTrace {
            status: eth::TransactionTraceStatus::Succeeded as i32,
            calls: vec![eth::Call {
                address: contract.clone(),
                storage_changes: vec![eth::StorageChange {
                    address: contract.clone(),
                    key: key.to_vec(),
                    old_value: vec![],
                    new_value: vec![1],
                    ordinal: 1_000_000,
                }],
                keccak_preimages: std::collections::HashMap::from([(hex::encode(hash(&owner)), hex::encode(owner)), (hex::encode(key), hex::encode(spender))]),
                ..Default::default()
            }],
            ..Default::default()
        });
        assert!(project(&block, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));
    }
}

#[test]
fn permit450_unknown_fixed_fields_remain_guarded() {
    let configured = layouts();
    for case in cases() {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = configured.iter().find(|l| l.contract == contract).unwrap();
        let mut block = captured(&case);
        block.transaction_traces.push(eth::TransactionTrace {
            status: eth::TransactionTraceStatus::Succeeded as i32,
            calls: vec![eth::Call {
                address: contract.clone(),
                storage_changes: vec![eth::StorageChange {
                    address: contract.clone(),
                    key: word(&[99]).unwrap().to_vec(),
                    new_value: vec![1],
                    ordinal: 1_000_000,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        });
        assert!(project(&block, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));
    }
}

#[test]
fn pin_original_unresolved_write_is_a_verified_allowance() {
    let mut layout = layouts()
        .into_iter()
        .find(|l| hex::encode(&l.contract) == "b86126b872e3f23bc62d6a4a3152ec65202a2073")
        .unwrap();
    let case = cases()
        .into_iter()
        .find(|c| c["contract"] == "0xb86126b872e3f23bc62d6a4a3152ec65202a2073")
        .unwrap();
    let block = captured(&case);
    let key = word(&hex_bytes("bdc3ccb0d2a426d29654cf84a816e2c70b3b893015a73ea2de9fcd0000b92a42").unwrap()).unwrap();
    let mut preimages = BTreeMap::new();
    let mut present = false;
    for call in block.transaction_traces.iter().flat_map(|t| &t.calls).chain(&block.system_calls) {
        for (encoded_key, encoded_preimage) in &call.keccak_preimages {
            let bytes = hex_bytes(encoded_preimage).unwrap();
            let digest = word(&hex_bytes(encoded_key).unwrap()).unwrap();
            assert_eq!(digest, hash(&bytes));
            preimages.insert(digest, bytes);
        }
        present |= call.storage_changes.iter().any(|s| s.address == layout.contract && s.key == key);
    }
    assert!(present);
    let mut allowance_root = [0; 32];
    allowance_root[31] = 2;
    assert!(mapping_has_base(key, &preimages, &allowance_root));
    assert!(project(&block, std::slice::from_ref(&layout)).is_ok());
    layout.other_mapping_slots.clear();
    assert!(project(&block, &[layout]).unwrap_err().to_string().contains("unresolved storage"));
}

#[test]
fn pin_burned_null_balance_is_filtered_without_losing_pair_balances() {
    let layout = layouts()
        .into_iter()
        .find(|l| hex::encode(&l.contract) == "b86126b872e3f23bc62d6a4a3152ec65202a2073")
        .unwrap();
    let case = cases()
        .into_iter()
        .find(|c| c["contract"] == "0xb86126b872e3f23bc62d6a4a3152ec65202a2073")
        .unwrap();
    let block = captured(&case);
    let rows = changes(&block, std::slice::from_ref(&layout)).unwrap();
    let null = rows.iter().find(|r| r.address.iter().all(|b| *b == 0)).unwrap();
    assert_ne!(null.amount, "0");
    assert_eq!(null.amount, case["excluded_null"]["rpc"].as_str().unwrap());
    assert_eq!(case["excluded_null"]["canonical_reference_has_null_holder"], false);
    let events = project(&block, std::slice::from_ref(&layout)).unwrap();
    assert!(events.balances.iter().all(|r| r.address.iter().any(|b| *b != 0)));
    assert_eq!(events.balances.len(), case["balances"].as_array().unwrap().len());
}

#[test]
fn slx_nonce_word_and_preimage_are_independently_bound() {
    let controls: Value = serde_json::from_str(include_str!("../tests/fixtures/permit450/controls.json")).unwrap();
    let token = controls["tokens"].as_array().unwrap().iter().find(|t| t["rank"] == 436).unwrap();
    let mut checked = 0;
    for c in token["metadata_controls"].as_array().unwrap().iter().filter(|c| c["kind"] == "nonce") {
        let key = c["storage_key"].as_str().unwrap();
        let value = BigInt::from_unsigned_bytes_be(&hex_bytes(c["state_diff"][key].as_str().unwrap()).unwrap());
        assert_eq!(value.to_string(), c["nonce_rpc"].as_str().unwrap());
        let mut preimage = [0; 64];
        preimage[12..32].copy_from_slice(&hex_bytes(token["holder"].as_str().unwrap()).unwrap());
        preimage[63] = 7;
        assert_eq!(hex::encode(hash(&preimage)), key.trim_start_matches("0x"));
        assert_eq!(c["preimages"][key.trim_start_matches("0x")], hex::encode(preimage));
        checked += 1;
    }
    assert_eq!(checked, 4);
}

#[test]
fn nonce_only_writes_do_not_emit_a_balance_or_require_a_cache() {
    let controls: Value = serde_json::from_str(include_str!("../tests/fixtures/permit450/controls.json")).unwrap();
    let token = controls["tokens"].as_array().unwrap().iter().find(|t| t["rank"] == 436).unwrap();
    let configured = layouts();
    let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
    let layout = configured.iter().find(|l| l.contract == contract).unwrap();
    for c in token["metadata_controls"].as_array().unwrap().iter().filter(|c| c["kind"] == "nonce") {
        let case = cases().into_iter().find(|c| c["contract"] == token["contract"]).unwrap();
        let mut block = captured(&case);
        let key = c["storage_key"].as_str().unwrap();
        block.system_calls.clear();
        block.transaction_traces = vec![eth::TransactionTrace {
            status: eth::TransactionTraceStatus::Succeeded as i32,
            calls: vec![eth::Call {
                address: contract.clone(),
                keccak_preimages: serde_json::from_value(c["preimages"].clone()).unwrap(),
                storage_changes: vec![eth::StorageChange {
                    address: contract.clone(),
                    key: hex_bytes(key).unwrap(),
                    old_value: vec![2],
                    new_value: hex_bytes(c["state_diff"][key].as_str().unwrap()).unwrap(),
                    ordinal: 1,
                }],
                ..Default::default()
            }],
            ..Default::default()
        }];
        assert!(project(&block, std::slice::from_ref(layout)).unwrap().balances.is_empty());
    }
}
