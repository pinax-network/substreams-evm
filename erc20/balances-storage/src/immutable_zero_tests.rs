use super::*;
use prost::Message;
use serde_json::{json, Value};

fn params() -> Value {
    serde_json::from_str(include_str!("../tests/fixtures/immutable-zero-layout.json")).unwrap()
}
fn layouts() -> Vec<VerifiedLayout> {
    layout::parse(&params().to_string()).unwrap()
}
fn activity() -> eth::Block {
    eth::Block::decode(include_bytes!("../tests/fixtures/log-only/activity.pb").as_slice()).unwrap()
}
fn creation() -> eth::Block {
    eth::Block::decode(include_bytes!("../tests/fixtures/log-only/creation.pb").as_slice()).unwrap()
}
fn mapping_write(l: &VerifiedLayout, old: u8, new: u8) -> eth::Call {
    let mut preimage = word(&[7; 20]).unwrap().to_vec();
    preimage.extend(l.balance_slot);
    let key = hash(&preimage);
    eth::Call {
        address: l.contract.clone(),
        keccak_preimages: [(hex::encode(key), hex::encode(preimage))].into(),
        storage_changes: vec![eth::StorageChange {
            address: l.contract.clone(),
            key: key.to_vec(),
            old_value: vec![old],
            new_value: vec![new],
            ordinal: u64::MAX - 1,
        }],
        ..Default::default()
    }
}
fn append(b: &mut eth::Block, c: eth::Call) {
    b.transaction_traces.push(eth::TransactionTrace {
        index: u32::MAX,
        status: 1,
        calls: vec![c],
        ..Default::default()
    });
}

#[test]
fn immutable_zero_capture_matches_every_independent_rpc_participant() {
    let e: Value = serde_json::from_str(include_str!("../docs/evidence/immutable-zero-qualification.json")).unwrap();
    let expected = e["independent_rpc_checks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let l = layouts();
    let b = activity();
    assert!(changes(&b, &l).unwrap().is_empty());
    let events = project(&b, &l).unwrap();
    assert_eq!(events.balances.len(), 298);
    assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&l[0].contract)));
    assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
    let mut raw = l.clone();
    raw[0].immutable_zero_mapping = false;
    assert!(project(&b, &raw).unwrap().balances.is_empty());
    // An RPC state override can break the proven empty-mapping invariant.
    // Raw-word diagnostics must expose that contradiction, not mask it as zero.
    assert_eq!(l[0].project_amount(&[7; 20], "123"), "123");
}

#[test]
fn immutable_zero_mapping_rejects_writes_noops_restores_and_system_writes() {
    let l = layouts();
    for (old, new) in [(0, 1), (1, 0), (0, 0), (1, 1)] {
        let mut b = activity();
        append(&mut b, mapping_write(&l[0], old, new));
        assert!(project(&b, &l).unwrap_err().to_string().contains("immutable-zero balance mapping was written"));
    }
    let mut b = activity();
    let mut c = mapping_write(&l[0], 0, 1);
    let mut restored = c.storage_changes[0].clone();
    restored.old_value = vec![1];
    restored.new_value = vec![0];
    restored.ordinal += 1;
    c.storage_changes.push(restored);
    append(&mut b, c.clone());
    assert!(project(&b, &l).is_err());
    b.transaction_traces.last_mut().unwrap().calls[0].state_reverted = true;
    assert_eq!(project(&b, &l).unwrap().balances.len(), 298);
    b.transaction_traces.last_mut().unwrap().calls[0].state_reverted = false;
    b.transaction_traces.last_mut().unwrap().status = 2;
    assert_eq!(project(&b, &l).unwrap().balances.len(), 298);
    b.system_calls.push(c);
    assert!(project(&b, &l).is_err());
}

#[test]
fn immutable_zero_birth_and_event_timing_are_enforced() {
    let l = layouts();
    let b = creation();
    assert!(project(&b, &l).unwrap().balances.is_empty());
    let mut forged = b.clone();
    let create = forged
        .transaction_traces
        .iter_mut()
        .flat_map(|tx| &mut tx.calls)
        .find(|c| c.address == l[0].contract && c.call_type == 5)
        .unwrap();
    let mut write = mapping_write(&l[0], 0, 1);
    write.storage_changes[0].ordinal = create.begin_ordinal + 1;
    create.keccak_preimages.extend(write.keccak_preimages);
    create.storage_changes.extend(write.storage_changes);
    assert!(project(&forged, &l).is_err());
    let mut early = activity();
    early.number = l[0].deployment.as_ref().unwrap().block - 1;
    early.header.as_mut().unwrap().number = early.number;
    assert!(project(&early, &l).unwrap_err().to_string().contains("before its deployment"));
    let mut event = activity()
        .transaction_traces
        .iter()
        .flat_map(|tx| &tx.calls)
        .flat_map(|c| &c.logs)
        .find(|log| log.address == l[0].contract)
        .unwrap()
        .clone();
    let mut at_birth = b.clone();
    let create = at_birth
        .transaction_traces
        .iter_mut()
        .flat_map(|tx| &mut tx.calls)
        .find(|c| c.address == l[0].contract && c.call_type == 5)
        .unwrap();
    event.ordinal = create.begin_ordinal;
    create.logs.push(event);
    assert!(project(&at_birth, &l).unwrap_err().to_string().contains("before CREATE"));
    let mut wrong_hash = b;
    wrong_hash.hash[0] ^= 1;
    assert!(project(&wrong_hash, &l).is_err());
}

