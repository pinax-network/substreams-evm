use crate::{cli::Range, data::*, rpc::*};
use anyhow::{bail, ensure, Context, Result};
use prost::Message;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
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
    let package_hash = sha256(args.package)?;
    let began = Instant::now();
    stream_format(args, output, "jsonl")?;
    let events = read_sparse_stream(output, args.start, stop, "map_events")?;
    let empty_blocks = args.blocks - events.len() as u64;
    let mut delivery = Value::Null;
    if empty_blocks > 0 {
        // JSONL omits nil/empty module output. The same finalized execution in
        // clock mode emits BlockScopedData identities even for empty output.
        // Retain both captures; never infer emptiness from a missing row alone.
        ensure!(
            received_blocks(&output.with_extension("log"))? == args.blocks,
            "JSONL capture did not receive every requested block"
        );
        let clocks_path = output.with_extension("clocks.txt");
        stream_format(args, &clocks_path, "clock")?;
        delivery = confirm_empty_outputs(&HttpRpc::from_env(), output, &clocks_path, args.start, stop, args.blocks)?;
    }
    ensure!(sha256(args.package)? == package_hash, "package changed during capture");
    let elapsed = began.elapsed().as_secs_f64();
    Ok(
        json!({"seconds_including_startup":elapsed,"blocks_per_second_including_startup":args.blocks as f64/elapsed,"package_sha256":package_hash,"empty_output_delivery":delivery}),
    )
}

pub fn confirm_empty_outputs(rpc: &dyn Rpc, output: &Path, clocks_path: &Path, start: u64, stop: u64, delivered: u64) -> Result<Value> {
    let events = read_sparse_stream(output, start, stop, "map_events")?;
    ensure!(delivered == stop - start, "JSONL capture did not receive every requested block");
    let clocks = read_clocks(clocks_path, start, stop)?;
    verify_clocks(rpc, &clocks)?;
    let raw_path = output.with_extension("sparse.jsonl");
    ensure!(!raw_path.exists(), "original sparse capture already exists");
    fs::rename(output, &raw_path)?;
    let mut complete = File::create(output)?;
    for height in start..stop {
        let empty = !events.contains_key(&height);
        let data = events.get(&height).cloned().unwrap_or_else(|| json!({"balances":[]}));
        serde_json::to_writer(
            &mut complete,
            &json!({"@module":"map_events","@block":height,"@type":"evm.balances.v1.Events","@data":data,"@empty_confirmed_by_clock":empty}),
        )?;
        writeln!(complete)?;
    }
    complete.flush()?;
    Ok(
        json!({"method":"Complete finalized BlockScopedData clock capture, all IDs matched to consecutive canonical RPC headers", "empty_blocks":stop-start-events.len() as u64,"clock_capture_sha256":sha256(clocks_path)?,"sparse_events_sha256":sha256(&raw_path)?,"normalized_events_sha256":sha256(output)?}),
    )
}

pub fn received_blocks(log: &Path) -> Result<u64> {
    let content = fs::read_to_string(log)?;
    let counts = content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("• Received Blocks: "))
        .collect::<Vec<_>>();
    ensure!(counts.len() == 1, "missing or ambiguous capture delivery count");
    let count = counts[0].strip_suffix(" blocks").context("invalid delivery count")?;
    ensure!(
        !count.is_empty() && count.bytes().all(|b| b.is_ascii_digit() || b == b','),
        "invalid delivery count"
    );
    Ok(count.replace(',', "").parse()?)
}

pub fn read_clocks(path: &Path, start: u64, stop: u64) -> Result<BTreeMap<u64, String>> {
    ensure!(start > 0 && stop > start, "invalid clock bounds");
    let mut clocks = BTreeMap::new();
    let mut expected = start;
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        let fields = line
            .strip_prefix("----------- BLOCK #")
            .context("unexpected clock output, undo or partial block")?;
        let (height, tail) = fields.split_once(" (").context("invalid clock line")?;
        ensure!(
            !height.is_empty() && height.bytes().all(|b| b.is_ascii_digit() || b == b','),
            "invalid clock height"
        );
        let height: u64 = height.replace(',', "").parse()?;
        let (hash, suffix) = tail.split_once(") age=").context("invalid clock identity")?;
        ensure!(suffix.ends_with(" ---------------"), "incomplete clock line");
        let hash = hash.strip_prefix("0x").unwrap_or(hash);
        ensure!(hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()), "invalid clock hash");
        ensure!(height == expected && height < stop, "clock capture gap, duplicate or out-of-range block");
        clocks.insert(height, format!("0x{}", hash.to_ascii_lowercase()));
        expected += 1;
    }
    ensure!(expected == stop, "incomplete clock capture");
    Ok(clocks)
}

pub fn verify_clocks(rpc: &dyn Rpc, clocks: &BTreeMap<u64, String>) -> Result<()> {
    ensure!(!clocks.is_empty(), "no delivered clocks");
    let mut previous = None;
    for (height, hash) in clocks {
        let header = rpc.header(*height)?;
        ensure!(&binary(&header["hash"], 32)? == hash, "delivered clock differs from canonical RPC");
        if let Some((prior_height, parent)) = previous {
            ensure!(*height == prior_height + 1, "clock capture gap");
            ensure!(binary(&header["parentHash"], 32)? == parent, "clock capture fork");
        }
        previous = Some((*height, hash.clone()));
    }
    Ok(())
}

fn stream_format(args: &Stream<'_>, output: &Path, format: &str) -> Result<()> {
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
        format,
    ]);
    if args.endpoint.ends_with(":80") || args.endpoint.starts_with("http://") {
        command.arg("--plaintext");
    }
    if let Some(params) = args.params {
        command.arg("-p").arg(format!("map_events={params}"));
    }
    let log = if format == "clock" {
        output.with_extension("clock-log")
    } else {
        output.with_extension("log")
    };
    command.stdout(File::create(output)?).stderr(File::create(log)?);
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
    Ok(())
}
