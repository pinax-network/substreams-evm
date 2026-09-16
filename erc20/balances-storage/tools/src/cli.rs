use crate::{audit, capture, comparison, data::*, probe, rpc::*};
use anyhow::{ensure, Context, Result};
use clap::{Args, Parser, Subcommand};
use erc20_balances_storage::layout::{self, VerifiedLayout};
use prost::Message;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(Parser)]
#[command(about = "Qualify the single RPC-free ERC-20 map_events against historical balanceOf")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Subcommand)]
pub enum Commands {
    /// Stream the RPC reference and rank contracts by emitted balance rows.
    RankTokens(Rank),
    /// Capture canonical Extended blocks for native tests, without a map module.
    CaptureBlocks(CaptureBlocks),
    /// Test ranked tokens with native storage hypotheses; never promote layouts.
    TestRanked(crate::survey::Survey),
    /// Compare map_events with erc20/balances v0.3.4, retaining coverage gaps.
    Compare(Compare),
    /// Audit every emitted end-of-block balance using canonical block hashes.
    AuditRpc(Audit),
    /// Native discovery over captured Extended Block .pb files; no map/cache.
    ProbeErc20(Probe),
}
#[derive(Args)]
pub struct CaptureBlocks {
    #[arg(long, required_unless_present = "ranking", conflicts_with = "ranking")]
    pub start: Option<u64>,
    /// Ranking report whose selected tokens determine active block samples.
    #[arg(long)]
    pub ranking: Option<PathBuf>,
    #[arg(long, default_value_t = 8)]
    pub samples_per_token: usize,
    #[arg(long, default_value_t = 32)]
    pub blocks: u64,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long, default_value = "bsc.firehose.pinax.network:443")]
    pub endpoint: String,
    #[arg(long, default_value_t = 60)]
    pub timeout: u64,
}
#[derive(Args)]
pub struct Rank {
    /// Omit to sample immediately before the current finalized head.
    #[arg(long)]
    pub start: Option<u64>,
    #[arg(long, default_value_t = 512)]
    pub blocks: u64,
    #[arg(long, default_value_t = 10)]
    pub top: usize,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long, default_value_os_t=default_reference())]
    pub reference: PathBuf,
    #[arg(long, default_value = "bsc.substreams.pinax.network:443")]
    pub endpoint: String,
    #[arg(long, default_value_t = 600)]
    pub timeout: u64,
}
#[derive(Args)]
pub struct Range {
    #[arg(long)]
    pub start: u64,
    #[arg(long, default_value_t = 64)]
    pub blocks: u64,
    #[arg(long)]
    pub output: PathBuf,
    /// JSON array of caller-qualified token layouts; no built-in token list.
    #[arg(long)]
    pub layouts: PathBuf,
    #[arg(long,default_value_os_t=default_package())]
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
    #[arg(long,default_value_os_t=default_reference())]
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
    /// Captured sf.ethereum.type.v2.Block protobuf files, in consecutive order.
    #[arg(long = "block-file", required = true)]
    pub block_files: Vec<PathBuf>,
    #[arg(long)]
    pub output: PathBuf,
}

