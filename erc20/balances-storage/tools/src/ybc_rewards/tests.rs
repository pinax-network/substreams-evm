use super::fixture;
use crate::data::uint;
use anyhow::Result;
use primitive_types::U256;
use serde_json::Value;

fn historical() -> Vec<Value> {
    serde_json::from_str(include_str!("../../../tests/fixtures/ybc-rewards/historical.json")).unwrap()
}

#[test]
fn captured_ybc_storage_snapshots_explain_all_six_raw_word_mismatches() {
    let rows = historical();
    assert_eq!(rows.len(), 24);
    let mut mismatches = 0;
    for row in rows {
        let state = fixture::decode(&row["state"]).unwrap();
        let expected = (uint(&row["expected_reward"]).unwrap(), uint(&row["expected_stopping_hour"]).unwrap());
        assert_eq!(state.reward().unwrap(), expected, "holder {} at {}", row["holder"], row["block"]);
        let balance = uint(&row["expected_balance"]).unwrap();
        assert_eq!(state.balance().unwrap(), balance);
        mismatches += usize::from(state.raw != balance);
    }
    assert_eq!(mismatches, 6);
}

fn check(actual: Result<Vec<U256>>, expected: &Value, case: &str) {
    if let Some(values) = expected["values"].as_array() {
        assert_eq!(actual.unwrap(), values.iter().map(|v| uint(v).unwrap()).collect::<Vec<_>>(), "{case}");
        return;
    }
    let error = actual.unwrap_err().to_string();
    let data = expected["revert_data"].as_str().unwrap();
    if error == "INSUFFICIENT_LIQUIDITY" {
        let bytes = hex::decode(data.trim_start_matches("0x")).unwrap();
        assert_eq!(bytes.len(), 100);
        assert_eq!(&bytes[..4], &hex::decode("08c379a0").unwrap());
        assert_eq!(U256::from_big_endian(&bytes[4..36]), U256::from(32));
        assert_eq!(U256::from_big_endian(&bytes[36..68]), U256::from(22));
        assert_eq!(&bytes[68..90], b"INSUFFICIENT_LIQUIDITY");
        assert!(bytes[90..].iter().all(|b| *b == 0));
    } else {
        assert!(error.starts_with("uint256 "), "unexpected model error for {case}: {error}");
        assert_eq!(data, format!("0x4e487b71{:064x}", 0x11), "{case}");
    }
}

#[test]
fn captured_ybc_controls_preserve_rounding_caps_early_exits_and_reverts() {
    let rows: Vec<Value> = serde_json::from_str(include_str!("../../../tests/fixtures/ybc-rewards/controls.json")).unwrap();
    assert_eq!(rows.len(), 30);
    let mut reverted = 0;
    for row in rows {
        let state = fixture::decode(&row["state"]).unwrap();
        let name = row["name"].as_str().unwrap();
        check(state.reward().map(|(reward, stop)| vec![reward, stop]), &row["expected"]["reward"], name);
        check(state.balance().map(|balance| vec![balance]), &row["expected"]["balance"], name);
        for getter in ["reward", "balance"] {
            reverted += usize::from(row["expected"][getter]["revert_data"].is_string());
        }
    }
    assert_eq!(reverted, 17);
}

#[test]
fn missing_or_inconsistent_initialized_ybc_words_are_not_treated_as_zero() {
    let rows = historical();
    let value = &rows.iter().find(|r| r["raw_word_mismatch"] == true).unwrap()["state"];
    let state = fixture::decode(value).unwrap();
    assert_eq!(state.hours.len(), 240);
    let mut incomplete = state.clone();
    incomplete.hours.pop();
    assert!(incomplete.reward().unwrap_err().to_string().contains("missing initialized hourly state"));
    let mut inconsistent = state;
    inconsistent.initial_user_rate += U256::one();
    assert!(inconsistent.reward().unwrap_err().to_string().contains("initial rate differs"));
    let mut absent = value.clone();
    absent["hours"][0].as_object_mut().unwrap().remove("no_burn");
    assert!(fixture::decode(&absent).is_err());
    absent = value.clone();
    absent.as_object_mut().unwrap().remove("pool_balance");
    assert!(fixture::decode(&absent).is_err());
    absent = value.clone();
    absent.as_object_mut().unwrap().remove("hours");
    assert!(fixture::decode(&absent).is_err());
}
