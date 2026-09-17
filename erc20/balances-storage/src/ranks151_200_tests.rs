use super::*;
use prost::Message;
use serde_json::Value;

#[test]
fn twenty_nine_ranked_profiles_match_captured_rpc_balances() {
    let layouts = layout::parse(include_str!("../tests/fixtures/ranks151-200/layouts.json")).unwrap();
    assert_eq!(layouts.len(), 29);
    let combined = layout::parse(include_str!("../tests/fixtures/bsc-ranks151-200-layouts.json")).unwrap();
    assert_eq!(combined.len(), 178);
    let cases: Vec<Value> = serde_json::from_str(include_str!("../tests/fixtures/ranks151-200/cases.json")).unwrap();
    assert_eq!(cases.len(), 30);
    let mut tokens = BTreeSet::new();
    for case in cases {
        let token = hex_bytes(case["contract"].as_str().unwrap()).unwrap();
        let l = layouts.iter().find(|l| l.contract == token).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/ranks151-200")
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
        let events = project(&block, std::slice::from_ref(l)).unwrap();
        assert!(events.balances.iter().all(|r| r.contract.as_ref() == Some(&token)));
        assert_eq!(events.balances.into_iter().map(|r| (r.address, r.amount)).collect::<BTreeMap<_, _>>(), expected);
        tokens.insert(token);
    }
    assert_eq!(tokens.len(), 29);
}

#[test]
fn arc_clone_requires_its_own_verified_fallback_for_initialized_holders() {
    let layouts = layout::parse(include_str!("../tests/fixtures/ranks151-200/layouts.json")).unwrap();
    let l = layouts
        .iter()
        .find(|l| hex::encode(&l.contract) == "32b133ca38c9b410a053f2bcfeea83831c3bcfe0")
        .unwrap();
    let mut raw_only = l.clone();
    raw_only.zero_balance = None;
    let mut holders = BTreeSet::new();
    let mut fallback_holders = 0;
    for line in include_str!("../tests/fixtures/ranks151-200/arc-checkpoint.jsonl").lines() {
        let row: Value = serde_json::from_str(line).unwrap();
        assert_eq!(row["contract"], format!("0x{}", hex::encode(&l.contract)));
        assert_eq!(row["hash"], "0xdf9d02ee1c9e345bfcdb971e488a20f7f51dc03d3d8e2e7bbc8fc9f0bb0814cf");
        let address = hex_bytes(row["address"].as_str().unwrap()).unwrap();
        assert!(holders.insert(address.clone()));
        let raw = row["storage"].as_str().unwrap();
        assert_eq!(l.project_amount(&address, raw), row["rpc"]);
        assert_eq!(row["projected"], row["rpc"]);
        if raw == "0" {
            assert_eq!(row["rpc"], "1000000000");
            assert_ne!(raw_only.project_amount(&address, raw), row["rpc"]);
            fallback_holders += 1;
        }
    }
    assert_eq!(holders.len(), 63);
    assert_eq!(fallback_holders, 60);
}
