use crate::{audit::*, cli::*, comparison::*, data::*, probe::*, rpc::*};
use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Parser;
use primitive_types::U256;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::{fs, sync::Mutex};

fn address() -> String {
    format!("0x{}", "11".repeat(20))
}
fn hash(height: u64) -> String {
    format!("0x{height:064x}")
}
fn candidate(height: u64, before: &str, after: &str) -> Value {
    json!({"number":height.to_string(),"hash":hash(height),"parentHash":hash(height-1),
        "balances":[{"contract":WBNB,"address":address(),"oldAmount":before,"amount":after}]})
}
fn reference(amount: &str) -> Value {
    json!({"balances":[{"contract":WBNB,"address":address(),"amount":amount}]})
}
fn report(ours: Blocks, theirs: Blocks) -> Value {
    let temp = tempfile::tempdir().unwrap();
    compare(&ours, &theirs, 1, ours.len() as u64 + 1, &temp.path().join("comparison.sqlite"))
        .unwrap()
        .report
}
struct FakeRpc {
    requests: Mutex<Vec<Value>>,
    after: u64,
    fail_height: Option<u64>,
}
impl Default for FakeRpc {
    fn default() -> Self {
        Self {
            requests: Mutex::new(Vec::new()),
            after: 0,
            fail_height: None,
        }
    }
}
impl Rpc for FakeRpc {
    fn request(&self, payload: Value) -> Result<Value> {
        self.requests.lock().unwrap().push(payload.clone());
        if let Some(rows) = payload.as_array() {
            let mut results = Vec::new();
            for row in rows {
                let before = row["params"][1]["blockHash"] == hash(0);
                results.push(json!({"id":row["id"],"result":format!("0x{:064x}",if before {5} else {self.after})}));
            }
            results.reverse();
            return Ok(json!(results));
        }
        match text(&payload["method"])? {
            "eth_getBlockByNumber" => {
                let height = quantity(&payload["params"][0])?.low_u64();
                if self.fail_height == Some(height) {
                    bail!("test RPC transport failed");
                }
                Ok(json!({"id":1,"result":{"number":format!("{height:#x}"),"hash":hash(height)}}))
            }
            "eth_call" => Ok(json!({"id":1,"result":format!("0x{:064x}",self.after)})),
            _ => bail!("unexpected test RPC method"),
        }
    }
}

