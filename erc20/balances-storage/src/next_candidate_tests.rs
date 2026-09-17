use super::*;
use prost::Message;
use serde_json::Value;

#[test]
fn twenty_six_more_candidates_match_captured_historical_rpc() {
    let layouts = layout::parse(include_str!("../tests/fixtures/bsc-next-candidates-layouts.json")).unwrap();
    assert_eq!(layouts.len(), 76);
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/next-candidates/cases.json")).unwrap();
    assert_eq!(cases.len(), 26);
    let mut contracts = BTreeSet::new();
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        assert!(contracts.insert(contract.clone()));
        let layout = layouts.iter().find(|l| l.contract == contract).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/next-candidates")
            .join(case["fixture"].as_str().unwrap());
        let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let events = project(&block, std::slice::from_ref(layout)).unwrap();
        assert!(!events.balances.is_empty());
        assert!(events.balances.iter().all(|b| b.contract.as_ref() == Some(&contract)));
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (hex_bytes(row["address"].as_str().unwrap()).unwrap(), row["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(events.balances.len(), expected.len(), "duplicate or missing output row");
        let actual = events.balances.into_iter().map(|b| (b.address, b.amount)).collect::<BTreeMap<_, _>>();
        assert_eq!(actual, expected, "rank {} contract {}", case["rank"], case["contract"]);
    }
}
