use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/remaining-ranked/cases.json")).unwrap()
}
fn block(case: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/remaining-ranked")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    block
}
#[test]
fn eight_remaining_ranked_getters_match_captured_rpc_balances() {
    let layouts = layout::parse(include_str!("../tests/fixtures/remaining-ranked/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 8);
    let cases = cases();
    assert_eq!(cases.len(), 10);
    let mut tokens = BTreeSet::new();
    for case in cases {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == token).unwrap();
        let b = block(&case);
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        let events = project(&b, std::slice::from_ref(l)).unwrap();
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&token)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        tokens.insert(token);
    }
    assert_eq!(tokens.len(), 8);
}
#[test]
fn recorded_supply_updates_require_the_explicit_reviewed_nonbalance_field() {
    let layouts = layout::parse(include_str!("../tests/fixtures/remaining-ranked/layouts.json")).unwrap();
    let l = layouts
        .iter()
        .find(|l| hex::encode(&l.contract) == "5ce033b2bfca3af30b3e8c8457deaf776a8b695a")
        .unwrap();
    let mut original = l.clone();
    original.other_slots.clear();
    let mut checked = 0;
    for case in cases().iter().filter(|c| [122288448, 122288504].contains(&c["block"].as_u64().unwrap())) {
        let b = block(case);
        let error = project(&b, std::slice::from_ref(&original)).unwrap_err().to_string();
        assert!(error.contains("unresolved storage") && error.contains(&format!("0x{:064x}", 2)));
        assert!(!project(&b, std::slice::from_ref(l)).unwrap().balances.is_empty());
        checked += 1;
    }
    assert_eq!(checked, 2);
}
