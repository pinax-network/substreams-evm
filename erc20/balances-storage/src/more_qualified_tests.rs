use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/more-qualified/cases.json")).unwrap()
}
fn block(case: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/more-qualified")
        .join(case["fixture"].as_str().unwrap());
    let b = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(b.number, case["block"].as_u64().unwrap());
    assert_eq!(b.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    b
}

#[test]
fn ten_more_proxy_and_array_profiles_match_captured_rpc() {
    let layouts = layout::parse(include_str!("../tests/fixtures/more-qualified/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 10);
    assert_eq!(
        layout::parse(include_str!("../tests/fixtures/bsc-more-qualified-layouts.json")).unwrap().len(),
        188
    );
    let cases = cases();
    assert_eq!(cases.len(), 11);
    let mut tokens = BTreeSet::new();
    for case in &cases {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == token).unwrap();
        let events = project(&block(case), std::slice::from_ref(l)).unwrap();
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&token)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        tokens.insert(token);
    }
    assert_eq!(tokens.len(), 10);
}

#[test]
fn captured_fee_and_shareholder_arrays_require_explicit_append_qualification() {
    let layouts = layout::parse(include_str!("../tests/fixtures/more-qualified/layouts.json")).unwrap();
    let mut checked = 0;
    for case in cases().iter().filter(|c| [122288022, 122288619].contains(&c["block"].as_u64().unwrap())) {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == token).unwrap();
        assert_eq!(l.address_lists.len(), 1);
        let b = block(case);
        let mut unqualified = l.clone();
        unqualified.address_lists.clear();
        assert!(project(&b, &[unqualified]).unwrap_err().to_string().contains("unresolved storage"));
        assert!(!project(&b, std::slice::from_ref(l)).unwrap().balances.is_empty());
        checked += 1;
    }
    assert_eq!(checked, 2);
}
