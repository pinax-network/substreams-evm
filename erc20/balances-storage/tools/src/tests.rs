use crate::{audit::*, cli::*, comparison::*, data::*, probe::*, rpc::*};
use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Parser;
use primitive_types::U256;
use rusqlite::Connection;
use serde_json::{json, Value};
use std::{fs, sync::Mutex};
const TOKEN: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn published_deployment_must_match_address_runtime_and_source_hashes() {
    let content = "contract Example {}";
    let metadata =
        json!({"sources":{"Example.sol":{"content":content,"keccak256":format!("0x{}",hex::encode(erc20_balances_storage::hash(content.as_bytes())))}}});
    let artifact = json!({"address":TOKEN,"deployedBytecode":"0xabcd","metadata":metadata.to_string()});
    assert!(crate::inspect::bind_deployment(&artifact, TOKEN, &[0xab, 0xcd]).is_ok());
    assert!(crate::inspect::bind_deployment(&artifact, &address(), &[0xab, 0xcd]).is_err());
    assert!(crate::inspect::bind_deployment(&artifact, TOKEN, &[0xab, 0xce]).is_err());
    let mut bad = artifact;
    let mut metadata = metadata;
    metadata["sources"]["Example.sol"]["content"] = json!("contract Changed {}");
    bad["metadata"] = json!(metadata.to_string());
    assert!(crate::inspect::bind_deployment(&bad, TOKEN, &[0xab, 0xcd]).is_err());
}

#[test]
fn runtime_qualification_rejects_a_changed_zero_balance_dependency() {
    struct DependencyRpc;
    impl Rpc for DependencyRpc {
        fn request(&self, payload: Value) -> Result<Value> {
            let result = match payload["method"].as_str().unwrap() {
                "eth_getCode" => json!("0xaa"),
                "eth_getStorageAt" => json!(hash(8)),
                _ => return FakeRpc::default().request(payload),
            };
            Ok(json!({"id":1,"result":result}))
        }
    }
    let mut params = json!([{"contract":TOKEN,"balance_slot":hash(7),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&[0xaa]))),"zero_balance":{"value":hash(8),"storage_slot":hash(4)}}]);
    qualify_runtime(&DependencyRpc, 1, 2, &erc20_balances_storage::layout::parse(&params.to_string()).unwrap()).unwrap();
    params[0]["zero_balance"]["value"] = json!(hash(9));
    assert!(
        qualify_runtime(&DependencyRpc, 1, 2, &erc20_balances_storage::layout::parse(&params.to_string()).unwrap())
            .unwrap_err()
            .to_string()
            .contains("dependency value")
    );
}

#[test]
fn targeted_survey_preserves_rank_order_and_rejects_unranked_contracts() {
    let ranked = vec![json!({"contract":TOKEN,"rank":1}), json!({"contract":address(),"rank":2})];
    assert_eq!(crate::survey::select_tokens(&ranked, &[]).unwrap().len(), 2);
    assert_eq!(crate::survey::select_tokens(&ranked, &[address()]).unwrap()[0]["rank"], 2);
    assert!(crate::survey::select_tokens(&ranked, &[format!("0x{}", "bb".repeat(20))]).is_err());
    assert!(crate::survey::select_tokens(&ranked, &["malformed".into()]).is_err());
}

