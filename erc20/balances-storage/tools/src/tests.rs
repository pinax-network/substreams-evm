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
fn computed_runtime_qualification_binds_masked_selectors_at_both_boundaries() {
    let mut params: Value = serde_json::from_str(include_str!("../../tests/fixtures/bsc-computed-layout.json")).unwrap();
    params[0]["code_hash"] = json!(format!("0x{}", hex::encode(erc20_balances_storage::hash(&[0xaa]))));
    let layouts = erc20_balances_storage::layout::parse(&params.to_string()).unwrap();
    struct SelectorRpc {
        bad: &'static str,
        calls: Mutex<Vec<Value>>,
    }
    impl Rpc for SelectorRpc {
        fn request(&self, p: Value) -> Result<Value> {
            self.calls.lock().unwrap().push(p.clone());
            let result = match p["method"].as_str().unwrap() {
                "eth_getCode" => json!(if self.bad == "runtime" { "0xbb" } else { "0xaa" }),
                "eth_getStorageAt" => {
                    let owner_slot = p["params"][1] == hash(3);
                    let address = if owner_slot {
                        "c54cb14840cabf9a29b43d528af7dea7771f7494"
                    } else {
                        "0000000000000000000000000000000000000000"
                    };
                    if (self.bad == "owner" && owner_slot) || (self.bad == "zero" && !owner_slot) {
                        json!(hash(1))
                    } else {
                        // The upper 96 bits are packed flags, not the selector.
                        json!(format!("0x{}{}", "ff".repeat(12), address))
                    }
                }
                _ => return FakeRpc::default().request(p),
            };
            Ok(json!({"id":1,"result":result}))
        }
    }
    let rpc = SelectorRpc {
        bad: "",
        calls: Mutex::new(vec![]),
    };
    qualify_runtime(&rpc, 10, 20, &layouts).unwrap();
    let calls = rpc.calls.lock().unwrap();
    let selectors = calls.iter().filter(|c| c["method"] == "eth_getStorageAt").collect::<Vec<_>>();
    assert_eq!(selectors.len(), 4);
    assert!(selectors.iter().all(|c| c["params"][0] == params[0]["contract"]));
    assert_ne!(selectors[0]["params"][2], selectors[2]["params"][2]);
    for bad in ["owner", "zero", "runtime"] {
        assert!(
            qualify_runtime(
                &SelectorRpc {
                    bad,
                    calls: Mutex::new(vec![])
                },
                10,
                20,
                &layouts
            )
            .is_err(),
            "{bad}"
        );
    }
}

#[test]
fn clone_runtime_qualification_checks_the_forwarder_and_implementation() {
    use erc20_balances_storage::layout::{parse, VerifiedMinimalProxy};
    let proxy = VerifiedMinimalProxy {
        implementation: vec![0xbb; 20],
        code_hash: erc20_balances_storage::hash(&[0xaa]),
    };
    let p = json!([{"contract":TOKEN,"balance_slot":hash(51),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&proxy.runtime()))),"minimal_proxy":{"implementation":format!("0x{}",hex::encode(&proxy.implementation)),"code_hash":format!("0x{}",hex::encode(proxy.code_hash))}}]);
    let l = parse(&p.to_string()).unwrap();
    struct CloneRpc {
        runtime: Vec<u8>,
        bad: &'static str,
    }
    impl Rpc for CloneRpc {
        fn request(&self, p: Value) -> Result<Value> {
            if p["method"] != "eth_getCode" {
                return FakeRpc::default().request(p);
            }
            let code = if p["params"][0] == TOKEN {
                if self.bad == "forwarder" {
                    "0xaa".to_string()
                } else {
                    format!("0x{}", hex::encode(&self.runtime))
                }
            } else if self.bad == "implementation" {
                "0xbb".to_string()
            } else if self.bad == "empty" {
                "0x".to_string()
            } else {
                "0xaa".to_string()
            };
            Ok(json!({"id":1,"result":code}))
        }
    }
    qualify_runtime(
        &CloneRpc {
            runtime: proxy.runtime(),
            bad: "",
        },
        1,
        2,
        &l,
    )
    .unwrap();
    for bad in ["forwarder", "implementation", "empty"] {
        assert!(qualify_runtime(&CloneRpc { runtime: proxy.runtime(), bad }, 1, 2, &l).is_err(), "{bad}");
    }
}
#[test]
fn predeployment_calls_are_unavailable_only_for_qualified_empty_successes() {
    use crate::survey::unavailable_before_deployment;
    let mut l = erc20_balances_storage::layout::parse(include_str!("../../tests/fixtures/bsc-clone-layouts.json")).unwrap();
    let layout = l.last_mut().unwrap();
    let height = layout.deployment.as_ref().unwrap().block;
    let empty = json!({"id":1,"result":"0x"});
    assert!(unavailable_before_deployment(layout, height, "before", &empty));
    for (block, boundary, response) in [
        (height, "after", empty.clone()),
        (height - 1, "before", empty.clone()),
        (height, "before", json!({"result":"0x","error":{"code":-1}})),
        (height, "before", json!({"result":format!("0x{}","00".repeat(32))})),
    ] {
        assert!(!unavailable_before_deployment(layout, block, boundary, &response));
    }
    layout.deployment = None;
    assert!(!unavailable_before_deployment(layout, height, "before", &empty));
}

