use super::*;
use prost::Message;
use serde_json::Value;

fn fixture() -> (eth::Block, layout::VerifiedLayout, Value) {
    let mut layouts = layout::parse(include_str!("../tests/fixtures/swkey/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 1);
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/swkey/cases.json")).unwrap();
    assert_eq!(cases.len(), 1);
    let case = cases.into_iter().next().unwrap();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/swkey")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    (block, layouts.remove(0), case)
}

#[test]
fn captured_swkey_shares_require_division_to_match_independent_rpc() {
    let (block, layout, case) = fixture();
    let expected = case["balances"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| (hex_bytes(row["address"].as_str().unwrap()).unwrap(), row["rpc"].as_str().unwrap().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let events = project(&block, std::slice::from_ref(&layout)).unwrap();
    assert_eq!(events.balances.len(), expected.len());
    assert!(events.balances.iter().all(|row| row.contract.as_ref() == Some(&layout.contract)));
    assert_eq!(
        events.balances.into_iter().map(|row| (row.address, row.amount)).collect::<BTreeMap<_, _>>(),
        expected
    );

    let mut raw_layout = layout;
    raw_layout.balance_divisor = None;
    let raw = project(&block, &[raw_layout]).unwrap();
    assert!(raw.balances.iter().any(|row| row.amount != expected[&row.address]));
}

#[test]
fn captured_swkey_layout_rejects_a_persisted_conversion_factor_change() {
    let (mut block, layout, _) = fixture();
    let rule = layout.balance_divisor.as_ref().unwrap();
    let change = block
        .transaction_traces
        .iter_mut()
        .filter(|tx| tx.status == 1)
        .flat_map(|tx| &mut tx.calls)
        .filter(|call| !call.state_reverted)
        .flat_map(|call| &mut call.storage_changes)
        .find(|change| change.address == layout.contract && change.old_value != change.new_value)
        .unwrap();
    change.key = rule.storage_slot.to_vec();
    change.old_value = rule.value.to_vec();
    change.new_value = vec![1];
    let error = project(&block, &[layout]).unwrap_err().to_string();
    assert!(
        error.contains("balance divisor changed; requalify layout and rebuild retained holder balances"),
        "{error}"
    );
}
