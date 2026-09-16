use crate::{data::*, rpc::*};
use anyhow::{ensure, Context, Result};
use primitive_types::U256;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Write,
    path::Path,
};

#[derive(Default)]
struct Token {
    stats: Value,
    holders: BTreeSet<String>,
    validated: BTreeSet<String>,
}
#[derive(Default)]
struct Layout {
    stats: Value,
    holders: BTreeSet<String>,
    nonzero: BTreeSet<String>,
}

pub fn classify_layout(stats: &Value) -> &'static str {
    if stats["code_changed"] == true {
        "code_change_requires_review"
    } else if stats["rpc_errors"].as_u64().unwrap_or(0) > 0 {
        "rpc_unresolved"
    } else if stats["mismatches"].as_u64().unwrap_or(0) > 0 {
        "not_direct_balance_mapping"
    } else if stats["nonzero_holders"].as_u64().unwrap_or(0) < 2 || stats["changed_observations"].as_u64().unwrap_or(0) < 2 {
        "insufficient_evidence"
    } else {
        "candidate_matches_rpc_not_qualified"
    }
}

pub fn analyze(rpc: &dyn Rpc, blocks: &Blocks, output: &Path) -> Result<Value> {
    validate_blocks(blocks)?;
    let mut tokens = BTreeMap::<String, Token>::new();
    let mut layouts = BTreeMap::<Key, Layout>::new();
    let mut raw = File::create(output.join("checks.jsonl"))?;
    for (height, block) in blocks {
        let digest = binary(&block["hash"], 32)?;
        let parent = binary(&block["parentHash"], 32)?;
        ensure!(binary(&rpc.header(*height)?["hash"], 32)? == digest, "probe RPC block differs");
        for token in items(block, "tokens")? {
            let contract = binary(&token["contract"], 20)?;
            let entry = tokens.entry(contract.clone()).or_insert_with(|| Token {
                stats: json!({"contract":contract,"blocks":0,
                "storage_changes":0,"unclassified_storage_changes":0,"code_changed":false}),
                ..Default::default()
            });
            inc(&mut entry.stats, "blocks", 1);
            inc(&mut entry.stats, "storage_changes", count_field(token, "storageChanges")?);
            inc(
                &mut entry.stats,
                "unclassified_storage_changes",
                count_field(token, "unclassifiedStorageChanges")?,
            );
            entry.stats["code_changed"] = json!(entry.stats["code_changed"] == true || token["codeChanged"] == true);
            for holder in items(token, "transferHolders")? {
                entry.holders.insert(binary(holder, 20)?);
            }
        }
        let rows = items(block, "candidates")?;
        let mut requests = BTreeMap::new();
        for row in rows {
            let contract = binary(&row["contract"], 20)?;
            let address = binary(&row["address"], 20)?;
            for (boundary, hash) in [("before", &parent), ("after", &digest)] {
                requests.insert(
                    (contract.clone(), address.clone(), boundary),
                    balance_request(&contract, &address, block_ref(hash)),
                );
            }
        }
        let mut observations = BTreeMap::new();
        for chunk in requests.iter().collect::<Vec<_>>().chunks(25) {
            let calls = chunk.iter().map(|(_, call)| (*call).clone()).collect::<Vec<_>>();
            let responses = rpc.batch_rows(&calls)?;
            for ((key, _), response) in chunk.iter().zip(responses) {
                let actual = if response["error"].is_null() {
                    balance_result(&response["result"], true).ok()
                } else {
                    None
                };
                observations.insert((*key).clone(), actual);
            }
        }
        for row in rows {
            let contract = binary(&row["contract"], 20)?;
            let address = binary(&row["address"], 20)?;
            let slot = binary(&row["mappingSlot"], 32)?;
            let storage_key = binary(&row["storageKey"], 32)?;
            let before = amount_field(row, "oldAmount")?;
            let after = amount_field(row, "amount")?;
            let token = tokens.get_mut(&contract).context("candidate missing token activity")?;
            let layout = layouts.entry((contract.clone(), slot.clone())).or_insert_with(|| Layout {
                stats: json!({"contract":contract,
                "mapping_slot":slot,"observations":0,"mismatches":0,"rpc_errors":0,"changed_observations":0,"code_changed":false}),
                ..Default::default()
            });
            inc(&mut layout.stats, "observations", 1);
            inc(&mut layout.stats, "changed_observations", u64::from(before != after));
            layout.holders.insert(address.clone());
            let mut matched = true;
            for (boundary, expected, hash) in [("before", before, &parent), ("after", after, &digest)] {
                let actual: Option<U256> = observations[&(contract.clone(), address.clone(), boundary)];
                let matches = actual == Some(expected);
                inc(&mut layout.stats, "rpc_errors", u64::from(actual.is_none()));
                inc(&mut layout.stats, "mismatches", u64::from(actual.is_some() && !matches));
                matched &= matches;
                serde_json::to_writer(
                    &mut raw,
                    &json!({"block":height,"hash":hash,"boundary":boundary,"contract":contract,"address":address,
                    "mapping_slot":slot,"storage_key":storage_key,"candidate":expected.to_string(),"rpc":actual.map(|v|v.to_string()),"match":matches}),
                )?;
                writeln!(raw)?;
            }
            if matched {
                token.validated.insert(address.clone());
                if !before.is_zero() || !after.is_zero() {
                    layout.nonzero.insert(address);
                }
            }
        }
        raw.flush()?;
        ensure!(binary(&rpc.header(*height)?["hash"], 32)? == digest, "probe RPC header changed");
        eprintln!(
            "Probed block {height}: {} token-like contracts, {} mapping candidates",
            items(block, "tokens")?.len(),
            rows.len()
        );
    }
    let mut results = Vec::new();
    let mut matching = BTreeMap::<String, u64>::new();
    let mut check_count = 0;
    for ((contract, _), mut layout) in layouts {
        layout.stats["holders"] = json!(layout.holders.len());
        layout.stats["nonzero_holders"] = json!(layout.nonzero.len());
        layout.stats["code_changed"] = tokens[&contract].stats["code_changed"].clone();
        let classification = classify_layout(&layout.stats);
        if classification == "candidate_matches_rpc_not_qualified" {
            *matching.entry(contract).or_default() += 1;
        }
        layout.stats["classification"] = json!(classification);
        check_count += count_field(&layout.stats, "observations")? * 2;
        results.push(layout.stats);
    }
    let mut token_results = Vec::new();
    for (contract, mut token) in tokens {
        let count = matching.get(&contract).copied().unwrap_or(0);
        token.stats["matching_layout_count"] = json!(count);
        token.stats["ambiguous_matching_layouts"] = json!(count > 1);
        token.stats["observed_transfer_holders"] = json!(token.holders.len());
        token.stats["holders_with_some_matching_candidate"] = json!(token.validated.len());
        token_results.push(token.stats);
    }
    Ok(json!({"token_like_contracts":token_results.len(),"mapping_layout_candidates":results.len(),
        "candidate_contracts_matching_rpc":matching.len(),"ambiguous_matching_contracts":matching.values().filter(|n| **n > 1).count(),
        "candidate_value_checks":check_count,"layouts":results,"tokens":token_results,"checks_sha256":sha256(&output.join("checks.jsonl"))?}))
}
