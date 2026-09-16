//! Preserve and diagnose prior unresolved RPC checks without rewriting a run.
use crate::{cli::record_run, data::*, rpc::*};
use anyhow::{ensure, Result};
use clap::Args;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

#[derive(Args)]
pub struct Recheck {
    /// rpc-checks.jsonl from a completed survey; only unresolved rows are retried.
    #[arg(long)]
    pub checks: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
}

pub fn run(args: Recheck) -> Result<bool> {
    record_run(
        &args.output,
        json!({"status":"incomplete","checks":0,"resolved_matches":0,"value_mismatches":0,"before_deployment":0,"unresolved":0}),
        |report| {
            let input = fs::read_to_string(&args.checks)?;
            report["original_checks_sha256"] = json!(sha256(&args.checks)?);
            let rows: Vec<Value> = input.lines().map(serde_json::from_str).collect::<std::result::Result<_, _>>()?;
            let rows: Vec<_> = rows.into_iter().filter(|r| r["rpc"].is_null() && r["match"] == false).collect();
            ensure!(!rows.is_empty(), "no unresolved checks");
            let rpc = HttpRpc::from_env();
            let stop = rows
                .iter()
                .map(|r| number(&r["block"]))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .max()
                .unwrap()
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("range overflow"))?;
            ensure_finalized(&rpc, stop)?;
            let mut headers = BTreeMap::new();
            let mut tokens = BTreeMap::<String, Value>::new();
            let mut file = File::create(args.output.join("checks.jsonl"))?;
            for row in rows {
                let contract = binary(&row["contract"], 20)?;
                let address = binary(&row["address"], 20)?;
                let hash = binary(&row["hash"], 32)?;
                let current = number(&row["block"])?;
                ensure!(current > 0, "positive block required");
                let height = match text(&row["boundary"])? {
                    "before" => current - 1,
                    "after" => current,
                    _ => anyhow::bail!("invalid boundary"),
                };
                if let std::collections::btree_map::Entry::Vacant(entry) = headers.entry(height) {
                    entry.insert(rpc.header(height)?);
                }
                ensure!(headers[&height]["hash"] == hash, "historical check is no longer canonical");
                let calls = [balance_request(&contract, &address, block_ref(&hash))];
                let response = rpc.batch_rows(&calls)?.remove(0);
                let decoded = if response["error"].is_null() {
                    balance_result(&response["result"], true).ok()
                } else {
                    None
                };
                let mut result = json!({"original":row,"response":response,"decoded":decoded.map(|v|v.to_string())});
                let category = if let Some(actual) = decoded {
                    if actual == uint(&row["storage"])? {
                        "resolved_matches"
                    } else {
                        "value_mismatches"
                    }
                } else if row["boundary"] == "before" && response["error"].is_null() && response["result"] == "0x" {
                    let old_code = rpc.call("eth_getCode", json!([contract, block_ref(&hash)]))?;
                    if let std::collections::btree_map::Entry::Vacant(entry) = headers.entry(current) {
                        entry.insert(rpc.header(current)?);
                    }
                    let current_header = &headers[&current];
                    let new_code = rpc.call("eth_getCode", json!([contract, block_ref(text(&current_header["hash"])?)]))?;
                    result["current_hash"] = current_header["hash"].clone();
                    result["parent_code_empty"] = json!(old_code == "0x");
                    result["current_code_empty"] = json!(new_code == "0x");
                    if old_code == "0x" && new_code != "0x" {
                        "before_deployment"
                    } else {
                        "unresolved"
                    }
                } else {
                    "unresolved"
                };
                result["classification"] = json!(category);
                inc(report, "checks", 1);
                inc(report, category, 1);
                let stats = tokens
                    .entry(contract)
                    .or_insert_with(|| json!({"checks":0,"resolved_matches":0,"value_mismatches":0,"before_deployment":0,"unresolved":0}));
                inc(stats, "checks", 1);
                inc(stats, category, 1);
                writeln!(file, "{result}")?;
            }
            for (height, header) in headers {
                ensure!(rpc.header(height)?["hash"] == header["hash"], "canonical header changed during diagnosis");
            }
            file.flush()?;
            report["tokens"] = json!(tokens);
            report["checks_sha256"] = json!(sha256(&args.output.join("checks.jsonl"))?);
            report["status"] = json!("inspected");
            report["scope"] = json!("Diagnosis of prior unresolved calls; absent predeployment balanceOf is never converted to zero or a passing value check");
            Ok(())
        },
    )
}