#[test]
fn holder_state_distinguishes_unknowns_repeats_and_explicit_zero() {
    use crate::coverage::HolderState;
    let key = (TOKEN.to_string(), address());
    let checkpoint = [(key.clone(), 5.into())].into();
    let mut state = HolderState::new(checkpoint);
    state.apply(1, &hash(1), &hash(0), &Balances::new(), &[(key.clone(), 5.into())].into()).unwrap();
    assert_eq!(state.tokens[TOKEN]["unseeded_unknown_rows"], 1);
    assert_eq!(state.tokens[TOKEN]["seeded_matches"], 1);
    state
        .apply(2, &hash(2), &hash(1), &[(key.clone(), 0.into())].into(), &[(key.clone(), 0.into())].into())
        .unwrap();
    state.apply(3, &hash(3), &hash(2), &Balances::new(), &[(key, 0.into())].into()).unwrap();
    assert_eq!(state.tokens[TOKEN]["carry_forward_matches"], 1);
    assert_eq!(state.tokens[TOKEN]["seeded_matches"], 3);
    assert_eq!(state.tokens[TOKEN]["unseeded_unknown_rows"], 1);
}
#[test]
fn holder_state_rejects_gaps_and_detects_missed_mutations() {
    use crate::coverage::HolderState;
    let key = (TOKEN.to_string(), address());
    let mut state = HolderState::new([(key.clone(), 5.into())].into());
    state.apply(1, &hash(1), &hash(0), &Balances::new(), &[(key.clone(), 6.into())].into()).unwrap();
    assert_eq!(state.tokens[TOKEN]["seeded_value_mismatches"], 1);
    assert!(state.apply(3, &hash(3), &hash(1), &Balances::new(), &Balances::new()).is_err());
    assert!(state.apply(2, &hash(2), &hash(0), &Balances::new(), &Balances::new()).is_err());
    assert_eq!(state.tokens[TOKEN]["unseeded_unknown_rows"], 1);
}
#[test]
fn sload_diagnostics_preserve_zero_extra_reads_and_depth() {
    let trace = json!({"structLogs":[{"pc":10,"op":"SLOAD","depth":1,"stack":["0x123"]},{"pc":11,"op":"POP","depth":1,"stack":["0x0"]},{"pc":20,"op":"SLOAD","depth":1,"stack":["4"]},{"pc":21,"op":"JUMP","depth":1,"stack":["6f05b59d3b200000"]}]});
    let reads = crate::inspect::storage_reads(&trace).unwrap();
    assert_eq!(reads.len(), 2);
    assert_eq!(reads[0]["value"], hash(0));
    assert_eq!(quantity(&reads[1]["value"]).unwrap().to_string(), "8000000000000000000");
    let mut bad = trace;
    bad["structLogs"][1]["depth"] = json!(2);
    assert!(crate::inspect::storage_reads(&bad).is_err());
}

#[test]
fn captured_core_layouts_emit_correct_balances_for_all_four_tokens() {
    use prost::Message;
    let b = substreams_ethereum::pb::eth::v2::Block::decode(include_bytes!("../../tests/fixtures/bsc-122260950.pb").as_slice()).unwrap();
    let layouts = erc20_balances_storage::layout::parse(include_str!("../../tests/fixtures/bsc-reviewed-layouts.json")).unwrap();
    let reference: Value = serde_json::from_str(include_str!("../../tests/fixtures/bsc-122260950-core-reference.json")).unwrap();
    assert_eq!(reference["block"], b.number);
    let expected = candidate_rows(&reference).unwrap();
    let events = erc20_balances_storage::project(&b, &layouts).unwrap();
    let emitted=candidate_rows(&json!({"balances":events.balances.into_iter().map(|b|json!({"contract":format!("0x{}",hex::encode(b.contract.unwrap())),"address":format!("0x{}",hex::encode(b.address)),"amount":b.amount})).collect::<Vec<_>>()})).unwrap();
    assert_eq!(emitted.keys().map(|k| &k.0).collect::<std::collections::BTreeSet<_>>().len(), 4);
    assert!(emitted.len() > 18);
    for (key, amount) in emitted {
        assert_eq!(expected.get(&key), Some(&amount), "{key:?}");
    }
}

#[test]
fn real_default_balance_counterexample_fails_direct_mapping_qualification() {
    let r: Value = serde_json::from_str(include_str!("../../tests/fixtures/default-balance-inspection.json")).unwrap();
    assert_eq!(r["storage_word"], "0");
    assert_eq!(r["balance_of"], "8000000000000000000");
    let checks = items(&r, "read_only_state_overrides").unwrap();
    assert_eq!(checks.len(), 4);
    // Nonzero and uint256-max controls would falsely suggest a direct mapping.
    assert!(checks[1..].iter().all(|c| c["overridden_mapping_word"] == c["balance_of"]));
    let mismatches = checks.iter().filter(|c| c["overridden_mapping_word"] != c["balance_of"]).count();
    let stats = json!({"nonzero_holders":2,"changed_observations":4,"mismatches":mismatches});
    assert_eq!(classify_layout(&stats), "not_direct_balance_mapping");
    assert_eq!(r["storage_reads"][1]["key"], hash(4));
    assert_eq!(
        quantity(&r["storage_reads"][1]["value"]).unwrap().to_string(),
        r["balance_of"].as_str().unwrap()
    );
}

#[test]
fn ranking_uses_rows_and_deterministic_contract_ties() {
    let mut second = reference("9");
    second["balances"][0]["contract"] = json!(address());
    let r = crate::ranking::rank(&[(1, reference("1")), (2, reference("1")), (3, second.clone())].into()).unwrap();
    assert_eq!(r[0]["contract"], TOKEN);
    assert_eq!(r[0]["reference_rows"], 2);
    assert_eq!(r[0]["unique_holders"], 1);
    assert_eq!(r[0]["heights"], json!([1, 2]));
    let tied = crate::ranking::rank(&[(1, reference("1")), (2, second)].into()).unwrap();
    assert_eq!(tied[0]["contract"], address());
}

