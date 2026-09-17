//! Probe zero-word paths that ordinary nonzero transfer samples can miss.
use crate::{
    cli::record_run,
    data::*,
    inspect::{self, Inspect},
    rpc::{HttpRpc, Rpc},
};
use anyhow::{ensure, Result};
use clap::Args;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Args)]
pub struct InspectRanked {
    /// Completed test-ranked report; its checks must remain beside the report.
    #[arg(long)]
    pub survey: PathBuf,
    #[arg(long = "contract")]
    pub contracts: Vec<String>,
    /// Omit per-step memory/storage snapshots while retaining stack attribution.
    #[arg(long)]
    pub compact_trace: bool,
    #[arg(long)]
    pub output: PathBuf,
}

/// Pick an actual post-block holder, including for newly deployed contracts.
/// Prefer a nonzero word so the zero override exercises a different path.
pub fn observe_probe(probes: &mut BTreeMap<String, Value>, contracts: &BTreeSet<String>, row: Value) -> Result<()> {
    let contract = binary(&row["contract"], 20)?;
    if !contracts.contains(&contract) || row["boundary"] != "after" {
        return Ok(());
    }
    let address = binary(&row["address"], 20)?;
    if address == "0x0000000000000000000000000000000000000000" {
        return Ok(());
    }
    let nonzero = !uint(&row["storage"])?.is_zero();
    if probes.get(&contract).is_none_or(|previous| previous["storage"] == "0" && nonzero) {
        probes.insert(contract, row);
    }
    Ok(())
}

pub fn control_result(report: &Value) -> &'static str {
    if report["status"] != "inspected" {
        return "incomplete";
    }
    let Some(rows) = report["read_only_state_overrides"].as_array().filter(|r| r.len() == 4) else {
        return "incomplete";
    };
    let expected = [
        "0",
        "1",
        "123",
        "115792089237316195423570985008687907853269984665640564039457584007913129639935",
    ];
    if rows
        .iter()
        .zip(expected)
        .any(|(row, word)| row["overridden_mapping_word"] != word || uint(&row["balance_of"]).is_err())
    {
        return "incomplete";
    }
    if rows
        .iter()
        .filter(|r| r["overridden_mapping_word"] != "0")
        .any(|r| r["overridden_mapping_word"] != r["balance_of"])
    {
        "nonzero_word_transform"
    } else if rows.iter().any(|r| r["overridden_mapping_word"] == "0" && r["balance_of"] != "0") {
        "zero_word_fallback"
    } else {
        "direct_word_controls_match_not_qualified"
    }
}

pub fn run(args: InspectRanked) -> Result<bool> {
    record_run(
        &args.output,
        json!({"status":"incomplete","scope":"Read-only zero/nonzero controls for ranked candidates; no automatic qualification", "tokens":[]}),
        |report| {
            let survey: Value = serde_json::from_slice(&fs::read(&args.survey)?)?;
            ensure!(survey["investigation_complete"] == true, "completed survey required");
            let checks = args.survey.parent().unwrap().join("rpc-checks.jsonl");
            ensure!(sha256(&checks)? == text(&survey["rpc_checks_sha256"])?, "survey checks digest differs");
            let selected = crate::survey::select_tokens(items(&survey, "tokens")?, &args.contracts)?;
            let contracts = selected.iter().map(|t| binary(&t["contract"], 20)).collect::<Result<BTreeSet<_>>>()?;
            let mut probes = BTreeMap::new();
            for line in BufReader::new(fs::File::open(checks)?).lines() {
                observe_probe(&mut probes, &contracts, serde_json::from_str(&line?)?)?;
            }
            let rpc = HttpRpc::from_env();
            for token in selected {
                let contract = text(&token["contract"])?;
                let mut result = json!({"rank":token["rank"],"contract":contract,"symbol":token["symbol"],"status":"no_probe_evidence"});
                if let (Some(slot), Some(probe)) = (token["candidate_balance_slot"].as_str(), probes.get(contract)) {
                    let block = number(&probe["block"])?;
                    ensure!(rpc.header(block)?["hash"] == probe["hash"], "probe block is no longer canonical");
                    let output = args.output.join(&contract[2..]);
                    inspect::run(Inspect {
                        contract: contract.into(),
                        address: text(&probe["address"])?.into(),
                        balance_slot: slot.into(),
                        zero_dependency_slot: None,
                        block,
                        compact_trace: args.compact_trace,
                        source: None,
                        deployment_artifact: None,
                        artifact_url: None,
                        output: output.clone(),
                    })?;
                    let inspection: Value = serde_json::from_slice(&fs::read(output.join("report.json"))?)?;
                    result["status"] = json!(control_result(&inspection));
                    result["probe"] = probe.clone();
                    result["inspection"] = inspection;
                }
                report["tokens"].as_array_mut().unwrap().push(result);
                write_report(&args.output, report)?;
            }
            report["survey_sha256"] = json!(sha256(&args.survey)?);
            let tokens = items(report, "tokens")?;
            report["summary"] = json!({"selected":tokens.len(),
            "inspected":tokens.iter().filter(|t|t["inspection"]["status"]=="inspected").count(),
            "zero_word_fallback":tokens.iter().filter(|t|t["status"]=="zero_word_fallback").count(),
            "nonzero_word_transform":tokens.iter().filter(|t|t["status"]=="nonzero_word_transform").count(),
            "no_probe_evidence":tokens.iter().filter(|t|t["status"]=="no_probe_evidence").count()});
            report["status"] = json!(if items(report, "tokens")?.iter().any(|t| t["status"] == "incomplete") {
                "incomplete"
            } else {
                "inspected"
            });
            Ok(())
        },
    )
}
