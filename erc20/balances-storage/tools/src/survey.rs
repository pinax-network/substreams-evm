//! Diagnostic only: discover from earlier active blocks, test later active blocks,
//! and keep unknown rows and strict mapper failures visible. No layout promotion.
use crate::{cli::record_run, data::*, rpc::*};
use anyhow::{ensure, Context, Result};
use clap::Args;
use erc20_balances_storage::{discovery, layout::VerifiedLayout};
use prost::Message;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use substreams_ethereum::pb::eth::v2 as eth;

#[derive(Args)]
pub struct Survey {
    #[arg(long)]
    pub ranking: PathBuf,
    #[arg(long, required = true)]
    pub block_dir: Vec<PathBuf>,
    /// Optional existing reviewed layouts, e.g. the WBNB regression fixture.
    /// Unconfigured tokens still use explicitly unqualified diagnostic inputs.
    #[arg(long)]
    pub layouts: Option<PathBuf>,
    /// Retest only these contracts from the ranking; repeat to select several.
    #[arg(long = "contract")]
    pub contracts: Vec<String>,
    #[arg(long)]
    pub output: PathBuf,
}

pub fn select_tokens<'a>(ranked: &'a [Value], requested: &[String]) -> Result<Vec<&'a Value>> {
    let requested = requested.iter().map(|c| binary(&json!(c), 20)).collect::<Result<BTreeSet<_>>>()?;
    let mut missing = requested.clone();
    let mut selected = Vec::new();
    for token in ranked {
        let contract = binary(&token["contract"], 20)?;
        if requested.is_empty() || requested.contains(&contract) {
            missing.remove(&contract);
            selected.push(token);
        }
    }
    ensure!(missing.is_empty(), "requested contract is not in the selected ranking");
    Ok(selected)
}

#[derive(Default)]
pub struct Metrics {
    pub reference: u64,
    pub storage: u64,
    pub shared: u64,
    pub mismatches: u64,
    pub missing: u64,
    pub extra: u64,
}
impl Metrics {
    pub fn observe(&mut self, storage: &Balances, reference: &Balances) {
        self.storage += storage.len() as u64;
        self.reference += reference.len() as u64;
        for (key, value) in storage {
            match reference.get(key) {
                Some(other) => {
                    self.shared += 1;
                    self.mismatches += u64::from(value != other);
                }
                None => self.extra += 1,
            }
        }
        self.missing += reference.keys().filter(|k| !storage.contains_key(*k)).count() as u64;
    }
    pub fn json(&self) -> Value {
        json!({"reference_rows":self.reference,"storage_rows":self.storage,"shared_rows":self.shared,
            "value_mismatches":self.mismatches,"reference_only_rows":self.missing,"storage_only_rows":self.extra,
            "exact_row_parity":self.reference > 0 && self.missing == 0 && self.extra == 0 && self.mismatches == 0})
    }
}

/// Completing the investigation is not passing the parity gate.
pub fn parity_status(tokens: &[Value]) -> &'static str {
    if tokens.is_empty() || tokens.iter().any(|t| t["status"] == "insufficient_samples") {
        "insufficient_samples"
    } else if tokens.iter().any(|t| t["status"] == "rpc_unresolved") {
        "rpc_unresolved"
    } else if tokens.iter().any(|t| t["status"] == "value_mismatch") {
        "mismatch"
    } else if tokens.iter().all(|t| t["exact_mapper_parity"] == true) {
        "bounded_parity"
    } else {
        "coverage_gap"
    }
}

#[derive(Default)]
struct Hypothesis {
    matched: u64,
    mismatched: u64,
    nonzero_holders: BTreeSet<String>,
}

fn token_rows(block: &Value, contract: &str) -> Result<Balances> {
    Ok(candidate_rows(block)?.into_iter().filter(|(key, _)| key.0 == contract).collect())
}
fn selected_rows(discovery: &Value, contract: &str, slot: &str, layout: &VerifiedLayout) -> Result<Balances> {
    items(discovery, "candidates")?
        .iter()
        .filter(|r| r["contract"] == contract && r["mappingSlot"] == slot && r["address"] != "0x0000000000000000000000000000000000000000")
        .map(|r| {
            Ok((
                (contract.to_string(), binary(&r["address"], 20)?),
                uint(&json!(layout.project_amount(text(&r["amount"])?)))?,
            ))
        })
        .collect()
}