#[test]
fn zero_path_probe_uses_postdeployment_nonzero_holders() {
    use crate::inspect_ranked::observe_probe;
    let mut probes = std::collections::BTreeMap::new();
    let contracts = std::collections::BTreeSet::from([TOKEN.to_string()]);
    for (boundary, holder, storage, rpc) in [
        ("before", address(), "7", Some("7")),
        ("after", format!("0x{}", "00".repeat(20)), "7", Some("7")),
    ] {
        observe_probe(
            &mut probes,
            &contracts,
            json!({"contract":TOKEN,"address":holder,"boundary":boundary,"storage":storage,"rpc":rpc}),
        )
        .unwrap();
    }
    assert!(probes.is_empty());
    for storage in ["0", "7", "0"] {
        observe_probe(
            &mut probes,
            &contracts,
            json!({"contract":TOKEN,"address":address(),"boundary":"after","storage":storage,"rpc":null}),
        )
        .unwrap();
    }
    assert_eq!(probes[TOKEN]["storage"], "7");
}

#[test]
fn zero_path_controls_distinguish_fallback_and_other_computed_balances() {
    let mut report = json!({"status":"inspected","read_only_state_overrides":[{"overridden_mapping_word":"0","balance_of":"0"},{"overridden_mapping_word":"1","balance_of":"1"},{"overridden_mapping_word":"123","balance_of":"123"},{"overridden_mapping_word":"115792089237316195423570985008687907853269984665640564039457584007913129639935","balance_of":"115792089237316195423570985008687907853269984665640564039457584007913129639935"}]});
    assert_eq!(crate::inspect_ranked::control_result(&report), "direct_word_controls_match_not_qualified");
    report["read_only_state_overrides"][0]["balance_of"] = json!("8");
    assert_eq!(crate::inspect_ranked::control_result(&report), "zero_word_fallback");
    report["read_only_state_overrides"][1]["balance_of"] = json!("2");
    assert_eq!(crate::inspect_ranked::control_result(&report), "nonzero_word_transform");
    report["status"] = json!("incomplete");
    assert_eq!(crate::inspect_ranked::control_result(&report), "incomplete");
    report["status"] = json!("inspected");
    report["read_only_state_overrides"][0] = json!({});
    assert_eq!(crate::inspect_ranked::control_result(&report), "incomplete");
}

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
fn runtime_qualification_checks_divisor_at_both_boundaries() {
    struct DivisorRpc {
        mismatch_on: usize,
        reads: std::sync::atomic::AtomicUsize,
    }
    impl Rpc for DivisorRpc {
        fn request(&self, payload: Value) -> Result<Value> {
            let result = match payload["method"].as_str().unwrap() {
                "eth_getCode" => json!("0xaa"),
                "eth_getStorageAt" => {
                    let n = self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    assert_eq!(payload["params"][1], hash(4));
                    assert_eq!(payload["params"][2]["requireCanonical"], true);
                    json!(hash(if self.mismatch_on == n { 9 } else { 8 }))
                }
                _ => return FakeRpc::default().request(payload),
            };
            Ok(json!({"id":1,"result":result}))
        }
    }
    let params = json!([{"contract":TOKEN,"balance_slot":hash(7),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&[0xaa]))),"balance_divisor":{"value":hash(8),"storage_slot":hash(4)}}]);
    let layouts = erc20_balances_storage::layout::parse(&params.to_string()).unwrap();
    for mismatch_on in [0, 1, 2] {
        let rpc = DivisorRpc { mismatch_on, reads: 0.into() };
        let result = qualify_runtime(&rpc, 1, 3, &layouts);
        if mismatch_on < 2 {
            assert!(result.unwrap_err().to_string().contains("divisor dependency value"));
        } else {
            result.unwrap();
            assert_eq!(rpc.reads.load(std::sync::atomic::Ordering::SeqCst), 2);
        }
    }
}
#[test]
fn beacon_runtime_qualification_checks_both_pointers_code_and_getter() {
    struct BeaconRpc {
        bad: &'static str,
    }
    impl Rpc for BeaconRpc {
        fn request(&self, payload: Value) -> Result<Value> {
            let token = TOKEN;
            let beacon = format!("0x{}", "bb".repeat(20));
            let implementation = format!("0x{}", "cc".repeat(20));
            let word = |a: &str| format!("0x{}{}", "00".repeat(12), &a[2..]);
            let address = payload["params"][0].as_str().unwrap_or("");
            let result = match payload["method"].as_str().unwrap() {
                "eth_getCode" => {
                    if address == token {
                        json!("0xaa")
                    } else if address == beacon {
                        json!(if self.bad == "code" { "0xdd" } else { "0xbb" })
                    } else {
                        json!("0xcc")
                    }
                }
                "eth_getStorageAt" => {
                    if self.bad == "pointer" && address == token || self.bad == "implementation" && address == beacon {
                        json!(hash(0))
                    } else {
                        json!(word(if address == token { &beacon } else { &implementation }))
                    }
                }
                "eth_call" => json!(if self.bad == "getter" { hash(0) } else { word(&implementation) }),
                _ => return FakeRpc::default().request(payload),
            };
            Ok(json!({"id":1,"result":result}))
        }
    }
    let hash_code = |b| format!("0x{}", hex::encode(erc20_balances_storage::hash(&[b])));
    let params = json!([{"contract":TOKEN,"balance_slot":hash(7),"code_hash":hash_code(0xaa),"beacon_proxy":{"beacon_slot":hash(99),"beacon":format!("0x{}","bb".repeat(20)),"beacon_code_hash":hash_code(0xbb),"implementation_slot":hash(1),"implementation":format!("0x{}","cc".repeat(20)),"implementation_code_hash":hash_code(0xcc)}}]);
    let layouts = erc20_balances_storage::layout::parse(&params.to_string()).unwrap();
    qualify_runtime(&BeaconRpc { bad: "" }, 1, 2, &layouts).unwrap();
    for bad in ["pointer", "implementation", "code", "getter"] {
        assert!(qualify_runtime(&BeaconRpc { bad }, 1, 2, &layouts).is_err(), "{bad}");
    }
}

