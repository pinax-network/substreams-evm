use super::{fixture::*, *};
use crate::data::*;
use serde_json::Value;

fn captured() -> Vec<Value> {
    [
        include_str!("../../tests/fixtures/reflection/BabyDoge.json"),
        include_str!("../../tests/fixtures/reflection/10SET.json"),
    ]
    .into_iter()
    .map(|s| serde_json::from_str(s).unwrap())
    .collect()
}

#[test]
fn source_bound_historical_reflection_snapshots_match_independent_rpc() {
    let mut checks = 0;
    let mut excluded = 0;
    let mut direct_word_mismatches = 0;
    for token in captured() {
        let layout: Layout = serde_json::from_value(token["layout"].clone()).unwrap();
        let mut previous = 0;
        for snapshot in items(&token, "snapshots").unwrap() {
            let height = number(&snapshot["block"]).unwrap();
            assert!(height > previous);
            previous = height;
            let words = storage(&snapshot["storage"]).unwrap();
            for observation in items(snapshot, "observations").unwrap() {
                let state = decode(&layout, text(&observation["holder"]).unwrap(), &words).unwrap();
                let expected = uint(&observation["balance_of"]).unwrap();
                assert_eq!(state.balance().unwrap(), expected);
                checks += 1;
                excluded += usize::from(state.is_excluded);
                direct_word_mismatches += usize::from(state.holder.tokens != expected);
            }
        }
    }
    assert_eq!((checks, excluded, direct_word_mismatches), (329, 22, 177));
}

#[test]
fn deployed_reflection_controls_preserve_branches_rounding_and_reverts() {
    let report: Value = serde_json::from_str(include_str!("../../tests/fixtures/reflection/controls.json")).unwrap();
    let mut checks = 0;
    let mut reverts = 0;
    for token in items(&report, "tokens").unwrap() {
        let layout: Layout = serde_json::from_value(token["layout"].clone()).unwrap();
        for case in items(token, "cases").unwrap() {
            let state = decode(&layout, text(&case["holder"]).unwrap(), &storage(&case["storage"]).unwrap()).unwrap();
            if case["outcome"]["reverted"] == true {
                assert!(state.balance().is_err(), "{}", case["name"]);
                reverts += 1;
            } else {
                assert_eq!(state.balance().unwrap(), uint(&case["outcome"]["balance_of"]).unwrap(), "{}", case["name"]);
            }
            checks += 1;
        }
    }
    assert_eq!((checks, reverts), (48, 10));
}

#[test]
fn missing_or_inconsistent_reflection_initialization_is_rejected() {
    let token = captured().remove(1);
    let layout: Layout = serde_json::from_value(token["layout"].clone()).unwrap();
    let snapshot = &token["snapshots"][0];
    let holder = text(&snapshot["observations"][0]["holder"]).unwrap();
    let words = storage(&snapshot["storage"]).unwrap();
    let mut state = decode(&layout, holder, &words).unwrap();
    assert!(!state.is_excluded);
    state.supply = None;
    assert!(state.balance().unwrap_err().to_string().contains("missing initialized"));
    for key in [word(layout.total_reflections), word(layout.total_tokens), word(layout.excluded_array)] {
        let mut missing = words.clone();
        assert!(missing.remove(&key).is_some());
        assert!(decode(&layout, holder, &missing).is_err());
    }
    let mut state = decode(&layout, holder, &words).unwrap();
    let supply = state.supply.as_mut().unwrap();
    supply.excluded_length += 1;
    assert!(state.balance().unwrap_err().to_string().contains("incomplete initialized exclusion list"));
    let mut state = decode(&layout, holder, &words).unwrap();
    let mut conflicting = state.holder.clone();
    conflicting.tokens = conflicting.tokens.overflowing_add(U256::one()).0;
    let supply = state.supply.as_mut().unwrap();
    supply.excluded.push(conflicting);
    supply.excluded_length += 1;
    assert!(state.balance().unwrap_err().to_string().contains("inconsistent initialized account"));
}

#[test]
fn excluded_holder_bypasses_uninitialized_global_state() {
    let state = State {
        holder: Account {
            address: [6; 20],
            reflections: U256::MAX,
            tokens: U256::from(77),
        },
        is_excluded: true,
        supply: None,
    };
    assert_eq!(state.balance().unwrap(), U256::from(77));
}

#[test]
fn missing_exclusion_elements_or_member_balances_are_not_treated_as_zero() {
    let token = captured().remove(0);
    let layout: Layout = serde_json::from_value(token["layout"].clone()).unwrap();
    let snapshot = &token["snapshots"][0];
    let observation = items(snapshot, "observations").unwrap().iter().find(|o| o["is_excluded"] == false).unwrap();
    let holder = text(&observation["holder"]).unwrap();
    let words = storage(&snapshot["storage"]).unwrap();
    let state = decode(&layout, holder, &words).unwrap();
    let first_member = &state.supply.as_ref().unwrap().excluded[0];
    let member = format!("0x{}", hex::encode(first_member.address));
    for key in [
        array_key(layout.excluded_array, 0),
        crate::survey::mapping_key(&member, &word(layout.reflections)).unwrap(),
        crate::survey::mapping_key(&member, &word(layout.tokens)).unwrap(),
    ] {
        let mut missing = words.clone();
        assert!(missing.remove(&key).is_some());
        assert!(decode(&layout, holder, &missing).is_err());
    }
}

#[test]
fn captured_passive_changes_match_fresh_rpc_without_holder_storage_writes() {
    let fixture: Value = serde_json::from_str(include_str!("../../tests/fixtures/reflection/10SET-passive.json")).unwrap();
    let layout: Layout = serde_json::from_value(fixture["layout"].clone()).unwrap();
    let controls = items(&fixture, "independent_rpc_checks").unwrap();
    let cases = items(&fixture, "cases").unwrap();
    assert_eq!(cases.len(), 12);
    assert_eq!(controls.len(), 24);
    for case in cases {
        assert_eq!(case["holder_storage_writes"], 0);
        let holder = text(&case["holder"]).unwrap();
        let before = decode(&layout, holder, &storage(&case["before_storage"]).unwrap()).unwrap();
        let after = decode(&layout, holder, &storage(&case["after_storage"]).unwrap()).unwrap();
        assert_eq!(before.holder, after.holder);
        assert_eq!(before.is_excluded, after.is_excluded);
        assert_ne!(before.balance().unwrap(), after.balance().unwrap());
        for (boundary, state, block) in [
            ("before", before, number(&case["block"]).unwrap() - 1),
            ("after", after, number(&case["block"]).unwrap()),
        ] {
            let expected = controls
                .iter()
                .find(|c| c["holder"] == holder && c["boundary"] == boundary && c["block"] == block)
                .unwrap();
            assert_eq!(state.balance().unwrap(), uint(&expected["balance_of"]).unwrap());
        }
    }
}
