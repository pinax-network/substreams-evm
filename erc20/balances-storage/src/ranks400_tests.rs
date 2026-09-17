use super::*;
use prost::Message;
use serde_json::Value;

fn check_captured_rpc(folder: &str, profiles: usize, rpc_balances: usize) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(folder);
    let layouts = layout::parse(&std::fs::read_to_string(root.join("layouts.json")).unwrap()).unwrap();
    assert_eq!(layouts.len(), profiles);
    let cases: Vec<Value> = serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
    assert_eq!(cases.len(), profiles);
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let layout = layouts.iter().find(|l| l.contract == token).unwrap();
        let block = eth::Block::decode(std::fs::read(root.join(case["fixture"].as_str().unwrap())).unwrap().as_slice()).unwrap();
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
    assert_eq!(tokens.len(), profiles);
    assert_eq!(checked, rpc_balances);
}

#[test]
fn direct_candidates_with_roles_fixed_arrays_and_voting_match_rpc() {
    check_captured_rpc("direct400", 16, 41);
}

#[test]
fn exact_proxy_family_candidates_match_independent_rpc() {
    check_captured_rpc("family400", 11, 37);
}
