use super::*;
use prost::Message;
use serde_json::Value;

#[test]
fn ranks301_to350_reviewed_profiles_match_captured_independent_rpc() {
    let layouts = layout::parse(include_str!("../tests/fixtures/ranks301-350/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 36);
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/ranks301-350/cases.json")).unwrap();
    assert_eq!(cases.len(), 36);
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == token).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/ranks301-350")
            .join(case["fixture"].as_str().unwrap());
        let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
        assert_eq!(block.number, case["block"].as_u64().unwrap());
        assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        checked += expected.len();
        let events = project(&block, std::slice::from_ref(layout)).unwrap();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&token)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        assert!(tokens.insert(token));
    }
    assert_eq!(tokens.len(), 36);
    assert_eq!(checked, 136);
}