pub fn record_run(output: &Path, mut report: Value, work: impl FnOnce(&mut Value) -> Result<()>) -> Result<bool> {
    new_output(output)?;
    let started = Instant::now();
    if let Err(error) = work(&mut report) {
        report["status"] = json!("incomplete");
        report["failure"] = json!(format!("{error:#}"));
    }
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["tool_language"] = json!("Rust");
    write_report(output, &report)?;
    let good = ["bounded_parity", "rpc_parity", "discovery_only", "ranked", "captured"]
        .iter()
        .any(|s| report["status"] == *s);
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
fn load_layouts(r: &Range, report: &mut Value) -> Result<Vec<VerifiedLayout>> {
    let layouts = layout::parse(&fs::read_to_string(&r.layouts)?)?;
    ensure!(!layouts.is_empty(), "qualification requires at least one configured token layout");
    report["layouts_sha256"] = json!(sha256(&r.layouts)?);
    report["configured_tokens"] = json!(layouts.len());
    Ok(layouts)
}
pub fn validate_events_layouts(events: &Blocks, layouts: &[VerifiedLayout]) -> Result<()> {
    let allowed = layouts
        .iter()
        .map(|l| format!("0x{}", hex::encode(&l.contract)))
        .collect::<std::collections::BTreeSet<_>>();
    for block in events.values() {
        for (contract, _) in candidate_rows(block)?.keys() {
            ensure!(allowed.contains(contract), "map emitted an unconfigured token");
        }
    }
    Ok(())
}
fn run_compare(args: Compare) -> Result<bool> {
    let r = &args.range;
    r.validate(10000)?;
    ensure!(args.rpc_samples >= 2, "at least two independent RPC samples required");
    record_run(&r.output, json!({"status":"incomplete","start":r.start,"blocks":r.blocks}), |report| {
        let rpc = HttpRpc::from_env();
        let stop = r.stop()?;
        let layouts = load_layouts(r, report)?;
        ensure_finalized(&rpc, stop)?;
        qualify_runtime(&rpc, r.start, stop, &layouts)?;
        let first = rpc.header(r.start)?;
        let last = rpc.header(stop - 1)?;
        let candidate_timing = capture::stream(r, &r.package, "map_events", &r.output.join("events.jsonl"))?;
        let reference_timing = capture::stream(r, &args.reference, "map_events", &r.output.join("reference.jsonl"))?;
        let events = read_stream(&r.output.join("events.jsonl"), r.start, stop, "map_events")?;
        let reference = read_stream(&r.output.join("reference.jsonl"), r.start, stop, "map_events")?;
        validate_events_layouts(&events, &layouts)?;
        let blocks = bind_headers(&rpc, &events)?;
        ensure!(
            blocks[&r.start]["hash"] == first["hash"] && blocks[&(stop - 1)]["hash"] == last["hash"],
            "capture boundary header changed"
        );
        let result = comparison::compare(&blocks, &reference, r.start, stop, &r.output.join("comparison.sqlite"))?;
        report.as_object_mut().unwrap().extend(result.report.as_object().unwrap().clone());
        let keys = result.changed.iter().collect::<Vec<_>>();
        let count = keys.len().min(args.rpc_samples);
        let mut checks = Vec::new();
        for i in 0..count {
            let key = keys[i * keys.len() / count];
            let actual = rpc.balance(&key.0, &key.1, block_ref(text(&report["final_hash"])?))?;
            checks.push(json!({"contract":key.0,"address":key.1,"block":stop-1,"hash":report["final_hash"],"storage":result.state[key].to_string(),"rpc":actual.to_string(),"match":result.state[key]==actual}));
        }
        ensure!(
            rpc.header(r.start)?["hash"] == first["hash"] && rpc.header(stop - 1)?["hash"] == last["hash"],
            "RPC header changed during comparison"
        );
        let audit = comparison::audit_differences(&rpc, &r.output.join("comparison.sqlite"), &blocks, args.audit_mismatches)?;
        if checks.is_empty() || checks.iter().any(|c| c["match"] != true) {
            report["status"] = json!("mismatch");
        }
        report["timing"] = json!({"events":candidate_timing,"reference":reference_timing});
        report["independent_rpc_checks"] = json!(checks);
        report["mismatch_rpc_audit"] = audit;
        report["scope"] = json!("Configured direct-mapping ERC-20 changed holders; reference-only rows are coverage gaps, never candidate seeds");
        report["reference"] = json!("erc20/balances map_events v0.3.4");
        report["rpc_in_ingestion"] = json!(false);
        report["chain_id"] = json!(56);
        report["finality_trust"] = json!("RPC provider finalized headers; Events contains no source hash");
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
        &r.output,
        json!({"status":"incomplete","start":r.start,"blocks":r.blocks,"checks":0,"token_checks":0,
        "zero_checks":0,"mismatches":0,"checked_blocks":0,"rpc_block_binding":"EIP-1898 blockHash, requireCanonical=true",
        "scope":"Every emitted end-of-block ERC-20 balance; Events has no old value or source hash"}),
        |report| {
            let rpc = HttpRpc::from_env();
            let stop = r.stop()?;
            let layouts = load_layouts(r, report)?;
            ensure_finalized(&rpc, stop)?;
            qualify_runtime(&rpc, r.start, stop, &layouts)?;
            let first = rpc.header(r.start)?;
            let last = rpc.header(stop - 1)?;
            report["capture"] = capture::stream(r, &r.package, "map_events", &r.output.join("events.jsonl"))?;
            let events = read_stream(&r.output.join("events.jsonl"), r.start, stop, "map_events")?;
            validate_events_layouts(&events, &layouts)?;
            let blocks = bind_headers(&rpc, &events)?;
            ensure!(
                blocks[&r.start]["hash"] == first["hash"] && blocks[&(stop - 1)]["hash"] == last["hash"],
                "capture boundary header changed"
            );
            report["first_hash"] = first["hash"].clone();
            report["last_hash"] = last["hash"].clone();
            audit::audit_blocks(&rpc, &blocks, args.batch_size, args.workers, &r.output, report)?;
            ensure!(
                rpc.header(r.start)?["hash"] == first["hash"] && rpc.header(stop - 1)?["hash"] == last["hash"],
                "RPC header changed during audit"
            );
            Ok(())
        },
    )
}
fn run_probe(args: Probe) -> Result<bool> {
    ensure!((1..=128).contains(&args.block_files.len()), "choose 1..128 captured block files");
    record_run(
        &args.output,
        json!({"status":"incomplete","promoted_adapters":0,"scope":"Native discovery over captured Extended blocks; no map/cache"}),
        |report| {
            let mut blocks = Blocks::new();
            let mut files = Vec::new();
            for path in &args.block_files {
                let block = substreams_ethereum::pb::eth::v2::Block::decode(fs::read(path)?.as_slice())?;
                files.push(json!({"block":block.number,"sha256":sha256(path)?}));
                let discovery = erc20_balances_storage::discovery::project(&block)?;
                ensure!(blocks.insert(block.number, discovery.into_json()).is_none(), "duplicate captured block");
            }
            let start = *blocks.first_key_value().unwrap().0;
            let stop = blocks.last_key_value().unwrap().0.checked_add(1).context("range overflow")?;
            ensure!(
                start > 0 && blocks.keys().copied().eq(start..stop),
                "captured blocks must form a contiguous positive range"
            );
            let rpc = HttpRpc::from_env();
            ensure_finalized(&rpc, stop)?;
            report["start"] = json!(start);
            report["blocks"] = json!(blocks.len());
            report["captured_files"] = json!(files);
            let analysis = probe::analyze(&rpc, &blocks, &args.output)?;
            report.as_object_mut().unwrap().extend(analysis.as_object().unwrap().clone());
            report["status"] = json!("discovery_only");
            Ok(())
        },
    )
}
pub fn run() -> Result<bool> {
    match Cli::parse().command {
        Commands::RankTokens(args) => crate::ranking::run(args),
        Commands::CaptureBlocks(args) => capture::blocks(args),
        Commands::TestRanked(args) => crate::survey::run(args),
        Commands::Compare(args) => run_compare(args),
        Commands::AuditRpc(args) => run_audit(args),
        Commands::ProbeErc20(args) => run_probe(args),
    }
}