#[test]
fn active_samples_include_rare_tokens_and_both_range_boundaries() {
    let r = json!({"selected_tokens":[{"heights":[1,2,3,4,5,6,7,8,9]},{"heights":[15,18]},{"heights":[30]}]});
    assert_eq!(crate::ranking::sample_heights(&r, 3).unwrap(), vec![1, 5, 9, 15, 18, 30]);
    for heights in [json!([]), json!([2, 1]), json!([1, 1]), json!([0, 1])] {
        assert!(crate::ranking::sample_heights(&json!({"selected_tokens":[{"heights":heights}]}), 8).is_err());
    }
}

#[test]
fn survey_never_calls_shared_value_equality_complete_parity() {
    let reference = candidate_rows(&reference("0")).unwrap();
    let mut metrics = crate::survey::Metrics::default();
    metrics.observe(&reference, &reference);
    assert_eq!(metrics.json()["exact_row_parity"], true);
    metrics.observe(&Balances::new(), &reference);
    assert_eq!(metrics.json()["reference_only_rows"], 1);
    assert_eq!(metrics.json()["value_mismatches"], 0);
    assert_eq!(metrics.json()["exact_row_parity"], false);
    assert_eq!(crate::survey::Metrics::default().json()["exact_row_parity"], false);
}

#[test]
fn survey_retains_wrong_values_and_extra_storage_rows() {
    let mut metrics = crate::survey::Metrics::default();
    metrics.observe(&candidate_rows(&reference("9")).unwrap(), &candidate_rows(&reference("8")).unwrap());
    assert_eq!(metrics.json()["value_mismatches"], 1);
    assert_eq!(metrics.json()["exact_row_parity"], false);
    metrics.observe(&candidate_rows(&reference("0")).unwrap(), &Balances::new());
    assert_eq!(metrics.json()["storage_only_rows"], 1);
}

#[test]
fn completed_survey_fails_parity_for_missing_rows_errors_or_mismatches() {
    use crate::survey::parity_status;
    assert_eq!(parity_status(&[]), "insufficient_samples");
    assert_eq!(parity_status(&[json!({"status":"insufficient_samples"})]), "insufficient_samples");
    assert_eq!(parity_status(&[json!({"status":"rpc_unresolved"})]), "rpc_unresolved");
    assert_eq!(parity_status(&[json!({"status":"value_mismatch"})]), "mismatch");
    assert_eq!(
        parity_status(&[json!({"status":"candidate_matches_values_not_qualified","exact_mapper_parity":false})]),
        "coverage_gap"
    );
    assert_eq!(parity_status(&[json!({"exact_mapper_parity":true})]), "bounded_parity");
}

#[test]
fn diagnostic_storage_key_matches_captured_keccak_preimages() {
    use prost::Message;
    let block = substreams_ethereum::pb::eth::v2::Block::decode(include_bytes!("../../tests/fixtures/bsc-122260950.pb").as_slice()).unwrap();
    let rows = erc20_balances_storage::discovery::project(&block).unwrap().candidates;
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(
            crate::survey::mapping_key(&format!("0x{}", hex::encode(row.address)), &format!("0x{}", hex::encode(row.mapping_slot))).unwrap(),
            format!("0x{}", hex::encode(row.storage_key))
        );
    }
}

#[test]
fn ranking_and_capture_do_not_require_a_token_layout_or_extra_map() {
    assert!(Cli::try_parse_from(["tools", "rank-tokens", "--output", "out"]).is_ok());
    assert!(Cli::try_parse_from(["tools", "capture-blocks", "--ranking", "rank.json", "--output", "out"]).is_ok());
    assert!(Cli::try_parse_from(["tools", "capture-blocks", "--output", "out"]).is_err());
    assert!(Cli::try_parse_from(["tools", "capture-blocks", "--ranking", "rank.json", "--start", "1", "--output", "out"]).is_err());
}