#[test]
fn proxied_beacon_qualification_pins_its_delegate_at_both_boundaries() {
    struct DelegateRpc {
        bad: &'static str,
        at_read: usize,
        reads: std::sync::atomic::AtomicUsize,
    }
    impl Rpc for DelegateRpc {
        fn request(&self, p: Value) -> Result<Value> {
            let beacon = format!("0x{}", "bb".repeat(20));
            let implementation = format!("0x{}", "cc".repeat(20));
            let delegate = format!("0x{}", "dd".repeat(20));
            let word = |a: &str| format!("0x{}{}", "00".repeat(12), &a[2..]);
            let address = p["params"][0].as_str().unwrap_or("");
            let n = self.reads.load(std::sync::atomic::Ordering::SeqCst);
            let result = match p["method"].as_str().unwrap() {
                "eth_getStorageAt" if address == beacon && p["params"][1] == hash(98) => {
                    let read = self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    assert_eq!(p["params"][2]["requireCanonical"], true);
                    json!(if self.bad == "pointer" && read == self.at_read {
                        hash(0)
                    } else {
                        word(&delegate)
                    })
                }
                "eth_getStorageAt" => json!(word(if address == TOKEN { &beacon } else { &implementation })),
                "eth_getCode" => json!(if address == TOKEN {
                    "0xaa"
                } else if address == beacon {
                    "0xbb"
                } else if address == implementation {
                    "0xcc"
                } else if self.bad == "code" && n == self.at_read + 1 {
                    "0xee"
                } else {
                    "0xdd"
                }),
                "eth_call" => json!(word(&implementation)),
                _ => return FakeRpc::default().request(p),
            };
            Ok(json!({"id":1,"result":result}))
        }
    }
    let code = |b| format!("0x{}", hex::encode(erc20_balances_storage::hash(&[b])));
    let p = json!([{"contract":TOKEN,"balance_slot":hash(7),"code_hash":code(0xaa),"beacon_proxy":{"beacon_slot":hash(99),"beacon":format!("0x{}","bb".repeat(20)),"beacon_code_hash":code(0xbb),"implementation_slot":hash(1),"implementation":format!("0x{}","cc".repeat(20)),"implementation_code_hash":code(0xcc),"proxy":{"implementation_slot":hash(98),"implementation":format!("0x{}","dd".repeat(20)),"code_hash":code(0xdd)}}}]);
    let layouts = erc20_balances_storage::layout::parse(&p.to_string()).unwrap();
    for at_read in [0, 1] {
        for bad in ["pointer", "code", ""] {
            let rpc = DelegateRpc { bad, at_read, reads: 0.into() };
            let result = qualify_runtime(&rpc, 1, 3, &layouts);
            if bad.is_empty() {
                result.unwrap();
                assert_eq!(rpc.reads.load(std::sync::atomic::Ordering::SeqCst), 2);
            } else {
                assert!(result.unwrap_err().to_string().contains("beacon proxy implementation"));
            }
        }
    }
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
fn sparse_events_require_complete_delivery_and_canonical_clocks_before_empty_normalization() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("events.jsonl");
    let clock_path = temp.path().join("clocks.txt");
    let row = format!(
        "{}\n",
        json!({"@module":"map_events","@type":"evm.balances.v1.Events","@block":2,"@data":reference("5")})
    );
    fs::write(&path, &row).unwrap();
    fs::write(
        &clock_path,
        (1..4)
            .map(|n| format!("----------- BLOCK #{n} ({}) age=1s ---------------\n", hash(n)))
            .collect::<String>(),
    )
    .unwrap();
    assert!(read_stream(&path, 1, 4, "map_events").is_err());
    assert!(crate::capture::confirm_empty_outputs(&FakeRpc::default(), &path, &clock_path, 1, 4, 2).is_err());
    assert_eq!(fs::read_to_string(&path).unwrap(), row);
    let receipt = crate::capture::confirm_empty_outputs(&FakeRpc::default(), &path, &clock_path, 1, 4, 3).unwrap();
    assert_eq!(receipt["empty_blocks"], 2);
    let complete = read_stream(&path, 1, 4, "map_events").unwrap();
    assert!(candidate_rows(&complete[&1]).unwrap().is_empty());
    assert!(candidate_rows(&complete[&3]).unwrap().is_empty());
    assert_eq!(complete[&2], reference("5"));
    assert_eq!(fs::read_to_string(path.with_extension("sparse.jsonl")).unwrap(), row);
}
#[test]
fn delivered_clocks_reject_gaps_duplicates_partial_blocks_bad_hashes_and_forks() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("clocks.txt");
    let line = |n| format!("----------- BLOCK #{n} ({}) age=1s ---------------\n", hash(n));
    for bad in [
        line(1),
        format!("{}{}", line(1), line(1)),
        format!("{}{}", line(1), line(3)),
        line(1).replace("BLOCK #", "PARTIAL BLOCK #"),
        line(1).replace(&hash(1), "bad"),
        "UNDO: 1\n".into(),
    ] {
        fs::write(&path, bad).unwrap();
        assert!(crate::capture::read_clocks(&path, 1, 3).is_err());
    }
    fs::write(&path, format!("{}{}", line(1), line(2))).unwrap();
    let mut clocks = crate::capture::read_clocks(&path, 1, 3).unwrap();
    crate::capture::verify_clocks(&FakeRpc::default(), &clocks).unwrap();
    clocks.insert(2, hash(99));
    assert!(crate::capture::verify_clocks(&FakeRpc::default(), &clocks).is_err());
    struct ForkRpc;
    impl Rpc for ForkRpc {
        fn request(&self, value: Value) -> Result<Value> {
            let mut response = FakeRpc::default().request(value)?;
            response["result"]["parentHash"] = json!(hash(99));
            Ok(response)
        }
    }
    clocks.insert(2, hash(2));
    assert!(crate::capture::verify_clocks(&ForkRpc, &clocks).is_err());
}
#[test]
fn capture_delivery_receipt_rejects_missing_or_ambiguous_counts() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("capture.log");
    fs::write(&path, " • Received Blocks: 1,024 blocks\nCompleted successfully\n").unwrap();
    assert_eq!(crate::capture::received_blocks(&path).unwrap(), 1024);
    for bad in [
        "Completed successfully\n",
        " • Received Blocks: unknown blocks\n",
        " • Received Blocks: 1 blocks\n • Received Blocks: 1 blocks\n",
    ] {
        fs::write(&path, bad).unwrap();
        assert!(crate::capture::received_blocks(&path).is_err());
    }
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
        io::{BufRead, BufReader, Read, Write},
        net::TcpListener,
        thread,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/private-endpoint", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        for _ in 0..16 {
            let (mut socket, _) = listener.accept().unwrap();
            socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
            {
                // TCP can split headers and body across reads. Closing with unread
                // request bytes can reset the socket before the 403 reaches ureq.
                let mut request = BufReader::new(&mut socket);
                let mut content_length = None;
                loop {
                    let mut line = String::new();
                    assert!(request.read_line(&mut line).unwrap() > 0);
                    if line == "\r\n" {
                        break;
                    }
                    if let Some((name, value)) = line.split_once(':') {
                        if name.eq_ignore_ascii_case("content-length") {
                            content_length = Some(value.trim().parse::<usize>().unwrap());
                        }
                    }
                }
                assert_eq!(content_length, Some(2));
                let mut body = [0; 2];
                request.read_exact(&mut body).unwrap();
                assert_eq!(&body, b"{}");
            }
            socket
                .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .unwrap();
        }
    });
    let rpc = HttpRpc::new(endpoint, Some("private-api-key".into()));
    let errors = (0..16).map(|_| rpc.request(json!({})).unwrap_err().to_string()).collect::<Vec<_>>();
    handle.join().unwrap();
    assert!(errors.iter().all(|error| error == "RPC HTTP 403"));
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
fn deployment_runtime_qualification_checks_birth_even_after_the_window_start() {
    struct CreationRpc {
        bad: &'static str,
        calls: Mutex<Vec<Value>>,
    }
    impl Rpc for CreationRpc {
        fn request(&self, payload: Value) -> Result<Value> {
            self.calls.lock().unwrap().push(payload.clone());
            let before = payload["params"][1]["blockHash"] == hash(9);
            let result = match payload["method"].as_str().unwrap() {
                "eth_getCode" if before => json!(if self.bad == "early_code" { "0xaa" } else { "0x" }),
                "eth_getCode" => json!(if self.bad == "wrong_runtime" { "0xbb" } else { "0xaa" }),
                "eth_getTransactionCount" => json!(if self.bad == "early_nonce" { "0x1" } else { "0x0" }),
                _ => return FakeRpc::default().request(payload),
            };
            Ok(json!({"id":1,"result":result}))
        }
    }
    let params = json!([{"contract":TOKEN,"balance_slot":hash(0),"code_hash":format!("0x{}",hex::encode(erc20_balances_storage::hash(&[0xaa]))),"deployment":{"block":10,"block_hash":hash(10)}}]);
    let mut layouts = erc20_balances_storage::layout::parse(&params.to_string()).unwrap();
    let rpc = CreationRpc {
        bad: "",
        calls: Mutex::new(vec![]),
    };
    qualify_runtime(&rpc, 11, 13, &layouts).unwrap();
    let calls = rpc.calls.lock().unwrap();
    assert!(calls.iter().any(|p| p["method"] == "eth_getCode" && p["params"][1] == block_ref(&hash(9))));
    assert!(calls.iter().all(|p| p["method"] != "eth_call"));
    drop(calls);
    for bad in ["early_code", "wrong_runtime", "early_nonce"] {
        assert!(
            qualify_runtime(
                &CreationRpc {
                    bad,
                    calls: Mutex::new(vec![])
                },
                10,
                13,
                &layouts
            )
            .is_err(),
            "{bad}"
        );
    }
    layouts[0].deployment.as_mut().unwrap().block_hash = [42; 32];
    assert!(qualify_runtime(&rpc, 11, 13, &layouts).is_err());
}

