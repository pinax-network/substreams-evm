use anyhow::{anyhow, ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use primitive_types::U256;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

pub type Key = (String, String);
pub type Blocks = BTreeMap<u64, Value>;
pub type Balances = BTreeMap<Key, U256>;

pub fn package_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}
pub fn default_package() -> PathBuf {
    package_dir()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("spkg/erc20-balances-storage-v0.1.0.spkg")
}
pub fn default_reference() -> PathBuf {
    package_dir().parent().unwrap().parent().unwrap().join("spkg/erc20-balances-v0.3.4.spkg")
}
pub fn text(value: &Value) -> Result<&str> {
    value.as_str().context("expected string")
}
pub fn binary(value: &Value, size: usize) -> Result<String> {
    let value = text(value)?;
    let raw = match value.strip_prefix("0x") {
        Some(hex) => hex::decode(hex)?,
        None => STANDARD.decode(value)?,
    };
    ensure!(raw.len() == size, "expected {size} bytes");
    Ok(format!("0x{}", hex::encode(raw)))
}
pub fn uint(value: &Value) -> Result<U256> {
    if let Some(n) = value.as_u64() {
        return Ok(n.into());
    }
    let s = text(value)?;
    ensure!(!s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()), "invalid unsigned integer");
    U256::from_dec_str(s).map_err(|_| anyhow!("amount outside uint256"))
}
pub fn number(value: &Value) -> Result<u64> {
    let n = uint(value)?;
    ensure!(n <= U256::from(u64::MAX), "integer exceeds u64");
    Ok(n.low_u64())
}
pub fn amount_field(value: &Value, field: &str) -> Result<U256> {
    value.get(field).map(uint).unwrap_or(Ok(U256::zero()))
}
pub fn count_field(value: &Value, field: &str) -> Result<u64> {
    value.get(field).map(number).unwrap_or(Ok(0))
}
pub fn items<'a>(value: &'a Value, field: &str) -> Result<&'a [Value]> {
    match value.get(field) {
        None => Ok(&[]),
        Some(v) => v.as_array().map(Vec::as_slice).context("expected array"),
    }
}
pub fn inc(report: &mut Value, key: &str, amount: u64) {
    report[key] = json!(report[key].as_u64().unwrap_or(0) + amount);
}
pub fn sha256(path: &Path) -> Result<String> {
    Ok(hex::encode(Sha256::digest(fs::read(path)?)))
}
pub fn write_report(output: &Path, report: &Value) -> Result<()> {
    fs::write(output.join("report.json"), format!("{}\n", serde_json::to_string_pretty(report)?))?;
    Ok(())
}
pub fn new_output(output: &Path) -> Result<()> {
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir(output).context("output directory must not already exist")
}
pub fn read_stream(path: &Path, start: u64, stop: u64, module: &str) -> Result<Blocks> {
    let mut blocks = Blocks::new();
    let expected_type = match module {
        "map_events" => "evm.balances.v1.Events",
        _ => return Err(anyhow!("unsupported capture module")),
    };
    for line in BufReader::new(fs::File::open(path)?).lines() {
        let row: Value = serde_json::from_str(&line?)?;
        ensure!(row["@module"] == module, "unexpected module output");
        ensure!(row["@type"] == expected_type, "unexpected protobuf output type");
        let height = number(&row["@block"])?;
        let data = row.get("@data").context("missing module data")?.clone();
        ensure!(blocks.insert(height, data).is_none(), "duplicate block output");
    }
    ensure!(blocks.keys().copied().eq(start..stop), "incomplete or out-of-range stream");
    Ok(blocks)
}
pub fn validate_blocks(blocks: &Blocks) -> Result<()> {
    let mut previous = None;
    for (height, block) in blocks {
        ensure!(number(&block["number"])? == *height, "candidate number mismatch");
        let parent = binary(&block["parentHash"], 32)?;
        ensure!(previous.as_ref().is_none_or(|p| p == &parent), "candidate gap or fork");
        previous = Some(binary(&block["hash"], 32)?);
    }
    Ok(())
}
pub fn candidate_rows(block: &Value) -> Result<Balances> {
    let mut rows = Balances::new();
    for row in items(block, "balances")? {
        let contract = binary(&row["contract"], 20)?;
        let key = (contract, binary(&row["address"], 20)?);
        ensure!(rows.insert(key, uint(&row["amount"])?).is_none(), "duplicate candidate balance");
    }
    Ok(rows)
}