pub fn mapping_key(address: &str, slot: &str) -> Result<String> {
    let address = binary(&json!(address), 20)?;
    let slot = binary(&json!(slot), 32)?;
    let mut preimage = [0; 64];
    preimage[12..32].copy_from_slice(&hex::decode(&address[2..])?);
    preimage[32..].copy_from_slice(&hex::decode(&slot[2..])?);
    Ok(format!("0x{}", hex::encode(erc20_balances_storage::hash(&preimage))))
}

/// Token metadata is display-only; contract address always identifies a result.
fn symbol(rpc: &dyn Rpc, contract: &str, digest: &str) -> Option<String> {
    let value = rpc.call("eth_call", json!([{"to":contract,"data":"0x95d89b41"},block_ref(digest)])).ok()?;
    let bytes = hex::decode(value.as_str()?.strip_prefix("0x")?).ok()?;
    let raw = if bytes.len() == 32 {
        bytes.as_slice()
    } else {
        if bytes.len() < 64 || bytes[..31] != [0; 31] || bytes[31] != 32 || bytes[32..63] != [0; 31] {
            return None;
        }
        bytes.get(64..64 + bytes[63] as usize)?
    };
    let value = std::str::from_utf8(raw).ok()?.trim_end_matches('\0');
    if value.is_empty() || value.chars().any(char::is_control) {
        return None;
    }
    Some(value.chars().take(80).collect())
}

