use super::*;
use prost::Message;
use serde_json::json;

#[test]
fn captured_clone_initialization_and_distribution_retain_all_initial_holders() {
    let b = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122288172-clone-creation-tx.pb").as_slice()).unwrap();
    let all = layout::parse(include_str!("../tests/fixtures/bsc-clone-layouts.json")).unwrap();
    let l = all.iter().find(|l| l.minimal_proxy.is_some()).unwrap();
    let rows = changes(&b, std::slice::from_ref(l)).unwrap();
    assert_eq!(rows.len(), 143);
    assert!(rows.iter().all(|r| r.old_amount == "0"));
    let total = rows.iter().fold(BigInt::from(0), |sum, r| sum + r.amount.parse::<BigInt>().unwrap());
    assert_eq!(total.to_string(), "1000000000000000000000000000");
    assert_eq!(project(&b, std::slice::from_ref(l)).unwrap().balances.len(), 143);
}
#[test]
fn minimal_proxy_configuration_binds_exact_forwarding_code_target_and_kind() {
    let mut params: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-clone-layouts.json")).unwrap();
    let good = params.as_array_mut().unwrap().pop().unwrap();
    assert!(layout::parse(&json!([good]).to_string()).is_ok());
    for case in 0..5 {
        let mut bad = good.clone();
        match case {
            0 => bad["minimal_proxy"]["implementation"] = json!(format!("0x{}", "ab".repeat(20))),
            1 => bad["minimal_proxy"]["implementation"] = bad["contract"].clone(),
            2 => bad["code_hash"] = json!(format!("0x{}", "00".repeat(32))),
            3 => bad["minimal_proxy"]["implementation"] = json!(format!("0x{}", "00".repeat(20))),
            4 => {
                bad["proxy"] = json!({"implementation_slot":format!("0x{}","ff".repeat(32)),"implementation":format!("0x{}","ab".repeat(20)),"code_hash":format!("0x{}","bb".repeat(32))})
            }
            _ => unreachable!(),
        }
        assert!(layout::parse(&json!([bad]).to_string()).is_err(), "case {case}");
    }
    let mut plain = good.clone();
    plain.as_object_mut().unwrap().remove("deployment");
    assert!(layout::parse(&json!([plain]).to_string()).is_ok());
}
#[test]
fn minimal_proxy_rejects_implementation_changes_even_without_token_writes() {
    let mut l = layout::parse(include_str!("../tests/fixtures/bsc-clone-layouts.json")).unwrap();
    l.retain(|l| l.minimal_proxy.is_some());
    l[0].deployment = None;
    let implementation = l[0].minimal_proxy.as_ref().unwrap().implementation.clone();
    let c = eth::Call {
        code_changes: vec![eth::CodeChange {
            address: implementation,
            old_hash: vec![1; 32],
            new_hash: vec![2; 32],
            ordinal: 10,
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut b = block();
    b.transaction_traces = vec![tx(c.clone())];
    assert!(project(&b, &l).is_err());
    let mut restore = c.code_changes[0].clone();
    restore.old_hash = vec![2; 32];
    restore.new_hash = vec![1; 32];
    restore.ordinal = 20;
    b.transaction_traces[0].calls[0].code_changes.push(restore);
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}

fn deployed_case() -> (eth::Block, Vec<VerifiedLayout>) {
    let mut b = block();
    let mut l = layouts();
    l.truncate(1);
    l[0].code_hash = hash(&[0x60, 0x00]);
    l[0].deployment = Some(layout::VerifiedDeployment {
        block: b.number,
        block_hash: b.hash.clone().try_into().unwrap(),
    });
    let mut c = token_call(&l[0], &[6; 20], 0, 100);
    c.call_type = eth::CallType::Create as i32;
    c.begin_ordinal = 5;
    c.end_ordinal = 30;
    c.code_changes.push(eth::CodeChange {
        address: l[0].contract.clone(),
        old_hash: hash(&[]).to_vec(),
        new_hash: l[0].code_hash.to_vec(),
        new_code: vec![0x60, 0],
        ordinal: 20,
        ..Default::default()
    });
    b.transaction_traces = vec![tx(c)];
    (b, l)
}
#[test]
fn captured_first_create_matches_the_historical_rpc_initial_mint() {
    // Fixture retains the actual deployment transaction, header and identity;
    // unrelated transactions/system changes are omitted to keep it small.
    let block = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122288338-deployment-tx.pb").as_slice()).unwrap();
    let layouts = layout::parse(include_str!("../tests/fixtures/bsc-deployment-layouts.json")).unwrap();
    let token = layouts.iter().find(|l| l.deployment.is_some()).unwrap();
    let events = project(&block, std::slice::from_ref(token)).unwrap();
    assert_eq!(events.balances.len(), 1);
    assert_eq!(hex::encode(&events.balances[0].address), "3dc263d768385802c8d2d20e9f7f06f6e9128782");
    assert_eq!(events.balances[0].amount, "1000000000000000000000000000");
    assert_eq!(changes(&block, std::slice::from_ref(token)).unwrap()[0].old_amount, "0");
}
#[test]
fn qualified_create_emits_constructor_mint_and_later_same_block_changes() {
    let (mut b, l) = deployed_case();
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "100");
    let mut later = token_call(&l[0], &[6; 20], 100, 90);
    later.index = 1;
    later.storage_changes[0].ordinal = 40;
    b.transaction_traces[0].calls.push(later);
    let rows = changes(&b, &l).unwrap();
    assert_eq!((&*rows[0].old_amount, &*rows[0].amount), ("0", "90"));
}
#[test]
fn deployment_rejects_unpinned_missing_reverted_and_ambiguous_creation() {
    let (b, l) = deployed_case();
    let mut no_pin = l.clone();
    no_pin[0].deployment = None;
    assert!(project(&b, &no_pin).is_err());
    for case in 0..12 {
        let mut b = b.clone();
        match case {
            0 => b.hash[0] ^= 1,
            1 => b.transaction_traces[0].calls[0].code_changes.clear(),
            2 => b.transaction_traces[0].calls[0].state_reverted = true,
            3 => b.transaction_traces[0].status = eth::TransactionTraceStatus::Failed as i32,
            4 => {
                let c = b.transaction_traces[0].calls[0].code_changes[0].clone();
                b.transaction_traces[0].calls[0].code_changes.push(c);
            }
            5 => b.transaction_traces[0].calls[0].call_type = eth::CallType::Call as i32,
            6 => b.transaction_traces[0].calls[0].code_changes[0].new_code.push(1),
            7 => b.transaction_traces[0].calls[0].code_changes[0].old_code.push(1),
            8 => b.transaction_traces[0].calls[0].code_changes[0].old_hash = vec![0; 32],
            9 => b.transaction_traces[0].calls[0].code_changes[0].ordinal = 30,
            10 => b.transaction_traces[0].calls[0].begin_ordinal = 0,
            11 => {
                let c = b.transaction_traces[0].calls[0].code_changes.remove(0);
                b.code_changes.push(c);
            }
            _ => unreachable!(),
        }
        assert!(project(&b, &l).is_err(), "case {case}");
    }
}
#[test]
fn deployment_rejects_preexisting_storage_early_writes_and_later_redeployment() {
    let (b, l) = deployed_case();
    for case in 0..4 {
        let mut b = b.clone();
        match case {
            0 => b.transaction_traces[0].calls[0].storage_changes[0].old_value = vec![1],
            1 => b.transaction_traces[0].calls[0].storage_changes[0].ordinal = 4,
            2 => {
                b.number += 1;
                b.header.as_mut().unwrap().number += 1;
            }
            3 => {
                b.number -= 1;
                b.header.as_mut().unwrap().number -= 1;
                b.transaction_traces[0].calls[0].code_changes.clear();
            }
            _ => unreachable!(),
        }
        assert!(project(&b, &l).is_err(), "case {case}");
    }
    let mut before = b.clone();
    before.number -= 1;
    before.header.as_mut().unwrap().number -= 1;
    before.transaction_traces.clear();
    assert!(project(&before, &l).unwrap().balances.is_empty());
}
#[test]
fn deployment_configuration_requires_direct_mapping_and_valid_identity() {
    let mut p = json!([{"contract":format!("0x{}","aa".repeat(20)),"balance_slot":format!("0x{}","00".repeat(32)),"code_hash":format!("0x{}","11".repeat(32)),"deployment":{"block":1,"block_hash":format!("0x{}","22".repeat(32))}}]);
    assert!(layout::parse(&p.to_string()).is_ok());
    p[0]["zero_balance"] = json!({"value":format!("0x{}","01".repeat(32))});
    assert!(layout::parse(&p.to_string()).is_err());
    p[0].as_object_mut().unwrap().remove("zero_balance");
    p[0]["deployment"]["block"] = json!(0);
    assert!(layout::parse(&p.to_string()).is_err());
    p[0]["deployment"]["block"] = json!(1);
    p[0]["deployment"]["block_hash"] = json!(format!("0x{}", "00".repeat(32)));
    assert!(layout::parse(&p.to_string()).is_err());
}

fn slot(n: u8) -> [u8; 32] {
    let mut value = [0; 32];
    value[31] = n;
    value
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(
        &json!([
            {"contract":format!("0x{}", "aa".repeat(20)),"balance_slot":format!("0x{}",hex::encode(slot(7))),
             "code_hash":format!("0x{}","11".repeat(32)),"other_mapping_slots":[format!("0x{}",hex::encode(slot(8)))]},
            {"contract":format!("0x{}", "bb".repeat(20)),"balance_slot":format!("0x{}","ff".repeat(32)),
             "code_hash":format!("0x{}","22".repeat(32))}
        ])
        .to_string(),
    )
    .unwrap()
}
fn block() -> eth::Block {
    eth::Block {
        ver: 5,
        number: 122260950,
        hash: vec![1; 32],
        detail_level: eth::block::DetailLevel::DetaillevelExtended as i32,
        header: Some(eth::BlockHeader {
            number: 122260950,
            parent_hash: vec![2; 32],
            state_root: vec![3; 32],
            ..Default::default()
        }),
        ..Default::default()
    }
}
fn tx(call: eth::Call) -> eth::TransactionTrace {
    eth::TransactionTrace {
        status: eth::TransactionTraceStatus::Succeeded as i32,
        calls: vec![call],
        ..Default::default()
    }
}
fn token_call(layout: &VerifiedLayout, owner: &[u8], old: u8, new: u8) -> eth::Call {
    let mut preimage = vec![0; 64];
    preimage[12..32].copy_from_slice(owner);
    preimage[32..].copy_from_slice(&layout.balance_slot);
    let key = hash(&preimage);
    eth::Call {
        address: layout.contract.clone(),
        keccak_preimages: [(hex::encode(key), hex::encode(preimage))].into(),
        storage_changes: vec![eth::StorageChange {
            address: layout.contract.clone(),
            key: key.to_vec(),
            old_value: vec![old],
            new_value: vec![new],
            ordinal: 10,
        }],
        ..Default::default()
    }
}
#[test]
fn two_arbitrary_tokens_and_full_width_slots_use_same_shared_events() {
    let l = layouts();
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 5, 0)), tx(token_call(&l[1], &[9; 20], 2, 8))];
    let expected = balances_pb::Events {
        balances: vec![
            balances_pb::Balance {
                contract: Some(l[0].contract.clone()),
                address: vec![6; 20],
                amount: "0".into(),
            },
            balances_pb::Balance {
                contract: Some(l[1].contract.clone()),
                address: vec![9; 20],
                amount: "8".into(),
            },
        ],
    };
    assert_eq!(project(&b, &l).unwrap().encode_to_vec(), expected.encode_to_vec());
    assert_eq!(changes(&b, &l).unwrap()[0].old_amount, "5");
}
#[test]
fn last_persisted_value_wins_and_first_old_value_is_retained() {
    let l = layouts();
    let mut c = token_call(&l[0], &[6; 20], 5, 9);
    let mut last = c.storage_changes[0].clone();
    last.ordinal = 30;
    last.old_value = vec![9];
    last.new_value = vec![0];
    c.storage_changes.insert(0, last);
    let mut b = block();
    b.transaction_traces = vec![tx(c)];
    let rows = changes(&b, &l).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!((&*rows[0].old_amount, &*rows[0].amount, rows[0].ordinal), ("5", "0", 30));
}
#[test]
fn reverted_execution_does_not_emit_balances() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 5, 8);
    c.state_reverted = true;
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn holder_preimage_does_not_require_a_transfer_log() {
    let l = layouts();
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 8, 0))];
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "0");
}
#[test]
fn fallback_address_must_match_the_configured_mapping_hash() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 8, 2);
    c.keccak_preimages.clear();
    c.caller = vec![6; 20];
    b.transaction_traces = vec![tx(c)];
    assert_eq!(project(&b, &l).unwrap().balances.len(), 1);
    b.transaction_traces[0].calls[0].caller = vec![7; 20];
    assert!(project(&b, &l).is_err());
}
#[test]
fn corrupt_preimage_is_rejected() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 1, 2);
    c.keccak_preimages.values_mut().for_each(|v| *v = "00".into());
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).is_err());
}
#[test]
fn only_explicitly_configured_other_storage_is_ignored() {
    let mut l = layouts();
    let mut b = block();
    let mut p1 = vec![0; 64];
    p1[12..32].copy_from_slice(&[6; 20]);
    p1[32..].copy_from_slice(&slot(8));
    let h1 = hash(&p1);
    let mut p2 = vec![0; 64];
    p2[12..32].copy_from_slice(&[7; 20]);
    p2[32..].copy_from_slice(&h1);
    let h2 = hash(&p2);
    b.transaction_traces = vec![tx(eth::Call {
        address: l[0].contract.clone(),
        keccak_preimages: [(hex::encode(h1), hex::encode(p1)), (hex::encode(h2), hex::encode(p2))].into(),
        storage_changes: vec![eth::StorageChange {
            address: l[0].contract.clone(),
            key: h2.to_vec(),
            new_value: vec![9],
            ordinal: 10,
            ..Default::default()
        }],
        ..Default::default()
    })];
    assert!(project(&b, &l).unwrap().balances.is_empty());
    l[0].other_mapping_slots.clear();
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].storage_changes[0].key = slot(2).to_vec();
    assert!(project(&b, &l).is_err());
    l[0].other_slots.insert(slot(2));
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn reviewed_nested_struct_fields_do_not_mask_adjacent_unknown_writes() {
    let mut l = layouts();
    l[0].other_mapping_slots.clear();
    l[0].other_mapping_words.insert(slot(8), 2);
    let mut outer = vec![0; 64];
    outer[12..32].copy_from_slice(&[6; 20]);
    outer[32..].copy_from_slice(&slot(8));
    let mut inner = vec![0; 64];
    inner[31] = 5; // checkpoint index
    inner[32..].copy_from_slice(&hash(&outer));
    let key = hash(&inner);
    let mut call = token_call(&l[0], &[6; 20], 1, 2);
    call.keccak_preimages.insert(hex::encode(hash(&outer)), hex::encode(outer));
    call.keccak_preimages.insert(hex::encode(key), hex::encode(inner));
    call.storage_changes.push(eth::StorageChange {
        address: l[0].contract.clone(),
        key: (BigInt::from_unsigned_bytes_be(&key) + 1_u32).to_bytes_be().1,
        old_value: vec![2],
        new_value: vec![3],
        ordinal: 20,
    });
    let mut b = block();
    b.transaction_traces = vec![tx(call)];
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "2");
    l[0].other_mapping_words.insert(slot(8), 1);
    assert!(project(&b, &l).is_err());
    l[0].other_mapping_words.insert(slot(8), 2);
    b.transaction_traces[0].calls[0].storage_changes[1].key = (BigInt::from_unsigned_bytes_be(&key) + 2_u32).to_bytes_be().1;
    assert!(project(&b, &l).is_err());
    assert_eq!(subtract_offset([0; 32], 1), [255; 32]);
    let mut carry = slot(0);
    carry[30] = 1;
    assert_eq!(subtract_offset(carry, 1), slot(255));
}
#[test]
fn mapping_width_cannot_hide_balance_or_upgrade_storage() {
    let original: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-reviewed-layouts.json")).unwrap();
    for (base, width) in [
        (original[2]["balance_slot"].clone(), 2),
        (original[2]["proxy"]["implementation_slot"].clone(), 2),
        (json!(format!("0x{}", hex::encode(slot(9)))), 0),
        (json!(format!("0x{}", hex::encode(slot(9)))), 33),
    ] {
        let mut invalid = original.clone();
        invalid[2]["other_mapping_words"] = json!({base.as_str().unwrap():width});
        assert!(layout::parse(&invalid.to_string()).is_err());
    }
}
#[test]
fn preserves_uint256_max() {
    let l = layouts();
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 0, 1);
    c.storage_changes[0].new_value = vec![255; 32];
    b.transaction_traces = vec![tx(c)];
    assert_eq!(
        project(&b, &l).unwrap().balances[0].amount,
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    );
}
#[test]
fn fallback_changes_public_values_without_weakening_raw_continuity() {
    let mut l = layouts();
    l[0].zero_balance = Some(layout::VerifiedZeroBalance {
        value: slot(8),
        storage_slot: Some(slot(4)),
    });
    let mut b = block();
    let mut c = token_call(&l[0], &[6; 20], 3, 0);
    b.transaction_traces = vec![tx(c.clone())];
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "8");
    assert_eq!(changes(&b, &l).unwrap()[0].amount, "0");
    // 0 and 8 project to the same public amount but are distinct raw words.
    let mut broken = c.storage_changes[0].clone();
    broken.old_value = vec![8];
    broken.new_value = vec![2];
    broken.ordinal = 20;
    c.storage_changes.push(broken);
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).unwrap_err().to_string().contains("discontinuous"));
}
#[test]
fn seven_recorded_fallback_mismatches_match_with_explicit_reviewed_rules() {
    let layouts = layout::parse(include_str!("../tests/fixtures/bsc-fallback-layouts.json")).unwrap();
    let recorded: Vec<serde_json::Value> = serde_json::from_str(include_str!("../docs/evidence/top50-mismatches.json")).unwrap();
    assert_eq!(recorded.len(), 7);
    for row in recorded {
        let contract = hex_bytes(row["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let raw = row["storage"].as_str().unwrap();
        let rpc = row["rpc"].as_str().unwrap();
        assert_ne!(raw, rpc, "fixture must preserve the original failure");
        assert_eq!(l.project_amount(&[42; 20], raw), rpc);
        let owner = hex_bytes(row["address"].as_str().unwrap()).unwrap();
        assert_eq!(format!("0x{}", hex::encode(mapping(&owner, &l.balance_slot))), row["storage_key"]);
        let mut b = block();
        b.transaction_traces = vec![tx(token_call(l, &owner, 1, 0))];
        assert_eq!(project(&b, std::slice::from_ref(l)).unwrap().balances[0].amount, rpc);
    }
}
#[test]
fn constant_fallback_preserves_uint256_max_without_rewriting_nonzero_words() {
    let mut l = layouts();
    l[0].zero_balance = Some(layout::VerifiedZeroBalance {
        value: [255; 32],
        storage_slot: None,
    });
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 1, 0))];
    assert_eq!(
        project(&b, &l).unwrap().balances[0].amount,
        "115792089237316195423570985008687907853269984665640564039457584007913129639935"
    );
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 0, 1))];
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "1");
}
#[test]
fn newly_discovered_zero_holders_match_captured_rpc_with_twenty_token_fixture() {
    let layouts = layout::parse(include_str!("../tests/fixtures/bsc-beacon-layouts.json")).unwrap();
    assert_eq!(layouts.len(), 20);
    let cases: Vec<serde_json::Value> = serde_json::from_str(include_str!("../tests/fixtures/bsc-extra-zero-cases.json")).unwrap();
    assert_eq!(cases.len(), 3);
    for row in cases {
        let contract = hex_bytes(row["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        assert_eq!(row["storage"], "0");
        assert_ne!(row["storage"], row["rpc"]);
        let mut b = block();
        b.transaction_traces = vec![tx(token_call(l, &hex_bytes(row["address"].as_str().unwrap()).unwrap(), 1, 0))];
        assert_eq!(project(&b, std::slice::from_ref(l)).unwrap().balances[0].amount, row["rpc"].as_str().unwrap());
    }
}
#[test]
fn fallback_dependency_changes_fail_even_if_restored_or_holder_silent() {
    let mut l = layouts();
    l[0].zero_balance = Some(layout::VerifiedZeroBalance {
        value: slot(8),
        storage_slot: Some(slot(4)),
    });
    let mut b = block();
    let mut c = eth::Call {
        address: l[0].contract.clone(),
        ..Default::default()
    };
    for (ordinal, old, new) in [(10, 8, 7), (20, 7, 8)] {
        c.storage_changes.push(eth::StorageChange {
            address: l[0].contract.clone(),
            key: slot(4).to_vec(),
            old_value: vec![old],
            new_value: vec![new],
            ordinal,
        });
    }
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).unwrap_err().to_string().contains("zero-balance dependency changed"));
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn output_excludes_null_holders_like_the_rpc_reference() {
    let mut b = block();
    let l = layouts();
    b.transaction_traces = vec![tx(token_call(&l[0], &[0; 20], 1, 9))];
    assert_eq!(changes(&b, &l).unwrap().len(), 1);
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn fallback_dependencies_cannot_be_ignored_or_confused_with_other_roles() {
    let original: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-reviewed-layouts.json")).unwrap();
    for field in ["balance_slot", "code_hash"] {
        let mut invalid = original.clone();
        invalid[2]["zero_balance"] = json!({"value":"0x01","storage_slot":invalid[2][field]});
        assert!(layout::parse(&invalid.to_string()).is_err());
    }
    for dep in [
        original[2]["balance_slot"].clone(),
        original[2]["proxy"]["implementation_slot"].clone(),
        original[2]["other_slots"][0].clone(),
        original[2]["other_mapping_slots"][0].clone(),
    ] {
        let mut invalid = original.clone();
        invalid[2]["zero_balance"] = json!({"value":format!("0x{}",hex::encode(slot(8))),"storage_slot":dep});
        assert!(layout::parse(&invalid.to_string()).is_err());
    }
}
#[test]
fn incomplete_or_ambiguous_input_fails() {
    let l = layouts();
    let mut b = block();
    b.detail_level = 1;
    assert!(project(&b, &l).is_err());
    b = block();
    b.ver = 99;
    assert!(project(&b, &l).is_err());
    b = block();
    let mut c = token_call(&l[0], &[6; 20], 0, 1);
    c.storage_changes.push(c.storage_changes[0].clone());
    b.transaction_traces = vec![tx(c)];
    assert!(project(&b, &l).is_err());
    b.transaction_traces[0].calls[0].storage_changes[1].ordinal = 20;
    b.transaction_traces[0].calls[0].storage_changes[1].old_value = vec![3];
    assert!(project(&b, &l).is_err());
}
#[test]
fn no_configured_tokens_or_no_changes_emits_empty_shared_events() {
    let l = layouts();
    let mut b = block();
    assert!(project(&b, &l).unwrap().encode_to_vec().is_empty());
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 1, 2))];
    assert!(project(&b, &[]).unwrap().balances.is_empty());
    assert!(project(&b, &l[1..]).unwrap().balances.is_empty());
}
#[test]
fn configured_code_changes_fail_even_when_returning_to_pinned_code() {
    let l = layouts();
    let mut b = block();
    b.code_changes = vec![eth::CodeChange {
        address: l[0].contract.clone(),
        new_hash: l[0].code_hash.to_vec(),
        ordinal: 5,
        ..Default::default()
    }];
    assert!(project(&b, &l).is_err());
    b.code_changes[0].address = vec![0x77; 20];
    assert!(project(&b, &l).is_ok());
}