fn address() -> String {
    format!("0x{}", "11".repeat(20))
}
fn hash(height: u64) -> String {
    format!("0x{height:064x}")
}
fn candidate(height: u64, _before: &str, after: &str) -> Value {
    json!({"number":height.to_string(),"hash":hash(height),"parentHash":hash(height-1),
        "balances":[{"contract":TOKEN,"address":address(),"amount":after}]})
}
fn reference(amount: &str) -> Value {
    json!({"balances":[{"contract":TOKEN,"address":address(),"amount":amount}]})
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
                Ok(json!({"id":1,"result":{"number":format!("{height:#x}"),"hash":hash(height),"parentHash":hash(height.saturating_sub(1))}}))
            }
            "eth_call" => Ok(json!({"id":payload["id"],"result":format!("0x{:064x}",self.after)})),
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
fn reference_only_rows_never_seed_candidate_state() {
    let mut ours = candidate(1, "5", "0");
    ours["balances"] = json!([]);
    let r = report([(1, ours)].into(), [(1, reference("0"))].into());
    assert_eq!(r["snapshot_comparisons"], 0);
    assert_eq!(r["distinct_storage_keys"], 0);
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
    assert_eq!(r["reference_contracts_without_candidate_updates"], 1);
    assert_eq!(r["reference_only_updates"], 1);
    assert_eq!(r["status"], "coverage_gap");
}
#[test]
fn any_token_address_is_supported_without_a_builtin_allowlist() {
    let mut ours = candidate(1, "5", "0");
    ours["balances"][0]["contract"] = json!(address());
    assert_eq!(candidate_rows(&ours).unwrap().len(), 1);
}
#[test]
fn forks_and_wrong_block_numbers_are_rejected() {
    let mut next = candidate(2, "0", "0");
    next["parentHash"] = json!(hash(0));
    assert!(validate_blocks(&[(1, candidate(1, "5", "0")), (2, next)].into()).is_err());
    assert!(validate_blocks(&[(2, candidate(1, "5", "0"))].into()).is_err());
}
#[test]
fn public_events_are_bound_to_contiguous_rpc_headers_for_auditing() {
    let blocks = bind_headers(&FakeRpc::default(), &[(1, reference("0")), (2, reference("0"))].into()).unwrap();
    assert_eq!(blocks[&2]["parentHash"], hash(1));
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
        json!({"@module":"map_events","@type":"evm.balances.v1.Events","@block":1,"@data":reference("0")})
    );
    fs::write(&path, &row).unwrap();
    assert!(read_stream(&path, 1, 2, "map_events").is_ok());
    assert!(read_stream(&path, 1, 3, "map_events").is_err());
    assert!(read_stream(&path, 1, 2, "unknown_module").is_err());
    fs::write(&path, row.repeat(2)).unwrap();
    assert!(read_stream(&path, 1, 2, "map_events").is_err());
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
fn token_result_requires_a_complete_leading_abi_word() {
    assert_eq!(balance_result(&json!(format!("0x{}", "ff".repeat(32))), true).unwrap(), U256::MAX);
    for value in [
        "0x".to_string(),
        "0x0".into(),
        format!("0x{}", "00".repeat(31)),
        format!("0x{}zz", "00".repeat(32)),
        "bad".into(),
    ] {
        assert!(balance_result(&json!(value), true).is_err());
    }
}
#[test]
fn token_return_decoding_matches_the_actual_rpc_reference_abi_decoder() {
    use substreams_abis::standard::erc20::functions::BalanceOf;
    for length in 0..=100 {
        let bytes: Vec<u8> = (0..length).map(|i| (i * 17) as u8).collect();
        let expected = BalanceOf::output(&bytes).map(|v| v.to_string()).ok();
        let actual = balance_result(&json!(format!("0x{}", hex::encode(bytes))), true).map(|v| v.to_string()).ok();
        assert_eq!(actual, expected, "return length {length}");
    }
    let captured: Value = serde_json::from_str(include_str!("../../tests/fixtures/vusdt-return-data.json")).unwrap();
    let value = &captured["response"]["result"];
    assert_eq!(
        balance_result(value, true).unwrap().to_string(),
        captured["original"]["storage"].as_str().unwrap()
    );
    assert!(balance_result(value, false).is_err());
}
#[test]
fn every_emitted_balance_uses_its_current_canonical_hash() {
    let rpc = FakeRpc::default();
    let checks = audit_block(&rpc, &candidate(1, "5", "0"), 25).unwrap();
    assert_eq!(checks.len(), 1);
    assert!(checks.iter().all(|c| c["match"] == true));
    let requests = rpc.requests.lock().unwrap();
    let request = requests.iter().find(|r| r["method"] == "eth_call").unwrap();
    assert_eq!(request["params"][1], block_ref(&hash(1)));
    assert!(!requests.iter().any(Value::is_array));
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
    assert_eq!(checks[0]["match"], false);
    assert_eq!(checks[0]["rpc"], "1");
}
#[test]
fn wrong_hash_fails_before_balance_calls() {
    let rpc = FakeRpc::default();
    let mut c = candidate(1, "5", "0");
    c["hash"] = json!(hash(9));
    assert!(audit_block(&rpc, &c, 25).is_err());
    assert!(!rpc.requests.lock().unwrap().iter().any(|r| r.is_array() || r["method"] == "eth_call"));
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
fn public_output_must_respect_configured_layouts_and_never_include_native() {
    let blocks = [(1, candidate(1, "5", "0"))].into();
    let layouts = erc20_balances_storage::layout::parse(&json!([{"contract":TOKEN,"balance_slot":hash(7),"code_hash":hash(8)}]).to_string()).unwrap();
    assert!(validate_events_layouts(&blocks, &layouts).is_ok());
    assert!(validate_events_layouts(&blocks, &[]).is_err());
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
    let success = record_run(&output, json!({"status":"incomplete","checks":0,"mismatches":0}), |report| {
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
    assert_eq!(result["checks"], 1);
    assert_eq!(fs::read_to_string(output.join("rpc-checks.jsonl")).unwrap().lines().count(), 1);
    assert!(new_output(&output).is_err());
}
#[test]
fn cli_uses_erc20_reference_and_rejects_invalid_bounds() {
    let cli = Cli::try_parse_from(["tools", "compare", "--start", "1", "--output", "out", "--layouts", "layouts.json"]).unwrap();
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
    let contract = TOKEN;
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

#[test]
fn runtime_qualification_uses_each_configured_contract_and_code_hash() {
    struct CodeRpc {
        calls: Mutex<Vec<Value>>,
    }
    impl Rpc for CodeRpc {
        fn request(&self, payload: Value) -> Result<Value> {
            if payload["method"] == "eth_getCode" {
                self.calls.lock().unwrap().push(payload.clone());
                let code = if payload["params"][0] == TOKEN { "0xaa" } else { "0xbb" };
                Ok(json!({"id":1,"result":code}))
            } else {
                FakeRpc::default().request(payload)
            }
        }
    }
    let json = json!([
        {"contract":TOKEN,"balance_slot":hash(7),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&[0xaa])))},
        {"contract":address(),"balance_slot":hash(100),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&[0xbb])))}
    ]);
    let mut layouts = erc20_balances_storage::layout::parse(&json.to_string()).unwrap();
    let rpc = CodeRpc { calls: Mutex::new(Vec::new()) };
    qualify_runtime(&rpc, 1, 2, &layouts).unwrap();
    let calls = rpc.calls.lock().unwrap();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0]["params"], json!([TOKEN, block_ref(&hash(0))]));
    assert_eq!(calls[3]["params"], json!([address(), block_ref(&hash(1))]));
    drop(calls);
    layouts[1].code_hash = [0; 32];
    assert!(qualify_runtime(&rpc, 1, 2, &layouts).is_err());
}

