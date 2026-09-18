use super::fixture::{self, key, word, HELPER, POOL_A, TOKEN};
use crate::data::uint;
use crate::rpc::quantity;
use anyhow::Result;
use primitive_types::U256;
use serde_json::{json, Value};

fn historical() -> Vec<Value> {
    serde_json::from_str(include_str!("../../../tests/fixtures/og-model/historical.json")).unwrap()
}
fn controls() -> Vec<Value> {
    serde_json::from_str(include_str!("../../../tests/fixtures/og-model/controls.json")).unwrap()
}
fn preview() -> Value {
    serde_json::from_str(include_str!("../../../tests/fixtures/og-model/preview.json")).unwrap()
}
fn replace(v: &mut Value, contract: &str, slot: U256, value: U256) {
    let row = v["raw_state"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|r| r["contract"] == contract && r["key"] == word(slot))
        .unwrap();
    row["value"] = json!(word(value));
}
fn check(actual: Result<Vec<U256>>, expected: &Value, label: &str) {
    match actual {
        Ok(values) => {
            assert_eq!(
                values,
                expected["values"]
                    .as_array()
                    .unwrap_or_else(|| panic!("missing expected values: {label}"))
                    .iter()
                    .map(|v| uint(v).unwrap())
                    .collect::<Vec<_>>(),
                "{label}"
            );
        }
        Err(error) => {
            let message = error.to_string();
            assert!(message.starts_with("uint256 "), "unexpected rejection {label}: {message}");
            assert_eq!(expected["rpc_error"]["data"], format!("0x4e487b71{:064x}", 0x11), "{label}");
        }
    }
}

#[test]
fn raw_snapshots_explain_all_five_og_mismatches_and_both_helper_outputs() {
    let rows = historical();
    assert_eq!(rows.len(), 5);
    for row in rows {
        let state = fixture::decode(&row).unwrap();
        let value = state.evaluate().unwrap();
        assert_eq!(value.balance, uint(&row["expected"]["balance"]["values"][0]).unwrap());
        assert_eq!(value.hourly, uint(&row["expected"]["hourly"]["values"][0]).unwrap());
        assert_eq!(value.stopping_hour, uint(&row["expected"]["hourly"]["values"][1]).unwrap());
        assert_eq!(value.daily, uint(&row["expected"]["daily"]["values"][0]).unwrap());
        assert_ne!(state.raw, value.balance);
        assert!(row["original_failure"].is_object());
    }
}

#[test]
fn rpc_controls_bind_rounding_three_positive_days_caps_carry_and_reverts() {
    let rows = historical();
    let controls = controls();
    assert_eq!(controls.len(), 43);
    let mut previously_unsupported = 0;
    for row in controls {
        let mut snapshot = rows[row["base_case"].as_u64().unwrap() as usize - 1].clone();
        snapshot["timestamp"] = row["timestamp"].clone();
        for change in row["state_diff"].as_array().unwrap() {
            replace(
                &mut snapshot,
                change["contract"].as_str().unwrap(),
                quantity(&change["key"]).unwrap(),
                quantity(&change["value"]).unwrap(),
            );
        }
        let state = fixture::decode(&snapshot).unwrap();
        let name = row["name"].as_str().unwrap();
        check(state.hourly().map(|(reward, stop)| vec![reward, stop]), &row["expected"]["hourly"], name);
        check(state.daily().map(|reward| vec![reward]), &row["expected"]["daily"], name);
        check(state.evaluate().map(|value| vec![value.balance]), &row["expected"]["balance"], name);
        previously_unsupported += usize::from(row["status"] == "explicitly_unsupported");
    }
    // Preserve original evidence status: these captures were refused by the
    // initial model and their outputs are now independently modeled.
    assert_eq!(previously_unsupported, 3);
}

