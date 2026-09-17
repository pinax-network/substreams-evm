use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/pending-direct/cases.json")).unwrap()
}
fn captured(case: &Value) -> eth::Block {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pending-direct")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    block
}

#[test]
fn nine_reviewed_direct_getters_match_captured_historical_rpc() {
    let layouts = layout::parse(include_str!("../tests/fixtures/pending-direct/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 9);
    assert_eq!(
        layout::parse(include_str!("../tests/fixtures/bsc-pending-direct-layouts.json")).unwrap().len(),
        197
    );
    let cases = cases();
    assert_eq!(cases.len(), 9);
    let mut tokens = BTreeSet::new();
    let mut count = 0;
    for case in &cases {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == token).unwrap();
        let events = project(&captured(case), std::slice::from_ref(l)).unwrap();
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (hex_bytes(r["address"].as_str().unwrap()).unwrap(), r["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&token)));
        count += expected.len();
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        tokens.insert(token);
    }
    assert_eq!(tokens.len(), 9);
    assert_eq!(count, 33);
}

#[test]
fn captured_bookkeeping_requires_its_mapping_and_nested_record_width() {
    let layouts = layout::parse(include_str!("../tests/fixtures/pending-direct/layouts.json")).unwrap();
    let mut checked = 0;
    for case in cases().iter().filter(|c| c["bookkeeping_regression"] == true) {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == token).unwrap();
        let block = captured(case);
        if case["rank"] == 167 {
            let mut unsupported = l.clone();
            assert!(unsupported.other_mapping_slots.remove(&word(&[30]).unwrap()));
            assert!(project(&block, &[unsupported]).unwrap_err().to_string().contains("unresolved storage"));
        } else {
            // The NO capture changes offsets 0, 2 and 3; YES changes offset 2.
            let last_required_offset = if case["rank"] == 196 { 3 } else { 2 };
            for width in 1..=last_required_offset {
                let mut unsupported = l.clone();
                assert_eq!(unsupported.other_mapping_words.insert(word(&[16]).unwrap(), width), Some(4));
                assert!(project(&block, &[unsupported]).unwrap_err().to_string().contains("unresolved storage"));
            }
        }
        assert!(!project(&block, std::slice::from_ref(l)).unwrap().balances.is_empty());
        checked += 1;
    }
    assert_eq!(checked, 3);
}