#[test]
fn deployment_holder_baseline_is_distinct_from_an_rpc_checkpoint() {
    use crate::coverage::HolderState;
    let minted = (TOKEN.to_owned(), address());
    let untouched = (TOKEN.to_owned(), format!("0x{}", "23".repeat(20)));
    let other = (format!("0x{}", "99".repeat(20)), address());
    let holders = [minted.clone(), untouched.clone(), other.clone()].into();
    let mut state = HolderState::default();
    assert!(state.seeded.is_empty());
    assert_eq!(state.initialize_deployed_token(TOKEN, &holders).unwrap(), 2);
    let emitted = [(minted.clone(), U256::from(100))].into();
    let reference = [(minted.clone(), U256::from(100)), (untouched.clone(), U256::zero())].into();
    state.apply(10, &hash(10), &hash(9), &emitted, &reference).unwrap();
    state.apply(11, &hash(11), &hash(10), &Balances::new(), &reference).unwrap();
    assert_eq!(state.tokens[TOKEN]["seeded_matches"], 4);
    assert_eq!(state.tokens[TOKEN]["unseeded_unknown_rows"], 0);
    assert_eq!(state.observed[&minted], U256::from(100));
    assert!(!state.seeded.contains_key(&other));
    assert!(state.initialize_deployed_token(TOKEN, &holders).is_err());
}

