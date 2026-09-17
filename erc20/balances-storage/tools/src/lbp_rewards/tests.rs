use super::fixture::*;
use crate::data::{items, text, uint};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

const MANIFEST: &str = include_str!("../../../tests/fixtures/lbp-rewards/manifest.json");
const CHECKPOINT: &str = include_str!("../../../tests/fixtures/lbp-rewards/checkpoint.json");
const CHECKS: &str = include_str!("../../../tests/fixtures/lbp-rewards/checks.json");
const REPLAY: &str = include_str!("../../../tests/fixtures/lbp-rewards/replay.json");
const CONTROLS: &str = include_str!("../../../tests/fixtures/lbp-rewards/controls.json");

fn parse(s: &str) -> Value {
    serde_json::from_str(s).unwrap()
}

fn historical_replay(freeze_updates: bool, freeze_time: bool) -> (usize, usize) {
    let manifest = parse(MANIFEST);
    let replay = parse(REPLAY);
    let checkpoint_rows = parse(CHECKPOINT);
    let expectations = parse(CHECKS);
    let exemptions = exemptions(&manifest).unwrap();
    let mut state = checkpoint(&checkpoint_rows).unwrap();
    assert_eq!(state.len(), 171);
    let initial_time = replay["parent"]["timestamp"].as_u64().unwrap();
    let mut previous = text(&replay["parent"]["hash"]).unwrap().to_owned();
    let mut height = replay["parent"]["number"].as_u64().unwrap();
    let checks = expectations.as_array().unwrap().iter().fold(BTreeMap::<u64, Vec<&Value>>::new(), |mut m, v| {
        m.entry(v["block"].as_u64().unwrap()).or_default().push(v);
        m
    });
    let mut tested = 0;
    let mut mismatches = 0;
    let mut verify = |state: &State, block: u64, hash: &str, timestamp: u64| {
        if let Some(rows) = checks.get(&block) {
            for row in rows {
                assert_eq!(row["hash"], hash);
                let (g, h) = decode(state, text(&row["holder"]).unwrap(), &exemptions).unwrap();
                let now = if freeze_time { initial_time } else { timestamp };
                let balance = h.balance(&g, now).unwrap();
                let pending = h.pending(&g, now).unwrap();
                // Only the independent RPC values are expectations. Recorded
                // model results never seed, repair or otherwise enter state.
                mismatches += usize::from(balance != uint(&row["rpc"]).unwrap() || pending != uint(&row["rpc_pending"]).unwrap());
                tested += 1;
            }
        }
    };
    verify(&state, height, &previous, initial_time);
    let mut updates = 0;
    let mut raw_updates = 0;
    for block in items(&replay, "blocks").unwrap() {
        assert_eq!(block["number"], height + 1);
        assert_eq!(block["parent_hash"], previous);
        for update in items(block, "updates").unwrap() {
            updates += 1;
            raw_updates += usize::from(update["contract"] == TOKEN);
        }
        if !freeze_updates {
            apply(&mut state, &block["updates"]).unwrap();
        }
        height = block["number"].as_u64().unwrap();
        previous = text(&block["hash"]).unwrap().into();
        verify(&state, height, &previous, block["timestamp"].as_u64().unwrap());
    }
    assert_eq!(height, 122289029);
    assert_eq!(updates, 110);
    assert_eq!(raw_updates, 0);
    assert_eq!(tested, 594);
    (tested, mismatches)
}

#[test]
fn captured_reward_replay_matches_all_594_independent_rpc_samples() {
    let manifest = parse(MANIFEST);
    for (field, bytes) in [
        ("checkpoint_sha256", CHECKPOINT),
        ("checks_sha256", CHECKS),
        ("replay_sha256", REPLAY),
        ("controls_sha256", CONTROLS),
    ] {
        assert_eq!(manifest[field], hex::encode(Sha256::digest(bytes.as_bytes())), "fixture digest {field}");
    }
    assert_eq!(manifest["qualified"], false);
    assert_eq!(manifest["production_changed"], false);
    assert_eq!(historical_replay(false, false), (594, 0));
}

#[test]
fn captured_reward_replay_requires_both_global_updates_and_block_time() {
    assert!(historical_replay(true, false).1 > 0, "dropping reward-state writes must be detectable");
    assert!(historical_replay(false, true).1 > 0, "freezing the preview clock must be detectable");
}

#[test]
fn simulated_states_match_independent_rpc_values_and_arithmetic_panics() {
    let manifest = parse(MANIFEST);
    let exemptions = exemptions(&manifest).unwrap();
    let controls = parse(CONTROLS);
    let rows = controls.as_array().unwrap();
    assert_eq!(rows.len(), manifest["controls"].as_u64().unwrap() as usize);
    assert_eq!(rows.len(), 34);
    let mut panics = 0;
    for case in rows {
        let state = checkpoint(&case["checkpoint"]).unwrap();
        let (g, h) = decode(&state, text(&case["holder"]).unwrap(), &exemptions).unwrap();
        let now = case["timestamp"].as_u64().unwrap();
        for (field, result) in [("balance", h.balance(&g, now)), ("pending", h.pending(&g, now))] {
            if case[field]["panic"] == "0x11" {
                assert!(result.unwrap_err().to_string().contains("overflow"), "{} {field}", case["name"]);
                panics += 1;
            } else {
                assert_eq!(result.unwrap(), uint(&case[field]["value"]).unwrap(), "{} {field}", case["name"]);
            }
        }
    }
    assert_eq!(panics, 5);
}

#[test]
fn reward_checkpoint_and_journal_reject_missing_or_inconsistent_state() {
    let checkpoint_rows = parse(CHECKPOINT);
    let replay = parse(REPLAY);
    let mut state = checkpoint(&checkpoint_rows).unwrap();
    let first = items(&replay, "blocks")
        .unwrap()
        .iter()
        .find(|b| !items(b, "updates").unwrap().is_empty())
        .unwrap();
    let mut wrong = first["updates"].clone();
    wrong[0]["old"] = Value::String((uint(&wrong[0]["old"]).unwrap() + 1).to_string());
    assert!(apply(&mut state, &wrong).unwrap_err().to_string().contains("discontinuity"));
    let mut duplicate = first["updates"].as_array().unwrap().clone();
    duplicate.insert(1, duplicate[0].clone());
    assert!(apply(&mut state, &Value::Array(duplicate)).unwrap_err().to_string().contains("ordinal"));
    let mut missing = checkpoint(&checkpoint_rows).unwrap();
    missing.remove(&(DEP.into(), slot(7)));
    let address = text(&replay["holders"][0]).unwrap();
    assert!(decode(&missing, address, &[]).unwrap_err().to_string().contains("uninitialized"));
}