fn proxy_layouts() -> Vec<VerifiedLayout> {
    let mut l = layouts();
    l[0].proxy = Some(layout::VerifiedProxy {
        implementation_slot: slot(99),
        implementation: vec![0xcc; 20],
        code_hash: [0xdd; 32],
    });
    l
}
fn beacon_layouts() -> Vec<VerifiedLayout> {
    let mut l = layouts();
    l[0].beacon_proxy = Some(layout::VerifiedBeaconProxy {
        beacon_slot: slot(99),
        beacon: vec![0xcc; 20],
        beacon_code_hash: [0xdd; 32],
        implementation_slot: slot(1),
        implementation: vec![0xee; 20],
        implementation_code_hash: [0xff; 32],
    });
    l
}
#[test]
fn beacon_proxy_projects_holder_writes_and_guards_external_upgrade_back() {
    let l = beacon_layouts();
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 1, 2))];
    assert_eq!(project(&b, &l).unwrap().balances[0].amount, "2");
    // The beacon is not a configured token. Its write still invalidates the token.
    let mut c = eth::Call {
        address: vec![0xcc; 20],
        ..Default::default()
    };
    for (ordinal, old, new) in [(20, 0xee, 0xab), (30, 0xab, 0xee)] {
        c.storage_changes.push(eth::StorageChange {
            address: vec![0xcc; 20],
            key: slot(1).to_vec(),
            old_value: vec![old; 20],
            new_value: vec![new; 20],
            ordinal,
        });
    }
    b.transaction_traces.push(tx(c));
    assert!(project(&b, &l).unwrap_err().to_string().contains("beacon implementation changed"));
    b.transaction_traces.remove(0);
    assert!(project(&b, &l).is_err(), "holder-silent beacon upgrade must fail");
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn beacon_pointer_and_dependency_code_changes_require_requalification() {
    let l = beacon_layouts();
    let mut b = block();
    for address in [vec![0xcc; 20], vec![0xee; 20]] {
        b.code_changes = vec![eth::CodeChange {
            address,
            new_hash: vec![1; 32],
            ordinal: 10,
            ..Default::default()
        }];
        assert!(project(&b, &l).is_err());
    }
    b.code_changes.clear();
    let mut call = token_call(&l[0], &[6; 20], 1, 2);
    call.storage_changes[0].key = slot(99).to_vec();
    b.transaction_traces = vec![tx(call)];
    assert!(project(&b, &l).unwrap_err().to_string().contains("proxy beacon changed"));
}
#[test]
fn beacon_configuration_rejects_ambiguous_dependency_roles() {
    let base = json!([{"contract":format!("0x{}","aa".repeat(20)),"balance_slot":format!("0x{}",hex::encode(slot(7))),"code_hash":format!("0x{}","11".repeat(32)),
        "beacon_proxy":{"beacon_slot":format!("0x{}",hex::encode(slot(99))),"beacon":format!("0x{}","cc".repeat(20)),"beacon_code_hash":format!("0x{}","dd".repeat(32)),
        "implementation_slot":format!("0x{}",hex::encode(slot(1))),"implementation":format!("0x{}","ee".repeat(20)),"implementation_code_hash":format!("0x{}","ff".repeat(32))}}]);
    assert!(layout::parse(&base.to_string()).is_ok());
    for field in ["other_slots", "other_mapping_slots"] {
        let mut bad = base.clone();
        bad[0][field] = json!([bad[0]["beacon_proxy"]["beacon_slot"]]);
        assert!(layout::parse(&bad.to_string()).is_err());
    }
    let mut bad = base.clone();
    bad[0]["proxy"] = json!({"implementation_slot":format!("0x{}",hex::encode(slot(98))),"implementation":format!("0x{}","bb".repeat(20)),"code_hash":format!("0x{}","ff".repeat(32))});
    assert!(layout::parse(&bad.to_string()).is_err());
    let mut bad = base;
    bad[0]["zero_balance"] = json!({"value":format!("0x{}",hex::encode(slot(8))),"storage_slot":bad[0]["beacon_proxy"]["beacon_slot"]});
    assert!(layout::parse(&bad.to_string()).is_err());
}
#[test]
fn pinned_proxy_emits_balances_for_proxy_address() {
    let l = proxy_layouts();
    let mut b = block();
    b.transaction_traces = vec![tx(token_call(&l[0], &[6; 20], 1, 2))];
    let events = project(&b, &l).unwrap();
    assert_eq!(events.balances[0].contract, Some(l[0].contract.clone()));
    assert_eq!(events.balances[0].amount, "2");
}
#[test]
fn proxy_upgrade_and_upgrade_back_require_requalification() {
    let l = proxy_layouts();
    let mut call = token_call(&l[0], &[6; 20], 1, 2);
    for (ordinal, old, new) in [(20, 0xcc, 0xee), (30, 0xee, 0xcc)] {
        call.storage_changes.push(eth::StorageChange {
            address: l[0].contract.clone(),
            key: slot(99).to_vec(),
            old_value: vec![old; 20],
            new_value: vec![new; 20],
            ordinal,
        });
    }
    let mut b = block();
    b.transaction_traces = vec![tx(call)];
    assert!(project(&b, &l).unwrap_err().to_string().contains("proxy implementation slot changed"));
    b.transaction_traces[0].calls[0].state_reverted = true;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
#[test]
fn implementation_code_changes_are_rejected_without_proxy_code_change() {
    let l = proxy_layouts();
    let mut b = block();
    b.code_changes = vec![eth::CodeChange {
        address: l[0].proxy.as_ref().unwrap().implementation.clone(),
        new_hash: vec![0xdd; 32],
        ordinal: 5,
        ..Default::default()
    }];
    assert!(project(&b, &l).unwrap_err().to_string().contains("implementation code changed"));
}
#[test]
fn proxy_layout_cannot_ignore_upgrade_slot_or_pin_zero_self_implementation() {
    let value: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-reviewed-layouts.json")).unwrap();
    for field in ["other_slots", "other_mapping_slots"] {
        let mut bad = value.clone();
        bad[2][field] = json!([bad[2]["proxy"]["implementation_slot"]]);
        assert!(layout::parse(&bad.to_string()).is_err());
    }
    for implementation in [format!("0x{}", "00".repeat(20)), value[2]["contract"].as_str().unwrap().into()] {
        let mut bad = value.clone();
        bad[2]["proxy"]["implementation"] = json!(implementation);
        assert!(layout::parse(&bad.to_string()).is_err());
    }
}
#[test]
fn layout_parameters_reject_ambiguity_and_malformed_values() {
    assert!(layout::parse("[]").unwrap().is_empty());
    let good = include_str!("../tests/fixtures/verified-layouts.json");
    let mut v: serde_json::Value = serde_json::from_str(good).unwrap();
    let duplicate = v[0].clone();
    v.as_array_mut().unwrap().push(duplicate);
    assert!(layout::parse(&v.to_string()).is_err());
    for field in ["contract", "balance_slot", "code_hash"] {
        let mut v: serde_json::Value = serde_json::from_str(good).unwrap();
        v[0][field] = json!("0x01");
        assert!(layout::parse(&v.to_string()).is_err());
    }
    let mut v: serde_json::Value = serde_json::from_str(good).unwrap();
    v[0]["other_mapping_slots"] = json!([v[0]["balance_slot"]]);
    assert!(layout::parse(&v.to_string()).is_err());
}

#[test]
fn package_has_exactly_one_map_and_only_the_shared_protobuf() {
    let manifest = include_str!("../substreams.yaml");
    let modules = manifest.lines().filter(|line| line.starts_with("  - name:")).collect::<Vec<_>>();
    assert_eq!(modules, vec!["  - name: map_events"]);
    assert!(manifest.contains("files: [balances.proto]"));
    assert!(manifest.contains("type: proto:evm.balances.v1.Events"));
}
#[test]
fn captured_failed_authorizations_are_processed_without_rpc() {
    let l = layouts();
    for bytes in [
        include_bytes!("../tests/fixtures/bsc-121114122-failed-setcode.pb").as_slice(),
        include_bytes!("../tests/fixtures/bsc-121114153-failed-setcode.pb").as_slice(),
    ] {
        let mut b = block();
        b.transaction_traces = vec![eth::TransactionTrace::decode(bytes).unwrap()];
        assert!(project(&b, &l).is_ok());
    }
}
#[test]
fn captured_block_matches_all_18_recorded_erc20_balances_with_external_layout() {
    let b = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122260950.pb").as_slice()).unwrap();
    let l = layout::parse(include_str!("../tests/fixtures/verified-layouts.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!("../tests/fixtures/bsc-122260950.json")).unwrap();
    let out = project(&b, &l).unwrap();
    assert_eq!(out.balances.len(), 18);
    for row in expected["rpc_checks"].as_array().unwrap().iter().filter(|r| r["contract"] != "") {
        let address = hex_bytes(row["address"].as_str().unwrap()).unwrap();
        let contract = hex_bytes(row["contract"].as_str().unwrap()).unwrap();
        let actual = out
            .balances
            .iter()
            .find(|v| v.address == address && v.contract.as_ref() == Some(&contract))
            .unwrap();
        assert_eq!(actual.amount, row["balance"].as_str().unwrap());
    }
    assert_eq!(
        hash(&hex_bytes(include_str!("../tests/fixtures/wbnb-runtime.hex").trim()).unwrap()),
        l[0].code_hash
    );
}
