use crate::{audit, capture, comparison, data::*, probe, rpc::*};
use anyhow::{ensure, Context, Result};
use clap::{Args, Parser, Subcommand};
use serde_json::{json, Value};
use std::{path::PathBuf, time::Instant};

#[derive(Parser)]
#[command(about = "Qualify RPC-free ERC-20 balances against erc20/balances and historical balanceOf")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand)]
pub enum Commands {
    /// Compare map_events with erc20/balances v0.3.4, retaining every coverage gap.
    Compare(Compare),
    /// Audit every emitted WBNB old/new balance using canonical block hashes.
    AuditRpc(Audit),
    /// Probe ERC-20 storage hypotheses; never promote an adapter automatically.
    ProbeErc20(Probe),
}
#[derive(Args)]
pub struct Range {
    #[arg(long)]
    pub start: u64,
    #[arg(long, default_value_t = 64)]
    pub blocks: u64,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long, default_value_os_t = default_package())]
    pub package: PathBuf,
    #[arg(long, default_value = "bsc.substreams.pinax.network:443")]
    pub endpoint: String,
    #[arg(long, default_value_t = 300)]
    pub timeout: u64,
}
impl Range {
    pub fn stop(&self) -> Result<u64> {
        self.start.checked_add(self.blocks).context("range overflow")
    }
    pub fn validate(&self, max: u64) -> Result<()> {
        ensure!(
            self.start > 0 && (1..=max).contains(&self.blocks),
            "choose a positive start and 1..{max} blocks"
        );
        ensure!(self.timeout > 0, "positive timeout required");
        ensure!(self.stop()? <= i64::MAX as u64, "range exceeds supported height");
        Ok(())
    }
}
#[derive(Args)]
pub struct Compare {
    #[command(flatten)]
    pub range: Range,
    #[arg(long, default_value_os_t = default_reference())]
    pub reference: PathBuf,
    #[arg(long, default_value_t = 20)]
    pub rpc_samples: usize,
    #[arg(long, default_value_t = 256)]
    pub audit_mismatches: usize,
}
#[derive(Args)]
pub struct Audit {
    #[command(flatten)]
    pub range: Range,
    #[arg(long, default_value_t = 1)]
    pub workers: usize,
    #[arg(long, default_value_t = 25)]
    pub batch_size: usize,
}
#[derive(Args)]
pub struct Probe {
    #[command(flatten)]
    pub range: Range,
}

// Once the directory exists, every failed run leaves a report and all partial captures.
pub fn record_run(args: &Range, mut report: Value, work: impl FnOnce(&mut Value) -> Result<()>) -> Result<bool> {
    new_output(&args.output)?;
    let started = Instant::now();
    if let Err(error) = work(&mut report) {
        report["status"] = json!("incomplete");
        report["failure"] = json!(format!("{error:#}"));
    }
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["tool_language"] = json!("Rust");
    write_report(&args.output, &report)?;
    let good = ["bounded_parity", "rpc_parity", "discovery_only"].iter().any(|s| report["status"] == *s);
    let mut summary = report;
    for key in ["layouts", "tokens", "independent_rpc_checks"] {
        summary.as_object_mut().unwrap().remove(key);
    }
    if let Some(audit) = summary.get_mut("mismatch_rpc_audit").and_then(Value::as_object_mut) {
        audit.remove("checks");
    }
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(good)
}

pub fn verify_public_output(storage: &Blocks, events: &Blocks) -> Result<()> {
    ensure!(storage.keys().eq(events.keys()), "public output block range mismatch");
    for (height, block) in storage {
        ensure!(items(block, "unresolvedWbnbSlots")?.is_empty(), "unresolved storage prevents event output");
        let expected = candidate_rows(block)?
            .into_iter()
            .map(|(key, (_, amount))| (key, amount))
            .collect::<comparison::State>();
        ensure!(
            comparison::reference_rows(&events[height])? == expected,
            "public events differ from storage projection at {height}"
        );
    }
    Ok(())
}

