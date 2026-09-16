use crate::{cli::Range, data::sha256};
use anyhow::{bail, ensure, Context, Result};
use serde_json::{json, Value};
use std::{
    fs::File,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

pub fn stream(args: &Range, package: &Path, module: &str, output: &Path) -> Result<Value> {
    let stop = args.stop()?;
    let mut command = Command::new("substreams");
    command.arg("run").arg(package).arg(module).args([
        "-e",
        &args.endpoint,
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