#[test]
fn immutable_zero_allows_reviewed_allowances_but_rejects_unknown_writes_and_code_changes() {
    let l = layouts();
    let mut b = activity();
    let mut first = word(&[7; 20]).unwrap().to_vec();
    first.extend(word(&[1]).unwrap());
    let mut second = word(&[8; 20]).unwrap().to_vec();
    second.extend(hash(&first));
    let key = hash(&second);
    let call = eth::Call {
        address: l[0].contract.clone(),
        keccak_preimages: [(hex::encode(hash(&first)), hex::encode(first)), (hex::encode(key), hex::encode(second))].into(),
        storage_changes: vec![eth::StorageChange {
            address: l[0].contract.clone(),
            key: key.to_vec(),
            old_value: vec![0],
            new_value: vec![42],
            ordinal: u64::MAX - 1,
        }],
        ..Default::default()
    };
    append(&mut b, call);
    assert_eq!(project(&b, &l).unwrap().balances.len(), 298);
    for (old, new) in [(0, 1), (1, 1)] {
        let mut unknown = b.clone();
        let c = unknown.transaction_traces.last_mut().unwrap().calls.first_mut().unwrap();
        c.storage_changes[0].key = word(&[2]).unwrap().to_vec();
        c.storage_changes[0].old_value = vec![old];
        c.storage_changes[0].new_value = vec![new];
        assert!(project(&unknown, &l).unwrap_err().to_string().contains("unresolved storage"));
    }
    b.code_changes.push(eth::CodeChange {
        address: l[0].contract.clone(),
        old_hash: l[0].code_hash.to_vec(),
        new_hash: vec![42; 32],
        ordinal: u64::MAX - 1,
        ..Default::default()
    });
    assert!(project(&b, &l).unwrap_err().to_string().contains("code changed"));
}

#[test]
fn immutable_zero_requires_an_explicit_direct_deployment_without_other_balance_rules() {
    let base = params();
    for field in ["deployment", "immutable_zero_mapping"] {
        let mut p = base.clone();
        p[0].as_object_mut().unwrap().remove(field);
        if field == "deployment" {
            assert!(layout::parse(&p.to_string()).is_err());
        } else {
            assert!(!layout::parse(&p.to_string()).unwrap()[0].immutable_zero_mapping);
        }
    }
    for (field, value) in [
        ("zero_balance", json!({"value":format!("0x{}", "00".repeat(32))})),
        (
            "proxy",
            json!({"implementation":format!("0x{}", "77".repeat(20)),"implementation_slot":format!("0x{}", "55".repeat(32)),"code_hash":format!("0x{}", "99".repeat(32))}),
        ),
        (
            "address_hash_balance",
            json!({"modulus":format!("0x{}01", "00".repeat(31)),"offset":format!("0x{}", "00".repeat(32)),"multiplier":format!("0x{}01", "00".repeat(31)),"stored_addresses":[]}),
        ),
    ] {
        let mut p = base.clone();
        p[0][field] = value;
        assert!(layout::parse(&p.to_string()).is_err());
    }
}

#[test]
fn immutable_zero_events_preserve_reference_filters_and_reject_malformed_logs() {
    let l = layouts();
    let mut b = activity();
    let count = project(&b, &l).unwrap().balances.len();
    let call = b
        .transaction_traces
        .iter_mut()
        .flat_map(|tx| &mut tx.calls)
        .find(|c| c.address == l[0].contract)
        .unwrap();
    call.logs.push(call.logs[0].clone());
    assert_eq!(project(&b, &l).unwrap().balances.len(), count);
    let call = b
        .transaction_traces
        .iter_mut()
        .flat_map(|tx| &mut tx.calls)
        .find(|c| c.address == l[0].contract)
        .unwrap();
    call.logs[0].data.clear();
    assert!(project(&b, &l).is_err());
    call_mut_revert(&mut b, &l[0].contract);
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
fn call_mut_revert(b: &mut eth::Block, token: &[u8]) {
    b.transaction_traces
        .iter_mut()
        .flat_map(|tx| &mut tx.calls)
        .find(|c| c.address == token)
        .unwrap()
        .state_reverted = true;
}

#[test]
fn immutable_zero_approval_participants_sender_and_contract_match_reference_selection() {
    let l = layouts();
    let token = &l[0].contract;
    let mut b = activity();
    for call in b.transaction_traces.iter_mut().flat_map(|tx| &mut tx.calls) {
        call.logs.retain(|log| log.address != *token);
    }
    let approval = eth::Log {
        address: token.clone(),
        topics: vec![
            hash(b"Approval(address,address,uint256)").to_vec(),
            word(&[6; 20]).unwrap().to_vec(),
            word(&[7; 20]).unwrap().to_vec(),
        ],
        data: word(&[123]).unwrap().to_vec(),
        ordinal: u64::MAX - 2,
        ..Default::default()
    };
    let transfer = eth::Log {
        topics: vec![
            hash(b"Transfer(address,address,uint256)").to_vec(),
            vec![0; 32],
            word(&[6; 20]).unwrap().to_vec(),
        ],
        ordinal: u64::MAX - 1,
        ..approval.clone()
    };
    b.transaction_traces.push(eth::TransactionTrace {
        index: u32::MAX,
        status: 1,
        from: vec![8; 20],
        calls: vec![eth::Call {
            logs: vec![approval.clone(), approval, transfer],
            ..Default::default()
        }],
        ..Default::default()
    });
    let events = project(&b, &l).unwrap();
    assert_eq!(events.balances.len(), 4);
    assert_eq!(
        events.balances.iter().map(|r| r.address.clone()).collect::<BTreeSet<_>>(),
        [token.clone(), vec![6; 20], vec![7; 20], vec![8; 20]].into()
    );
    assert!(events.balances.iter().all(|r| r.amount == "0"));
    b.transaction_traces.last_mut().unwrap().status = 2;
    assert!(project(&b, &l).unwrap().balances.is_empty());
}