#[test]
fn inactive_holder_tokens_are_coverage_gaps_not_balance_mismatches() {
    use crate::coverage::outcome;
    let contracts = [TOKEN.to_string(), address()].into();
    let mut tokens = std::collections::BTreeMap::from([(
        TOKEN.to_string(),
        json!({"reference_rows":33,"seeded_value_mismatches":0,"seeded_unknown_rows":0}),
    )]);
    assert_eq!(outcome(&tokens, &contracts), ("coverage_gap", vec![address()]));
    tokens.insert(address(), tokens[TOKEN].clone());
    assert_eq!(outcome(&tokens, &contracts), ("bounded_parity", vec![]));
    tokens.get_mut(TOKEN).unwrap()["seeded_unknown_rows"] = json!(1);
    assert_eq!(outcome(&tokens, &contracts).0, "mismatch");
    tokens.get_mut(TOKEN).unwrap()["seeded_unknown_rows"] = json!(0);
    tokens.get_mut(TOKEN).unwrap()["seeded_value_mismatches"] = json!(1);
    assert_eq!(outcome(&tokens, &contracts).0, "mismatch");
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

#[test]
fn immutable_zero_checkpoint_starts_only_after_qualified_creation() {
    use crate::coverage::known_checkpoint_amount;
    let mut layouts = erc20_balances_storage::layout::parse(include_str!("../../tests/fixtures/immutable-zero-layout.json")).unwrap();
    let birth = layouts[0].deployment.as_ref().unwrap().block;
    for start in [birth - 1, birth] {
        assert_eq!(known_checkpoint_amount(&layouts[0], start, &[7; 20]), None);
    }
    assert_eq!(known_checkpoint_amount(&layouts[0], birth + 1, &[7; 20]), Some("0".into()));
    layouts[0].immutable_zero_mapping = false;
    assert_eq!(known_checkpoint_amount(&layouts[0], birth + 1, &[7; 20]), None);
}
