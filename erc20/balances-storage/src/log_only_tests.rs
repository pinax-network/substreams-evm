use super::*;
use prost::Message;
use serde_json::Value;

fn evidence() -> Value {
    serde_json::from_str(include_str!("../docs/evidence/log-only-creation-review.json")).unwrap()
}

#[test]
fn captured_log_batch_does_not_fabricate_balance_updates() {
    let e = evidence();
    let l = layout::parse(&serde_json::json!([e["layout_hypothesis"]]).to_string()).unwrap();
    let b = eth::Block::decode(include_bytes!("../tests/fixtures/log-only/activity.pb").as_slice()).unwrap();
    assert_eq!(b.number, 122288749);
    assert_eq!(b.hash, hex_bytes(e["activity"]["hash"].as_str().unwrap()).unwrap());
    let calls = b
        .transaction_traces
        .iter()
        .flat_map(|tx| &tx.calls)
        .filter(|c| c.address == l[0].contract && !c.state_reverted)
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].storage_changes.is_empty());
    assert_eq!(calls[0].logs.len(), 159);
    let transfer = hash(b"Transfer(address,address,uint256)");
    assert!(calls[0]
        .logs
        .iter()
        .all(|log| log.topics[0] == transfer && BigInt::from_unsigned_bytes_be(&log.data) > 0));
    let checks = e["activity"]["checks"].as_array().unwrap();
    assert_eq!(checks.len(), 298);
    assert!(checks.iter().all(|c| c["rpc"] == "0"));
    assert!(project(&b, &l).unwrap().balances.is_empty());
}

#[test]
fn captured_log_generator_creation_has_no_balance_initialization() {
    let e = evidence();
    let token = hex_bytes(e["contract"].as_str().unwrap()).unwrap();
    let b = eth::Block::decode(include_bytes!("../tests/fixtures/log-only/creation.pb").as_slice()).unwrap();
    assert_eq!(b.number, 121118665);
    assert_eq!(b.hash, hex_bytes(e["creation"]["hash"].as_str().unwrap()).unwrap());
    assert_eq!(
        b.header.as_ref().unwrap().parent_hash,
        hex_bytes(e["before"]["hash"].as_str().unwrap()).unwrap()
    );
    assert_eq!(e["before"]["code"], "0x");
    assert_eq!(e["before"]["nonce"], "0x0");
    let calls = b.transaction_traces.iter().flat_map(|tx| &tx.calls).collect::<Vec<_>>();
    assert!(calls.iter().flat_map(|c| &c.storage_changes).all(|s| s.address != token));
    let creation = calls.iter().find(|c| c.address == token && c.call_type == 5 && !c.state_reverted).unwrap();
    let runtime = hex_bytes(e["runtime"].as_str().unwrap()).unwrap();
    assert_eq!(runtime.len(), 2264);
    assert_eq!(creation.code_changes.len(), 1);
    assert!(creation.code_changes[0].old_code.is_empty());
    assert_eq!(creation.code_changes[0].new_code, runtime);
    // This constructor copies and returns the runtime without initializing storage.
    assert_eq!(
        &creation.input[..28],
        &hex_bytes("6080604052348015600e575f5ffd5b506108d88061001c5f395ff3fe").unwrap()
    );
    assert_eq!(&creation.input[28..], runtime.as_slice());
    // Inspect opcode boundaries, excluding compiler metadata and PUSH payloads.
    let mut pc = 0;
    let mut writes = vec![];
    while pc < 2210 {
        let op = runtime[pc];
        if op == 0x55 {
            writes.push(pc);
        }
        assert!(![0xf0, 0xf1, 0xf2, 0xf4, 0xf5, 0xfa, 0xff].contains(&op));
        pc += 1 + if (0x60..=0x7f).contains(&op) { usize::from(op - 0x5f) } else { 0 };
    }
    assert_eq!(writes, [856]); // Reviewed nested allowance mapping, not mapping 0.
}