fn run_compare(args: Compare) -> Result<bool> {
    let r = &args.range;
    r.validate(10000)?;
    ensure!(args.rpc_samples >= 2, "at least two independent RPC samples required");
    record_run(r, json!({"status":"incomplete","start":r.start,"blocks":r.blocks}), |report| {
        let rpc = HttpRpc::from_env();
        let stop = r.stop()?;
        ensure_finalized(&rpc, stop)?;
        qualify_runtime(&rpc, r.start, stop)?;
        let first = rpc.header(r.start)?;
        let last = rpc.header(stop - 1)?;
        let storage_timing = capture::stream(r, &r.package, "map_storage_changes", &r.output.join("storage.jsonl"))?;
        let event_timing = capture::stream(r, &r.package, "map_events", &r.output.join("events.jsonl"))?;
        let reference_timing = capture::stream(r, &args.reference, "map_events", &r.output.join("reference.jsonl"))?;
        let storage = read_stream(&r.output.join("storage.jsonl"), r.start, stop, "map_storage_changes")?;
        let events = read_stream(&r.output.join("events.jsonl"), r.start, stop, "map_events")?;
        let reference = read_stream(&r.output.join("reference.jsonl"), r.start, stop, "map_events")?;
        ensure!(
            binary(&storage[&r.start]["hash"], 32)? == binary(&first["hash"], 32)?,
            "first block differs from RPC"
        );
        ensure!(
            binary(&storage[&(stop - 1)]["hash"], 32)? == binary(&last["hash"], 32)?,
            "last block differs from RPC"
        );
        verify_public_output(&storage, &events)?;
        let result = comparison::compare(&storage, &reference, r.start, stop, &r.output.join("comparison.sqlite"))?;
        *report = result.report;
        let keys = result.changed.iter().collect::<Vec<_>>();
        let count = keys.len().min(args.rpc_samples);
        let mut checks = Vec::new();
        for i in 0..count {
            let key = keys[i * keys.len() / count];
            let actual = rpc.balance(&key.0, &key.1, block_ref(text(&report["final_hash"])?))?;
            checks.push(json!({"contract":key.0,"address":key.1,"block":stop-1,"hash":report["final_hash"],
                "storage":result.state[key].to_string(),"rpc":actual.to_string(),"match":result.state[key] == actual}));
        }
        ensure!(
            rpc.header(r.start)?["hash"] == first["hash"] && rpc.header(stop - 1)?["hash"] == last["hash"],
            "RPC header changed during comparison"
        );
        let audit = comparison::audit_differences(&rpc, &r.output.join("comparison.sqlite"), &storage, args.audit_mismatches)?;
        if checks.is_empty() || checks.iter().any(|c| c["match"] != true) {
            report["status"] = json!("mismatch");
        }
        report["timing"] = json!({"storage":storage_timing,"events":event_timing,"reference":reference_timing});
        report["independent_rpc_checks"] = json!(checks);
        report["mismatch_rpc_audit"] = audit;
        report["scope"] = json!("Shared ERC-20 Events protobuf; WBNB changed holders only; all reference-only rows reported as coverage gaps");
        report["reference"] = json!("erc20/balances map_events v0.3.4");
        report["rpc_in_ingestion"] = json!(false);
        report["chain_id"] = json!(56);
        report["finality_trust"] = json!("RPC provider finalized header; stable boundary headers around captures");
        Ok(())
    })
}

fn run_audit(args: Audit) -> Result<bool> {
    let r = &args.range;
    r.validate(2048)?;
    ensure!(
        (1..=4).contains(&args.workers) && (1..=100).contains(&args.batch_size),
        "workers 1..4; batch size 1..100"
    );
    record_run(
        r,
        json!({"status":"incomplete","start":r.start,"blocks":r.blocks,"checks":0,"native_checks":0,"token_checks":0,
        "zero_checks":0,"mismatches":0,"before_checks":0,"after_checks":0,"checked_blocks":0,
        "rpc_block_binding":"EIP-1898 blockHash, requireCanonical=true","scope":"Every emitted WBNB balance before and after each block; not full holder discovery"}),
        |report| {
            let rpc = HttpRpc::from_env();
            let stop = r.stop()?;
            ensure_finalized(&rpc, stop)?;
            report["capture"] = capture::stream(r, &r.package, "map_storage_changes", &r.output.join("storage.jsonl"))?;
            report["events_capture"] = capture::stream(r, &r.package, "map_events", &r.output.join("events.jsonl"))?;
            let blocks = read_stream(&r.output.join("storage.jsonl"), r.start, stop, "map_storage_changes")?;
            let events = read_stream(&r.output.join("events.jsonl"), r.start, stop, "map_events")?;
            qualify_runtime(&rpc, r.start, stop)?;
            validate_blocks(&blocks)?;
            verify_public_output(&blocks, &events)?;
            report["first_hash"] = json!(binary(&blocks[&r.start]["hash"], 32)?);
            report["last_hash"] = json!(binary(&blocks[&(stop - 1)]["hash"], 32)?);
            audit::audit_blocks(&rpc, &blocks, args.batch_size, args.workers, &r.output, report)
        },
    )
}

fn run_probe(args: Probe) -> Result<bool> {
    let r = &args.range;
    r.validate(128)?;
    record_run(
        r,
        json!({"status":"incomplete","start":r.start,"blocks":r.blocks,"promoted_adapters":0,
        "scope":"Contracts with ERC-20-shaped Transfer logs in this window; direct mapping hypotheses only"}),
        |report| {
            let rpc = HttpRpc::from_env();
            let stop = r.stop()?;
            ensure_finalized(&rpc, stop)?;
            report["capture"] = capture::stream(r, &r.package, "map_erc20_candidates", &r.output.join("candidates.jsonl"))?;
            let blocks = read_stream(&r.output.join("candidates.jsonl"), r.start, stop, "map_erc20_candidates")?;
            let analysis = probe::analyze(&rpc, &blocks, &r.output)?;
            report.as_object_mut().unwrap().extend(analysis.as_object().unwrap().clone());
            report["status"] = json!("discovery_only");
            Ok(())
        },
    )
}
pub fn run() -> Result<bool> {
    match Cli::parse().command {
        Commands::Compare(args) => run_compare(args),
        Commands::AuditRpc(args) => run_audit(args),
        Commands::ProbeErc20(args) => run_probe(args),
    }
}