pub fn run(args: Survey) -> Result<bool> {
    record_run(
        &args.output,
        json!({"status":"incomplete","promoted_layouts":0,"tokens":[],
        "scope":"Exploratory native tests on sampled active blocks; hypotheses are not verified layouts; no cross-gap carry-forward"}),
        |report| {
            let ranking: Value = serde_json::from_slice(&fs::read(&args.ranking)?)?;
            ensure!(ranking["status"] == "ranked" && ranking["chain_id"] == 56, "completed BSC ranking required");
            let selected = select_tokens(items(&ranking, "selected_tokens")?, &args.contracts)?;
            report["selected_contracts"] = json!(selected.iter().map(|t| &t["contract"]).collect::<Vec<_>>());
            let start = number(&ranking["start"])?;
            let stop = number(&ranking["stop_exclusive"])?;
            let reference_path = args.ranking.parent().context("ranking parent")?.join("reference.jsonl");
            ensure!(sha256(&reference_path)? == text(&ranking["reference_sha256"])?, "reference digest changed");
            let reference = read_stream(&reference_path, start, stop, "map_events")?;
            let mut blocks = BTreeMap::<u64, eth::Block>::new();
            let mut captured = Vec::new();
            for dir in &args.block_dir {
                for entry in fs::read_dir(dir)? {
                    let path = entry?.path();
                    if path.extension().is_none_or(|e| e != "pb") {
                        continue;
                    }
                    let block = eth::Block::decode(fs::read(&path)?.as_slice())?;
                    ensure!((start..stop).contains(&block.number), "block outside ranked range");
                    if let Some(previous) = blocks.get(&block.number) {
                        ensure!(previous == &block, "conflicting duplicate block");
                    } else {
                        captured.push(json!({"block":block.number,"sha256":sha256(&path)?}));
                        blocks.insert(block.number, block);
                    }
                }
            }
            ensure!(!blocks.is_empty(), "no Extended blocks");
            let rpc = HttpRpc::from_env();
            let reviewed = if let Some(path) = &args.layouts {
                report["reviewed_layouts_sha256"] = json!(sha256(path)?);
                let layouts = erc20_balances_storage::layout::parse(&fs::read_to_string(path)?)?;
                qualify_runtime(&rpc, start, stop, &layouts)?;
                layouts
            } else {
                Vec::new()
            };
            ensure_finalized(&rpc, stop)?;
            ensure!(
                rpc.header(start)?["hash"] == ranking["first_hash"] && rpc.header(stop - 1)?["hash"] == ranking["last_hash"],
                "ranking boundary changed"
            );
            let mut discovered = Blocks::new();
            for (height, block) in &blocks {
                let header = rpc.header(*height)?;
                ensure!(
                    binary(&header["hash"], 32)? == format!("0x{}", hex::encode(&block.hash)),
                    "captured block differs from RPC"
                );
                ensure!(
                    binary(&header["parentHash"], 32)? == format!("0x{}", hex::encode(&block.header.as_ref().context("missing header")?.parent_hash)),
                    "captured parent differs from RPC"
                );
                discovered.insert(*height, discovery::project(block)?.into_json());
            }
            report["ranking_sha256"] = json!(sha256(&args.ranking)?);
            report["reference_sha256"] = ranking["reference_sha256"].clone();
            report["ranked_blocks"] = json!(stop - start);
            report["sampled_blocks"] = json!(blocks.len());
            report["captured"] = json!(captured);
            write_report(&args.output, report)?;
            let mut checks = File::create(args.output.join("rpc-checks.jsonl"))?;
            let mut observations = File::create(args.output.join("observations.jsonl"))?;
            for token in selected {
                let contract = text(&token["contract"])?;
                let known = reviewed.iter().find(|l| format!("0x{}", hex::encode(&l.contract)) == contract);
                let active = blocks
                    .keys()
                    .copied()
                    .filter(|h| {
                        reference[h]["balances"]
                            .as_array()
                            .is_some_and(|rows| rows.iter().any(|r| binary(&r["contract"], 20).ok().as_deref() == Some(contract)))
                    })
                    .collect::<Vec<_>>();
                let mut result = json!({"rank":token["rank"],"contract":contract,"ranked_reference_rows":token["reference_rows"],"sampled_active_blocks":active.len(),"status":"insufficient_samples","exact_mapper_parity":false});
                if active.len() < 2 {
                    report["tokens"].as_array_mut().unwrap().push(result);
                    continue;
                }
                let cutoff = active[active.len() / 2];
                result["holdout_start"] = json!(cutoff);
                let mut hypotheses = BTreeMap::<String, Hypothesis>::new();
                for height in active.iter().filter(|h| **h < cutoff) {
                    let expected = token_rows(&reference[height], contract)?;
                    for row in items(&discovered[height], "candidates")?.iter().filter(|r| r["contract"] == contract) {
                        let address = binary(&row["address"], 20)?;
                        if let Some(value) = expected.get(&(contract.to_string(), address.clone())) {
                            let h = hypotheses.entry(binary(&row["mappingSlot"], 32)?).or_default();
                            if *value == uint(&row["amount"])? {
                                h.matched += 1;
                                if !value.is_zero() || !uint(&row["oldAmount"])?.is_zero() {
                                    h.nonzero_holders.insert(address);
                                }
                            } else {
                                h.mismatched += 1;
                            }
                        }
                    }
                }
                result["hypotheses"] = json!(hypotheses
                    .iter()
                    .map(|(slot, h)| json!({"slot":slot,"matched":h.matched,"mismatched":h.mismatched,"nonzero_holders":h.nonzero_holders.len()}))
                    .collect::<Vec<_>>());
                let matching = hypotheses
                    .iter()
                    .filter(|(_, h)| h.mismatched == 0 && h.matched >= 2 && h.nonzero_holders.len() >= 2)
                    .collect::<Vec<_>>();
                let first = discovered.first_key_value().unwrap().1;
                let last = discovered.last_key_value().unwrap().1;
                let digest = text(&last["hash"])?;
                result["symbol"] = json!(symbol(&rpc, contract, digest));
                let mut codes = Vec::new();
                for boundary in [&first["parentHash"], &last["hash"]] {
                    let code = rpc.call("eth_getCode", json!([contract, block_ref(text(boundary)?)]))?;
                    let bytes = hex::decode(text(&code)?.strip_prefix("0x").context("runtime hex")?)?;
                    codes.push(format!("0x{}", hex::encode(erc20_balances_storage::hash(&bytes))));
                }
                result["runtime_hashes"] = json!(codes);
                result["runtime_stable_at_boundaries"] = json!(codes[0] == codes[1]);
                if matching.len() != 1 && known.is_none() {
                    result["status"] = json!(if matching.is_empty() {
                        "no_direct_mapping_evidence"
                    } else {
                        "ambiguous_layout"
                    });
                    result["sampled_reference_rows"] = json!(active
                        .iter()
                        .map(|h| token_rows(&reference[h], contract).map(|r| r.len()))
                        .collect::<Result<Vec<_>>>()?
                        .iter()
                        .sum::<usize>());
                    report["tokens"].as_array_mut().unwrap().push(result);
                    write_report(&args.output, report)?;
                    continue;
                }
                // Reviewed transforms are supplied independently of discovery.
                // A raw direct-mapping hypothesis must never erase their zero rule.
                let configured_slot = known.map(|l| format!("0x{}", hex::encode(l.balance_slot)));
                let slot = configured_slot.as_ref().unwrap_or_else(|| matching[0].0);
                result["candidate_balance_slot"] = json!(slot);
                // No unknown write is automatically declared harmless. The mapper
                // runs with an empty ignore list; errors remain distinct from no rows.
                let mut layout = VerifiedLayout {
                    contract: hex::decode(&contract[2..])?,
                    balance_slot: hex::decode(&slot[2..])?.try_into().unwrap(),
                    code_hash: hex::decode(&codes[0][2..])?.try_into().unwrap(),
                    deployment: None,
                    other_slots: BTreeSet::new(),
                    other_mapping_slots: BTreeSet::new(),
                    other_mapping_words: BTreeMap::new(),
                    zero_balance: None,
                    proxy: None,
                    beacon_proxy: None,
                };
                result["mapper_configuration"] = json!("unqualified balance-slot hypothesis; empty ignore lists");
                if let Some(known) = known {
                    ensure!(
                        known.balance_slot == layout.balance_slot && known.code_hash == layout.code_hash,
                        "reviewed layout differs from observed candidate"
                    );
                    layout = known.clone();
                    result["mapper_configuration"] = json!("caller-supplied reviewed layout");
                }
                let (mut discovery_metrics, mut holdout_metrics, mut mapper_metrics) = (Metrics::default(), Metrics::default(), Metrics::default());
                let mut errors = Vec::new();
                let mut value_checks = 0_u64;
                let mut rpc_mismatches = 0_u64;
                let mut rpc_errors = 0_u64;
                // Reference-silent blocks still matter: the strict mapper can emit
                // a write without a Transfer log. Such extra rows must fail parity.
                result["evaluated_blocks"] = json!(blocks.len());
                for height in blocks.keys() {
                    let d = &discovered[height];
                    let expected = token_rows(&reference[height], contract)?;
                    let proposed = selected_rows(d, contract, slot, &layout)?;
                    if *height < cutoff {
                        discovery_metrics.observe(&proposed, &expected);
                    } else {
                        holdout_metrics.observe(&proposed, &expected);
                    }
                    let mapped = match erc20_balances_storage::project(&blocks[height], std::slice::from_ref(&layout)) {
                        Ok(events) => {
                            let emitted = events
                                .balances
                                .into_iter()
                                .map(|r| {
                                    json!({"contract":r.contract.map(|c|format!("0x{}",hex::encode(c))),
                                "address":format!("0x{}",hex::encode(r.address)),"amount":r.amount})
                                })
                                .collect::<Vec<_>>();
                            // Preserve the emitted identity, and apply the same
                            // address/amount/duplicate checks as streamed Events.
                            let rows = candidate_rows(&json!({"balances":emitted}))?;
                            if *height >= cutoff {
                                mapper_metrics.observe(&rows, &expected);
                            }
                            Some(rows)
                        }
                        Err(e) => {
                            errors.push(json!({"block":height,"error":e.to_string(),"holdout":*height>=cutoff}));
                            None
                        }
                    };
                    for (side, rows) in [("reference", &expected), ("hypothesis", &proposed)] {
                        for ((_, address), amount) in rows {
                            writeln!(
                                observations,
                                "{}",
                                json!({"block":height,"hash":d["hash"],"contract":contract,"address":address,"amount":amount.to_string(),"side":side,"holdout":*height>=cutoff})
                            )?;
                        }
                    }
                    if let Some(rows) = mapped {
                        for ((emitted_contract, address), amount) in rows {
                            writeln!(
                                observations,
                                "{}",
                                json!({"block":height,"hash":d["hash"],"contract":emitted_contract,"address":address,"amount":amount.to_string(),"side":"strict_mapper","holdout":*height>=cutoff})
                            )?;
                        }
                    }
                    let mut requests = Vec::new();
                    let mut evidence = Vec::new();
                    for row in items(d, "candidates")?
                        .iter()
                        .filter(|r| r["contract"] == contract && r["mappingSlot"] == *slot && r["address"] != "0x0000000000000000000000000000000000000000")
                    {
                        for (boundary, hash, amount) in [("before", &d["parentHash"], &row["oldAmount"]), ("after", &d["hash"], &row["amount"])] {
                            requests.push(balance_request(contract, text(&row["address"])?, block_ref(text(hash)?)));
                            evidence.push(json!({"block":height,"hash":hash,"contract":contract,"address":row["address"],"boundary":boundary,"storage":amount,"holdout":*height>=cutoff}));
                        }
                    }
                    for (calls, rows) in requests.chunks(25).zip(evidence.chunks_mut(25)) {
                        for (row, response) in rows.iter_mut().zip(rpc.batch_rows(calls)?) {
                            let actual = if response["error"].is_null() {
                                balance_result(&response["result"], true).ok()
                            } else {
                                None
                            };
                            let raw = uint(&row["storage"])?;
                            let expected = uint(&json!(layout.project_amount(&raw.to_string())))?;
                            row["projected"] = json!(expected.to_string());
                            value_checks += 1;
                            rpc_errors += u64::from(actual.is_none());
                            rpc_mismatches += u64::from(actual.is_some_and(|v| v != expected));
                            row["rpc"] = json!(actual.map(|v| v.to_string()));
                            row["match"] = json!(actual == Some(expected));
                            if actual.is_none() {
                                // Retain empty/malformed ABI data separately from
                                // provider errors instead of collapsing both to null.
                                row["rpc_response"] = response.clone();
                            }
                            if actual.is_some_and(|v| v != expected) {
                                let key = mapping_key(text(&row["address"])?, slot)?;
                                let stored = rpc.call("eth_getStorageAt", json!([contract, key, block_ref(text(&row["hash"])?)]))?;
                                row["rpc_storage_word"] = json!(quantity(&stored)?.to_string());
                                row["storage_key"] = json!(key);
                                row["rpc_storage_matches_firehose"] = json!(quantity(&stored)? == raw);
                            }
                            writeln!(checks, "{row}")?;
                        }
                    }
                    checks.flush()?;
                    observations.flush()?;
                    if !requests.is_empty() {
                        ensure!(rpc.header(*height)?["hash"] == d["hash"], "survey RPC header changed");
                    }
                }
                result["discovery"] = discovery_metrics.json();
                result["holdout"] = holdout_metrics.json();
                result["strict_mapper_successful_holdout_blocks_only"] = mapper_metrics.json();
                result["strict_mapper_errors"] = json!(errors);
                result["rpc_value_checks"] = json!(value_checks);
                result["rpc_mismatches"] = json!(rpc_mismatches);
                result["rpc_errors"] = json!(rpc_errors);
                result["status"] = json!(if rpc_errors > 0 {
                    "rpc_unresolved"
                } else if rpc_mismatches > 0 || holdout_metrics.mismatches > 0 {
                    "value_mismatch"
                } else {
                    "candidate_matches_values_not_qualified"
                });
                result["exact_mapper_parity"] = json!(
                    errors.is_empty() && mapper_metrics.json()["exact_row_parity"] == true && rpc_errors == 0 && rpc_mismatches == 0 && codes[0] == codes[1]
                );
                eprintln!(
                    "Tested rank {} {}: {} shared holdout rows, {} missing, {} mapper errors",
                    token["rank"],
                    contract,
                    holdout_metrics.shared,
                    holdout_metrics.missing,
                    errors.len()
                );
                report["tokens"].as_array_mut().unwrap().push(result);
                write_report(&args.output, report)?;
            }
            ensure!(
                rpc.header(start)?["hash"] == ranking["first_hash"] && rpc.header(stop - 1)?["hash"] == ranking["last_hash"],
                "final ranking boundary changed"
            );
            report["rpc_checks_sha256"] = json!(sha256(&args.output.join("rpc-checks.jsonl"))?);
            report["observations_sha256"] = json!(sha256(&args.output.join("observations.jsonl"))?);
            let tokens = items(report, "tokens")?;
            let summary = json!({"tested_tokens":tokens.len(),"tokens_with_matching_candidate":tokens.iter().filter(|t|t["status"]=="candidate_matches_values_not_qualified").count(),
            "tokens_with_exact_mapper_parity":tokens.iter().filter(|t|t["exact_mapper_parity"]==true).count(),
            "rpc_value_checks":tokens.iter().filter_map(|t|t["rpc_value_checks"].as_u64()).sum::<u64>(),
            "rpc_mismatches":tokens.iter().filter_map(|t|t["rpc_mismatches"].as_u64()).sum::<u64>(),
            "rpc_errors":tokens.iter().filter_map(|t|t["rpc_errors"].as_u64()).sum::<u64>()});
            let status = parity_status(tokens);
            report["summary"] = summary;
            report["status"] = json!(status);
            report["investigation_complete"] = json!(true);
            Ok(())
        },
    )
}
