use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/vsd/cases.json")).unwrap()
}

fn captured(case: &Value) -> eth::Block {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/vsd")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(file).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    block
}

#[test]
fn captured_vsd_proxy_transfers_match_historical_rpc() {
    let layouts = layout::parse(include_str!("../tests/fixtures/vsd/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 1);
    assert!(layouts[0].proxy.is_some());
    assert_eq!(layout::parse(include_str!("../tests/fixtures/bsc-vsd-layouts.json")).unwrap().len(), 198);
    let cases = cases();
    assert_eq!(cases.len(), 2);
    let mut count = 0;
    for case in cases {
        let events = project(&captured(&case), &layouts).unwrap();
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&layouts[0].contract)));
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        count += expected.len();
        assert_eq!(events.balances.len(), expected.len());
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
    }
    assert_eq!(count, 6);
}

#[test]
fn captured_vsd_allowance_decrease_requires_the_reviewed_nested_root() {
    let layouts = layout::parse(include_str!("../tests/fixtures/vsd/layouts.json")).unwrap();
    let case = cases().into_iter().find(|c| c["allowance_regression"] == true).unwrap();
    let block = captured(&case);
    let mut missing = layouts[0].clone();
    assert_eq!(missing.other_mapping_slots.len(), 1);
    missing.other_mapping_slots.clear();
    let error = project(&block, &[missing]).unwrap_err().to_string();
    assert!(error.contains("unresolved storage"));
    assert!(error.contains("7104cacca4451611e3aae993596614f957ead68b1cdb5de9d1d667788e6af0a5"));
    assert_eq!(project(&block, &layouts).unwrap().balances.len(), 3);
}
