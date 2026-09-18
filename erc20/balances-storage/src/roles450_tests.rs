use super::*;
use prost::Message;
use serde_json::Value;

fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(include_str!("../tests/fixtures/roles450/layouts.json")).unwrap()
}

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/roles450/cases.json")).unwrap()
}

fn captured(case: &Value) -> eth::Block {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/roles450");
    eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap()
}

#[test]
fn roles450_captured_blocks_match_independent_rpc() {
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
    assert_eq!(tokens.len(), 1);
    assert_eq!(checked, 2);
}

#[test]
fn roles450_fixed_fields_allowances_and_records_preserve_the_independent_rpc_balance() {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/roles450/controls.json")).unwrap();
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
            if control["configured_metadata"] == false {
                assert!(control["kind"].as_str().unwrap().starts_with("role-admin-"));
                assert!(project(&block, std::slice::from_ref(layout))
                    .unwrap_err()
                    .to_string()
                    .contains("unresolved storage"));
                continue;
            }
            let events = project(&block, std::slice::from_ref(layout)).unwrap();
            assert_eq!(events.balances.len(), 1, "{}", control["name"]);
            assert_eq!(events.balances[0].contract.as_ref(), Some(&contract));
            assert_eq!(events.balances[0].address, holder);
            assert_eq!(events.balances[0].amount, control["balance_of"].as_str().unwrap());
            checked += 1;
        }
    }
    assert_eq!(checked, 40);
}

#[test]
fn roles450_raw_word_controls_match_rpc_including_null_address_filtering() {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/roles450/controls.json")).unwrap();
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
    assert_eq!(checked, 20);
    assert_eq!(excluded_null, 4);
}