#[test]
fn fresh_preview_controls_bind_pool_decay_daily_fallback_rate_clock_and_overflow() {
    let fixture = preview();
    let rows = fixture["controls"].as_array().unwrap();
    assert_eq!(rows.len(), 49);
    for row in rows {
        let mut snapshot = fixture["base"].clone();
        snapshot["timestamp"] = row["timestamp"].clone();
        for change in row["state_diff"].as_array().unwrap() {
            replace(
                &mut snapshot,
                change["contract"].as_str().unwrap(),
                quantity(&change["key"]).unwrap(),
                quantity(&change["value"]).unwrap(),
            );
        }
        let state = fixture::decode(&snapshot).unwrap();
        let name = row["name"].as_str().unwrap();
        check(state.hourly().map(|(reward, stop)| vec![reward, stop]), &row["expected"]["hourly"], name);
        check(state.daily().map(|reward| vec![reward]), &row["expected"]["daily"], name);
        check(state.evaluate().map(|value| vec![value.balance]), &row["expected"]["balance"], name);
    }
}

#[test]
fn trace_returns_bind_internal_preview_reward_and_remaining_pool() {
    let fixture = preview();
    let state = fixture::decode(&fixture["base"]).unwrap();
    let rows = fixture["internal_preview_returns"].as_array().unwrap();
    assert_eq!(rows.len(), 10);
    for row in rows {
        let pool = uint(&row["input_pool"]).unwrap();
        let result = if row["daily"] == true {
            state.daily_preview(pool)
        } else {
            state.hourly_preview(pool)
        };
        assert_eq!(
            result.unwrap(),
            (uint(&row["expected_reward"]).unwrap(), uint(&row["expected_remaining_pool"]).unwrap()),
            "{}",
            row["name"]
        );
    }
}

#[test]
fn missing_words_runtimes_addresses_and_unmodeled_recursion_fail_closed() {
    let row = historical().remove(0);
    let state = fixture::decode(&row).unwrap();
    let holder = quantity(&row["holder"]).unwrap();
    for (contract, slot) in [
        (TOKEN, key(&[state.last_hour], 24)),
        (TOKEN, key(&[quantity(&json!(POOL_A)).unwrap()], 0)),
        (HELPER, U256::one()),
    ] {
        let mut missing = row.clone();
        missing["raw_state"]
            .as_array_mut()
            .unwrap()
            .retain(|r| !(r["contract"] == contract && r["key"] == word(slot)));
        assert!(fixture::decode(&missing).unwrap_err().to_string().starts_with("missing raw word"));
    }
    let mut bad_runtime = row.clone();
    bad_runtime["runtime_bindings"][0]["runtime_hash"] = json!("0x00");
    assert!(fixture::decode(&bad_runtime).is_err());
    let mut wrong_address = row.clone();
    replace(&mut wrong_address, HELPER, U256::one(), U256::one());
    assert!(fixture::decode(&wrong_address)
        .unwrap_err()
        .to_string()
        .contains("unreviewed stored dependency"));
    let mut missing_hour = state.clone();
    missing_hour.hours.pop();
    assert!(missing_hour.hourly().unwrap_err().to_string().contains("missing initialized hourly"));
    let mut missing_day = state.clone();
    missing_day.days.clear();
    assert!(missing_day.daily().unwrap_err().to_string().contains("missing initialized daily"));
    let mut nonconsecutive = state.clone();
    nonconsecutive.hours[1].index += U256::one();
    assert!(nonconsecutive.hourly().unwrap_err().to_string().contains("nonconsecutive"));
    let mut recursive = state;
    recursive.pool_hour_rate = U256::one();
    assert!(recursive.evaluate().unwrap_err().to_string().contains("unmodeled recursive pool reward"));
    // Field 1 is a packed uint8, not a proven Solidity bool. It is unused by
    // these paths, so decoding must preserve values above one without guessing.
    let mut packed = row;
    let base = key(&[holder], 9);
    let previous = quantity(
        &packed["raw_state"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["contract"] == TOKEN && r["key"] == word(base))
            .unwrap()["value"],
    )
    .unwrap();
    let mask = U256::from(255) << 160;
    replace(&mut packed, TOKEN, base, (previous & !mask) | (U256::from(255) << 160));
    assert_eq!(fixture::decode(&packed).unwrap().user[1], U256::from(255));
}
