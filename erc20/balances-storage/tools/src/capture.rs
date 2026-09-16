use crate::{cli::Range, data::sha256};
use anyhow::{bail, ensure, Context, Result};
use prost::Message;
use serde_json::{json, Value};
use std::{
    fs::File,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

pub fn blocks(args: crate::cli::CaptureBlocks) -> Result<bool> {
    use crate::{data::*, rpc::*};
    ensure!((1..=512).contains(&args.blocks) && args.timeout > 0, "invalid capture bounds");
    let heights = if let Some(path) = &args.ranking {
        crate::ranking::sample_heights(&serde_json::from_slice(&std::fs::read(path)?)?, args.samples_per_token)?
    } else {
        let start = args.start.context("start or ranking required")?;
        ensure!(start > 0, "positive start required");
        (start..start.checked_add(args.blocks).context("range overflow")?).collect::<Vec<_>>()
    };
    let stop = heights.last().context("empty capture")?.checked_add(1).context("range overflow")?;
    crate::cli::record_run(
        &args.output,
        json!({"status":"incomplete","requested_heights":heights,"captured":[]}),
        |report| {
            let rpc = HttpRpc::from_env();
            ensure_finalized(&rpc, stop)?;
            let mut previous = None;
            for height in heights {
                let header = rpc.header(height)?;
                let digest = binary(&header["hash"], 32)?;
                let raw_path = args.output.join(format!("{height}.hex"));
                let mut command = Command::new("firecore");
                command.args([
                    "tools",
                    "firehose-single-block-client",
                    &args.endpoint,
                    &format!("{height}:{digest}"),
                    "--compression",
                    "gzip",
                    "--api-key-env-var",
                    "SUBSTREAMS_API_KEY",
                    "--output",
                    "bytes",
                    "--bytes-encoding",
                    "hex",
                ]);
                if args.endpoint.ends_with(":80") || args.endpoint.starts_with("http://") {
                    command.arg("--plaintext");
                }
                command
                    .stdout(File::create(&raw_path)?)
                    .stderr(File::create(args.output.join(format!("{height}.log")))?);
                run_command(command, args.timeout)?;
                let bytes = hex::decode(std::fs::read_to_string(&raw_path)?.trim())?;
                let block = substreams_ethereum::pb::eth::v2::Block::decode(bytes.as_slice())?;
                ensure!(
                    block.number == height && format!("0x{}", hex::encode(&block.hash)) == digest,
                    "Firehose block differs from RPC"
                );
                let parent = &block.header.as_ref().context("missing header")?.parent_hash;
                if let Some((prior_height, prior_hash)) = &previous {
                    if *prior_height + 1 == height {
                        ensure!(prior_hash == parent, "Firehose fork");
                    }
                }
                ensure!(
                    format!("0x{}", hex::encode(parent)) == binary(&header["parentHash"], 32)?,
                    "parent differs from RPC"
                );
                ensure!(
                    block.detail_level == substreams_ethereum::pb::eth::v2::block::DetailLevel::DetaillevelExtended as i32,
                    "Extended block required"
                );
                ensure!(rpc.header(height)?["hash"] == header["hash"], "RPC header changed");
                let path = args.output.join(format!("{height}.pb"));
                std::fs::write(&path, bytes)?;
                std::fs::remove_file(raw_path)?;
                report["captured"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"block":height,"hash":digest,"sha256":sha256(&path)?}));
                previous = Some((height, block.hash));
                write_report(&args.output, report)?;
                eprintln!("Captured Extended block {height}");
            }
            report["status"] = json!("captured");
            Ok(())
        },
    )
}

fn run_command(mut command: Command, timeout: u64) -> Result<()> {
    let began = Instant::now();
    let mut child = command.spawn().context("could not start capture CLI")?;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                ensure!(status.success(), "capture CLI failed; see log");
                return Ok(());
            }
            Ok(None) if began.elapsed() < Duration::from_secs(timeout) => thread::sleep(Duration::from_millis(100)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                result.context("could not wait for capture CLI")?;
                bail!("capture timed out; see log");
            }
        }
    }
}

pub fn stream(args: &Range, package: &Path, module: &str, output: &Path) -> Result<Value> {
    ensure!(module == "map_events", "only map_events is supported");
    let params = if package == args.package {
        let layouts = std::fs::read_to_string(&args.layouts)?;
        erc20_balances_storage::layout::parse(&layouts)?;
        Some(layouts)
    } else {
        None
    };
    stream_events(
        &Stream {
            start: args.start,
            blocks: args.blocks,
            endpoint: &args.endpoint,
            timeout: args.timeout,
            package,
            params: params.as_deref(),
        },
        output,
    )
}

pub struct Stream<'a> {
    pub start: u64,
    pub blocks: u64,
    pub endpoint: &'a str,
    pub timeout: u64,
    pub package: &'a Path,
    pub params: Option<&'a str>,
}

pub fn stream_events(args: &Stream<'_>, output: &Path) -> Result<Value> {
    let stop = args.start.checked_add(args.blocks).context("range overflow")?;
    let module = "map_events";
    let package = args.package;
    let mut command = Command::new("substreams");
    command.arg("run").arg(package).arg(module).args([
        "-e",
        args.endpoint,
        "-s",
        &args.start.to_string(),
        "-t",
        &stop.to_string(),
        "--final-blocks-only",
        "--max-retries",
        "0",
        "-o",
        "jsonl",
    ]);
    if args.endpoint.ends_with(":80") || args.endpoint.starts_with("http://") {
        command.arg("--plaintext");
    }
    if let Some(params) = args.params {
        command.arg("-p").arg(format!("map_events={params}"));
    }
    command.stdout(File::create(output)?).stderr(File::create(output.with_extension("log"))?);
    let began = Instant::now();
    let mut child = command.spawn().context("could not start substreams")?;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if began.elapsed() < Duration::from_secs(args.timeout) => thread::sleep(Duration::from_millis(100)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                result.context("could not wait for substreams")?;
                bail!("{module} capture timed out; see capture log");
            }
        }
    };
    ensure!(status.success(), "{module} failed; see capture log");
    let elapsed = began.elapsed().as_secs_f64();
    Ok(
        json!({"seconds_including_startup":elapsed,"blocks_per_second_including_startup":args.blocks as f64/elapsed,
        "package_sha256":sha256(package)?}),
    )
}
