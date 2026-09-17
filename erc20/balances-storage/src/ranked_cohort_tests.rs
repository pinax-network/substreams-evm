use super::*;
use prost::Message;
use serde_json::Value;

#[test]
fn thirty_three_ranked_candidates_match_captured_rpc_balances() {
    let layouts = layout::parse(include_str!("../tests/fixtures/bsc-ranks101-150-layouts.json")).unwrap();
    assert_eq!(layouts.len(), 133);
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/ranks101-150/cases.json")).unwrap();
    assert_eq!(cases.len(), 33);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ranks101-150");
    let mut contracts = BTreeSet::new();
    for case in cases {
        let contract = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        assert!(contracts.insert(contract.clone()));
        let l = layouts.iter().find(|l| l.contract == contract).unwrap();
        let b = eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap();
        assert_eq!(b.number, case["block"].as_u64().unwrap());
        assert_eq!(b.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        let events = project(&b, std::slice::from_ref(l)).unwrap();
        assert!(!events.balances.is_empty());
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&contract)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        if l.balance_divisor.is_some() {
            let mut wrong = l.clone();
            wrong.balance_divisor = None;
            let unscaled = project(&b, &[wrong]).unwrap();
            assert!(unscaled.balances.iter().any(|r| r.amount != expected[&r.address]));
        }
    }
}