#[test]
fn roles450_unreviewed_mapping_writes_still_stop_processing() {
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
fn roles450_packed_metadata_is_required_and_unknown_fixed_fields_reject() {
    let configured = layouts();
    let controls: Value = serde_json::from_str(include_str!("../tests/fixtures/roles450/controls.json")).unwrap();
    let mut checked = 0;
    for token in controls["tokens"].as_array().unwrap() {
        let contract = hex_bytes(token["contract"].as_str().unwrap()).unwrap();
        let layout = configured.iter().find(|l| l.contract == contract).unwrap();
        let case = cases().into_iter().find(|c| c["contract"] == token["contract"]).unwrap();
        for control in token["packed_field_controls"].as_array().unwrap() {
            let mut block = captured(&case);
            let expected = project(&block, std::slice::from_ref(layout)).unwrap();
            let slot: [u8; 32] = hex_bytes(control["slot"].as_str().unwrap()).unwrap().try_into().unwrap();
            let value = hex_bytes(control["word"].as_str().unwrap()).unwrap();
            block.transaction_traces.push(eth::TransactionTrace {
                status: eth::TransactionTraceStatus::Succeeded as i32,
                calls: vec![eth::Call {
                    address: contract.clone(),
                    storage_changes: vec![eth::StorageChange {
                        address: contract.clone(),
                        key: slot.to_vec(),
                        old_value: if value.iter().all(|b| *b == 0) { vec![1] } else { vec![] },
                        new_value: value,
                        ordinal: 1_000_000,
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            });
            assert_eq!(project(&block, std::slice::from_ref(layout)).unwrap(), expected);
            let mut unsupported = layout.clone();
            assert!(unsupported.other_slots.remove(&slot));
            assert!(project(&block, &[unsupported]).unwrap_err().to_string().contains("unresolved storage"));
            checked += 1;
        }
        let mut block = captured(&case);
        block.system_calls.push(eth::Call {
            address: contract.clone(),
            storage_changes: vec![eth::StorageChange {
                address: contract,
                key: word(&[99]).unwrap().to_vec(),
                new_value: vec![1],
                ordinal: 1_000_000,
                ..Default::default()
            }],
            ..Default::default()
        });
        assert!(project(&block, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));
    }
    assert_eq!(checked, 9);
}

fn roles450_metadata_block(token: &Value, control: &Value, key: &[u8]) -> eth::Block {
    let mut block = captured(&cases()[0]);
    block.transaction_traces.clear();
    block.system_calls.clear();
    block.transaction_traces.push(eth::TransactionTrace {
        status: eth::TransactionTraceStatus::Succeeded as i32,
        calls: vec![eth::Call {
            address: hex_bytes(token["contract"].as_str().unwrap()).unwrap(),
            storage_changes: vec![eth::StorageChange {
                address: hex_bytes(token["contract"].as_str().unwrap()).unwrap(),
                key: key.to_vec(),
                old_value: vec![],
                new_value: vec![1],
                ordinal: 1,
            }],
            keccak_preimages: serde_json::from_value(control["preimages"].clone()).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    });
    block
}

#[test]
fn roles450_metadata_only_writes_emit_no_balances_and_require_each_reviewed_rule() {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/roles450/controls.json")).unwrap();
    let token = &fixture["tokens"][0];
    let layout = &layouts()[0];
    let mut kinds = BTreeSet::new();
    for control in token["metadata_controls"].as_array().unwrap() {
        let key = hex_bytes(control["storage_key"].as_str().unwrap()).unwrap();
        let block = roles450_metadata_block(token, control, &key);
        let kind = control["kind"].as_str().unwrap();
        kinds.insert(kind.to_owned());
        if control["configured_metadata"] == false {
            assert!(kind.starts_with("role-admin-"));
            assert!(project(&block, std::slice::from_ref(layout))
                .unwrap_err()
                .to_string()
                .contains("unresolved storage"));
            continue;
        }
        assert!(project(&block, std::slice::from_ref(layout)).unwrap().balances.is_empty());
        let mut missing = layout.clone();
        match kind {
            "allowance" => assert!(missing.other_mapping_slots.remove(&word(&[1]).unwrap())),
            "blacklist" => assert!(missing.other_mapping_slots.remove(&word(&[8]).unwrap())),
            k if k.starts_with("role-membership-") => assert!(missing.other_mapping_slots.remove(&word(&[6]).unwrap())),
            k if k.starts_with("fixed-") => assert!(missing.other_slots.remove(&word(&key).unwrap())),
            _ => panic!("unreviewed control kind"),
        }
        assert!(project(&block, &[missing]).unwrap_err().to_string().contains("unresolved storage"));
    }
    assert_eq!(kinds.len(), 13);
}

#[test]
fn roles450_role_admin_and_member_offsets_reject_and_membership_requires_valid_preimages() {
    let fixture: Value = serde_json::from_str(include_str!("../tests/fixtures/roles450/controls.json")).unwrap();
    let token = &fixture["tokens"][0];
    let layout = &layouts()[0];
    assert!(layout.other_mapping_words.is_empty());
    let mut checked_roles = BTreeSet::new();
    for control in token["metadata_controls"].as_array().unwrap() {
        let kind = control["kind"].as_str().unwrap();
        let Some(role) = kind.strip_prefix("role-membership-") else { continue };
        if !checked_roles.insert(role.to_owned()) {
            continue;
        }
        let key = hex_bytes(control["storage_key"].as_str().unwrap()).unwrap();
        let block = roles450_metadata_block(token, control, &key);
        assert!(project(&block, std::slice::from_ref(layout)).unwrap().balances.is_empty());
        // A membership boolean has no second word, even though the outer RoleData does.
        let outside = BigInt::from_unsigned_bytes_be(&key) + BigInt::from(1);
        let rejected = roles450_metadata_block(token, control, &word(&outside.to_bytes_be().1).unwrap());
        assert!(project(&rejected, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));

        let admin = token["metadata_controls"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["kind"] == format!("role-admin-{role}"))
            .unwrap();
        let admin_key = hex_bytes(admin["storage_key"].as_str().unwrap()).unwrap();
        let rejected = roles450_metadata_block(token, admin, &admin_key);
        assert!(project(&rejected, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));
        let outside = BigInt::from_unsigned_bytes_be(&admin_key) + BigInt::from(1);
        let rejected = roles450_metadata_block(token, admin, &word(&outside.to_bytes_be().1).unwrap());
        assert!(project(&rejected, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));

        let mut absent = block.clone();
        absent.transaction_traces[0].calls[0].keccak_preimages.clear();
        assert!(project(&absent, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("unresolved storage"));
        let mut corrupt = block;
        for value in corrupt.transaction_traces[0].calls[0].keccak_preimages.values_mut() {
            value.replace_range(0..2, "aa");
        }
        assert!(project(&corrupt, std::slice::from_ref(layout))
            .unwrap_err()
            .to_string()
            .contains("invalid Keccak preimage"));
    }
    assert_eq!(checked_roles.len(), 3);
}