#[test]
fn uint256_and_explicit_zero_remain_sqlite_text() {
    let temp = tempfile::tempdir().unwrap();
    let database = temp.path().join("comparison.sqlite");
    let max = U256::MAX.to_string();
    let result = compare(
        &[(1, candidate(1, "5", &max)), (2, candidate(2, &max, "0"))].into(),
        &[(1, reference(&max)), (2, reference("0"))].into(),
        1,
        3,
        &database,
    )
    .unwrap();
    assert_eq!(result.report["status"], "bounded_parity");
    let db = Connection::open(database).unwrap();
    let rows = db
        .prepare("SELECT balance,typeof(balance) FROM observations WHERE side='storage' ORDER BY block_num")
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .unwrap()
        .collect::<rusqlite::Result<Vec<_>>>()
        .unwrap();
    assert_eq!(rows, vec![(max, "text".into()), ("0".into(), "text".into())]);
}
#[test]
fn reference_seed_is_not_verified_and_reports_output_coverage_gap() {
    let mut ours = candidate(1, "5", "0");
    ours["balances"] = json!([]);
    let r = report([(1, ours)].into(), [(1, reference("0"))].into());
    assert_eq!(r["reference_bootstrap_seeds"], 1);
    assert_eq!(r["snapshot_comparisons"], 0);
    assert_eq!(r["seed_only_comparisons"], 1);
    assert_eq!(r["reference_only_updates"], 1);
    assert_eq!(r["status"], "coverage_gap");
}
#[test]
fn missing_reference_is_not_zero() {
    let r = report([(1, candidate(1, "5", "0"))].into(), [(1, json!({}))].into());
    assert_eq!(r["status"], "coverage_gap");
    assert_eq!(r["candidate_only_keys"], 1);
}
#[test]
fn missing_later_update_is_a_mismatch() {
    let r = report(
        [(1, candidate(1, "5", "0")), (2, candidate(2, "0", "9"))].into(),
        [(1, reference("0")), (2, json!({}))].into(),
    );
    assert_eq!(r["differences"], 1);
    assert_eq!(r["status"], "mismatch");
}
#[test]
fn unsupported_reference_tokens_are_visible_not_claimed_as_storage() {
    let mut other = reference("20");
    other["balances"][0]["contract"] = json!(address());
    let r = report([(1, candidate(1, "5", "0"))].into(), [(1, other)].into());
    assert_eq!(r["unsupported_contracts"], 1);
    assert_eq!(r["reference_only_updates"], 1);
    assert_eq!(r["reference_bootstrap_seeds"], 0);
    assert_eq!(r["status"], "coverage_gap");
}
#[test]
fn unresolved_storage_cannot_pass() {
    let mut ours = candidate(1, "5", "0");
    ours["unresolvedWbnbSlots"] = json!([{"key":hash(0)}]);
    assert_eq!(report([(1, ours)].into(), [(1, reference("0"))].into())["status"], "mismatch");
}
#[test]
fn forks_and_wrong_block_numbers_are_rejected() {
    let mut next = candidate(2, "0", "0");
    next["parentHash"] = json!(hash(0));
    assert!(validate_blocks(&[(1, candidate(1, "5", "0")), (2, next)].into()).is_err());
    assert!(validate_blocks(&[(2, candidate(1, "5", "0"))].into()).is_err());
}
#[test]
fn cross_block_old_balance_discontinuity_is_rejected() {
    let temp = tempfile::tempdir().unwrap();
    assert!(compare(
        &[(1, candidate(1, "5", "0")), (2, candidate(2, "5", "0"))].into(),
        &[(1, reference("0")), (2, reference("0"))].into(),
        1,
        3,
        &temp.path().join("db")
    )
    .is_err());
}
#[test]
fn duplicate_candidate_and_reference_holders_are_rejected() {
    let mut c = candidate(1, "5", "0");
    c["balances"] = json!([c["balances"][0], c["balances"][0]]);
    assert!(candidate_rows(&c).is_err());
    assert!(reference_rows(&c).is_err());
}
#[test]
fn stream_requires_exact_range_and_rejects_duplicates_or_wrong_module() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("capture.jsonl");
    let row = format!(
        "{}\n",
        json!({"@module":"map_storage_changes","@type":"evm.balances.storage.v1.BlockBalances","@block":1,"@data":candidate(1,"5","0")})
    );
    fs::write(&path, &row).unwrap();
    assert!(read_stream(&path, 1, 2, "map_storage_changes").is_ok());
    assert!(read_stream(&path, 1, 3, "map_storage_changes").is_err());
    assert!(read_stream(&path, 1, 2, "map_events").is_err());
    fs::write(&path, row.repeat(2)).unwrap();
    assert!(read_stream(&path, 1, 2, "map_storage_changes").is_err());
}
#[test]
fn rpc_disagreement_audit_preserves_database_and_uses_original_hash() {
    let temp = tempfile::tempdir().unwrap();
    let db = temp.path().join("db");
    let blocks = [(1, candidate(1, "5", "0"))].into();
    compare(&blocks, &[(1, reference("10"))].into(), 1, 2, &db).unwrap();
    let rpc = FakeRpc::default();
    assert_eq!(audit_differences(&rpc, &db, &blocks, 10).unwrap()["agrees_with_storage"], 1);
    let requests = rpc.requests.lock().unwrap();
    let balance = requests.iter().find(|r| r["method"] == "eth_call").unwrap();
    assert_eq!(balance["params"][1], block_ref(&hash(1)));
    assert_eq!(
        Connection::open(&db)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM differences", [], |r| r.get::<_, u64>(0))
            .unwrap(),
        1
    );
    assert_eq!(audit_differences(&rpc, &db, &blocks, 0).unwrap()["unchecked"], 1);
}
#[test]
fn binary_encodings_and_invalid_uint256_values() {
    assert_eq!(binary(&json!(STANDARD.encode([0x11; 20])), 20).unwrap(), address());
    assert_eq!(binary(&json!(address()), 20).unwrap(), address());
    for v in [json!(-1), json!(format!("{}0", U256::MAX)), json!(1.5), json!(true), json!("1.5"), Value::Null] {
        assert!(uint(&v).is_err());
    }
}
#[test]
fn batches_match_by_id_not_response_order() {
    assert_eq!(
        batch_results(json!([{"id":1,"result":"0xb"},{"id":0,"result":"0xa"}]), 2).unwrap(),
        vec![json!("0xa"), json!("0xb")]
    );
}
#[test]
fn missing_duplicate_error_null_and_invalid_batch_ids_fail() {
    for response in [
        json!([]),
        json!([{"id":0,"result":"0x0"},{"id":0,"result":"0x0"}]),
        json!([{"id":0,"error":{"code":-1}}]),
        json!([{"id":0,"result":null}]),
        json!([{"id":2,"result":"0x0"}]),
        json!([{"id":true,"result":"0x0"}]),
    ] {
        let count = if response.as_array().unwrap().len() == 2 { 2 } else { 1 };
        assert!(batch_results(response, count).is_err());
    }
}
#[test]
fn token_result_requires_one_complete_abi_word() {
    assert_eq!(balance_result(&json!(format!("0x{}", "ff".repeat(32))), true).unwrap(), U256::MAX);
    for value in ["0x".to_string(), "0x0".into(), format!("0x{}", "00".repeat(64)), "bad".into()] {
        assert!(balance_result(&json!(value), true).is_err());
    }
}
#[test]
fn every_balance_uses_parent_and_current_canonical_hash() {
    let rpc = FakeRpc::default();
    let checks = audit_block(&rpc, &candidate(1, "5", "0"), 25).unwrap();
    assert_eq!(checks.len(), 2);
    assert!(checks.iter().all(|c| c["match"] == true));
    let requests = rpc.requests.lock().unwrap();
    let batch = requests.iter().find_map(Value::as_array).unwrap();
    assert_eq!(batch[0]["params"][1], block_ref(&hash(0)));
    assert_eq!(batch[1]["params"][1], block_ref(&hash(1)));
    assert!(batch.iter().all(|r| r["method"] == "eth_call"));
}
#[test]
fn rpc_balance_mismatch_is_retained() {
    let checks = audit_block(
        &FakeRpc {
            after: 1,
            ..Default::default()
        },
        &candidate(1, "5", "0"),
        25,
    )
    .unwrap();
    assert_eq!(checks[0]["match"], true);
    assert_eq!(checks[1]["match"], false);
    assert_eq!(checks[1]["rpc"], "1");
}
#[test]
fn wrong_hash_or_unresolved_slots_fail_before_balance_calls() {
    for key in ["hash", "unresolvedWbnbSlots"] {
        let rpc = FakeRpc::default();
        let mut c = candidate(1, "5", "0");
        c[key] = if key == "hash" { json!(hash(9)) } else { json!([{}]) };
        assert!(audit_block(&rpc, &c, 25).is_err());
        assert!(!rpc.requests.lock().unwrap().iter().any(Value::is_array));
    }
}
#[test]
fn zero_only_or_single_holder_discovery_is_insufficient() {
    let mut stats = json!({"nonzero_holders":0,"changed_observations":10});
    assert_eq!(classify_layout(&stats), "insufficient_evidence");
    stats["nonzero_holders"] = json!(1);
    assert_eq!(classify_layout(&stats), "insufficient_evidence");
    stats["nonzero_holders"] = json!(2);
    assert_eq!(classify_layout(&stats), "candidate_matches_rpc_not_qualified");
    stats["changed_observations"] = json!(1);
    assert_eq!(classify_layout(&stats), "insufficient_evidence");
}
#[test]
fn discovery_code_changes_errors_and_mismatches_prevent_qualification() {
    for (field, value, expected) in [
        ("code_changed", json!(true), "code_change_requires_review"),
        ("rpc_errors", json!(1), "rpc_unresolved"),
        ("mismatches", json!(1), "not_direct_balance_mapping"),
    ] {
        let mut stats = json!({"nonzero_holders":10,"changed_observations":100});
        stats[field] = value;
        assert_eq!(classify_layout(&stats), expected);
    }
}
#[test]
fn public_output_must_match_storage_exactly_and_never_include_native() {
    let blocks = [(1, candidate(1, "5", "0"))].into();
    assert!(verify_public_output(&blocks, &[(1, reference("0"))].into()).is_ok());
    assert!(verify_public_output(&blocks, &[(1, reference("1"))].into()).is_err());
    assert!(verify_public_output(&blocks, &[(1, json!({}))].into()).is_err());
    let mut native = candidate(1, "5", "0");
    native["balances"][0]["contract"] = json!("");
    assert!(candidate_rows(&native).is_err());
    let mut missing = reference("0");
    missing["balances"][0].as_object_mut().unwrap().remove("amount");
    assert!(reference_rows(&missing).is_err());
}
#[test]
fn failure_retains_completed_blocks_and_a_report() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("audit");
    let args = Range {
        start: 1,
        blocks: 2,
        output: output.clone(),
        package: default_package(),
        endpoint: String::new(),
        timeout: 1,
    };
    let success = record_run(&args, json!({"status":"incomplete","checks":0,"mismatches":0}), |report| {
        audit_blocks(
            &FakeRpc {
                fail_height: Some(2),
                ..Default::default()
            },
            &[(1, candidate(1, "5", "0")), (2, candidate(2, "0", "0"))].into(),
            25,
            1,
            &output,
            report,
        )
    })
    .unwrap();
    assert!(!success);
    let result: Value = serde_json::from_slice(&fs::read(output.join("report.json")).unwrap()).unwrap();
    assert_eq!(result["status"], "incomplete");
    assert_eq!(result["checked_blocks"], 1);
    assert_eq!(result["checks"], 2);
    assert_eq!(fs::read_to_string(output.join("rpc-checks.jsonl")).unwrap().lines().count(), 2);
    assert!(new_output(&output).is_err());
}
#[test]
fn cli_uses_erc20_reference_and_rejects_invalid_bounds() {
    let cli = Cli::try_parse_from(["tools", "compare", "--start", "1", "--output", "out"]).unwrap();
    let Commands::Compare(args) = cli.command else { panic!() };
    assert!(args.reference.ends_with("spkg/erc20-balances-v0.3.4.spkg"));
    assert!(args.range.validate(10000).is_ok());
    let mut range = args.range;
    range.start = 0;
    assert!(range.validate(10000).is_err());
    range.start = u64::MAX;
    assert!(range.validate(10000).is_err());
    assert!(Cli::try_parse_from(["tools", "audit-rpc", "--start", "-1", "--output", "out"]).is_err());
}

