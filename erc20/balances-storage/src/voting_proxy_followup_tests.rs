use super::*;
use prost::Message;
use serde_json::Value;

fn cases() -> Vec<Value> {
    serde_json::from_str(include_str!("../tests/fixtures/voting-proxy-followup/cases.json")).unwrap()
}
fn load(case: &Value) -> (eth::Block, layout::VerifiedLayout) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/voting-proxy-followup")
        .join(case["fixture"].as_str().unwrap());
    let block = eth::Block::decode(std::fs::read(path).unwrap().as_slice()).unwrap();
    assert_eq!(block.number, case["block"].as_u64().unwrap());
    assert_eq!(block.hash, hex_bytes(case["hash"].as_str().unwrap()).unwrap());
    let mut layouts = layout::parse(&serde_json::json!([case["layout"]]).to_string()).unwrap();
    assert_eq!(layouts.len(), 1);
    (block, layouts.remove(0))
}

#[test]
fn six_more_profiles_match_independent_rpc_in_transfers_mints_and_burns() {
    let cases = cases();
    assert_eq!(cases.len(), 9);
    let mut tokens = BTreeSet::new();
    let mut checked = 0;
    for case in cases {
        let (block, layout) = load(&case);
        let expected = case["balances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| (hex_bytes(row["address"].as_str().unwrap()).unwrap(), row["rpc"].as_str().unwrap().to_owned()))
            .collect::<BTreeMap<_, _>>();
        assert!(!expected.is_empty());
        checked += expected.len();
        let events = project(&block, std::slice::from_ref(&layout)).unwrap();
        assert_eq!(events.balances.len(), expected.len());
        assert!(events.balances.iter().all(|row| row.contract.as_ref() == Some(&layout.contract)));
        assert_eq!(
            events.balances.into_iter().map(|row| (row.address, row.amount)).collect::<BTreeMap<_, _>>(),
            expected
        );
        tokens.insert(layout.contract);
    }
    assert_eq!(tokens.len(), 6);
    assert_eq!(checked, 25);
}

#[test]
fn captured_mints_and_burn_registration_require_their_reviewed_storage_rules() {
    let mut checked = 0;
    for case in cases().into_iter().filter(|case| case["required_rule"] != "") {
        let (block, mut layout) = load(&case);
        assert!(!project(&block, std::slice::from_ref(&layout)).unwrap().balances.is_empty());
        layout.voting_checkpoints = None;
        layout.address_lists.clear();
        let error = project(&block, &[layout]).unwrap_err().to_string();
        assert_eq!(error, case["without_special_rule"].as_str().unwrap());
        assert!(error.contains("unresolved storage for configured token"));
        checked += 1;
    }
    assert_eq!(checked, 3);
}

#[test]
fn captured_voting_mints_reject_the_other_clock_type() {
    let mut checked = 0;
    for case in cases().into_iter().filter(|case| case["required_rule"] == "voting_checkpoints") {
        let (block, mut layout) = load(&case);
        let rule = layout.voting_checkpoints.as_mut().unwrap();
        rule.clock = match rule.clock {
            layout::CheckpointClock::BlockNumber => layout::CheckpointClock::Timestamp,
            layout::CheckpointClock::Timestamp => layout::CheckpointClock::BlockNumber,
        };
        let error = project(&block, &[layout]).unwrap_err().to_string();
        assert!(error.contains("checkpoint word has the wrong clock"), "{error}");
        checked += 1;
    }
    assert_eq!(checked, 2);
}