#[test]
fn runtime_qualification_rejects_changed_proxy_target_or_implementation_code() {
    struct ProxyRpc {
        wrong_target: bool,
        wrong_code: bool,
    }
    impl Rpc for ProxyRpc {
        fn request(&self, payload: Value) -> Result<Value> {
            let result = match text(&payload["method"])? {
                "eth_getStorageAt" => json!(format!(
                    "0x{}{}",
                    "00".repeat(12),
                    if self.wrong_target { "33".repeat(20) } else { "11".repeat(20) }
                )),
                "eth_getCode" => json!(if payload["params"][0] == TOKEN {
                    "0xaa"
                } else if self.wrong_code {
                    "0xcc"
                } else {
                    "0xbb"
                }),
                _ => return FakeRpc::default().request(payload),
            };
            Ok(json!({"id":1,"result":result}))
        }
    }
    let layouts=erc20_balances_storage::layout::parse(&json!([{"contract":TOKEN,"balance_slot":hash(1),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&[0xaa]))),"proxy":{"implementation_slot":hash(99),"implementation":address(),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&[0xbb])))}}]).to_string()).unwrap();
    qualify_runtime(
        &ProxyRpc {
            wrong_target: false,
            wrong_code: false,
        },
        1,
        2,
        &layouts,
    )
    .unwrap();
    assert!(qualify_runtime(
        &ProxyRpc {
            wrong_target: true,
            wrong_code: false
        },
        1,
        2,
        &layouts
    )
    .unwrap_err()
    .to_string()
    .contains("proxy implementation"));
    assert!(qualify_runtime(
        &ProxyRpc {
            wrong_target: false,
            wrong_code: true
        },
        1,
        2,
        &layouts
    )
    .unwrap_err()
    .to_string()
    .contains("implementation runtime"));
}