#[test]
fn http_errors_do_not_expose_endpoint_or_credentials() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/private-endpoint", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut buffer = [0; 4096];
        let _ = socket.read(&mut buffer).unwrap();
        socket
            .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
    });
    let error = HttpRpc::new(endpoint, Some("private-api-key".into()))
        .request(json!({}))
        .unwrap_err()
        .to_string();
    handle.join().unwrap();
    assert_eq!(error, "RPC HTTP 403");
}

#[test]
fn discovery_checks_are_persisted_and_later_code_changes_invalidate_layouts() {
    let contract = WBNB;
    let holder = address();
    let mut first = candidate(1, "5", "0");
    first["tokens"] = json!([{"contract":contract,"transferHolders":[holder],"storageChanges":1}]);
    first["candidates"] = json!([{"contract":contract,"address":holder,"mappingSlot":hash(3),"storageKey":hash(9),"oldAmount":"5","amount":"0"}]);
    let mut second = candidate(2, "0", "0");
    second["tokens"] = json!([{"contract":contract,"codeChanged":true}]);
    let temp = tempfile::tempdir().unwrap();
    let result = analyze(&FakeRpc::default(), &[(1, first), (2, second)].into(), temp.path()).unwrap();
    assert_eq!(result["candidate_value_checks"], 2);
    assert_eq!(result["layouts"][0]["classification"], "code_change_requires_review");
    assert_eq!(result["tokens"][0]["holders_with_some_matching_candidate"], 1);
    assert_eq!(result["candidate_contracts_matching_rpc"], 0);
    assert_eq!(fs::read_to_string(temp.path().join("checks.jsonl")).unwrap().lines().count(), 2);
}
