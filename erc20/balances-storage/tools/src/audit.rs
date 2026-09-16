use crate::{data::*, rpc::*};
use anyhow::{ensure, Result};
use serde_json::{json, Value};
use std::{fs::File, io::Write, path::Path, thread};

pub fn audit_block(rpc: &dyn Rpc, block: &Value, batch_size: usize) -> Result<Vec<Value>> {
    ensure!((1..=100).contains(&batch_size), "batch size must be 1..100");
    let height = number(&block["number"])?;
    ensure!(height > 0, "positive block required");
    let hash = binary(&block["hash"], 32)?;
    ensure!(binary(&rpc.header(height)?["hash"], 32)? == hash, "RPC block identity mismatch");
    let mut pending = Vec::new();
    for ((contract, address), expected) in candidate_rows(block)? {
        pending.push((
            json!({"block":height,"hash":hash,"contract":contract,
                "address":address,"storage":expected.to_string()}),
            balance_request(&contract, &address, block_ref(&hash)),
        ));
    }
    let mut checks = Vec::new();
    for chunk in pending.chunks(batch_size) {
        let values = rpc.batch(&chunk.iter().map(|(_, call)| call.clone()).collect::<Vec<_>>())?;
        ensure!(values.len() == chunk.len(), "incomplete RPC batch");
        for ((check, _), value) in chunk.iter().zip(values) {
            let actual = balance_result(&value, check["contract"] != "")?;
            let mut check = check.clone();
            check["rpc"] = json!(actual.to_string());
            check["match"] = json!(check["rpc"] == check["storage"]);
            checks.push(check);
        }
    }
    ensure!(binary(&rpc.header(height)?["hash"], 32)? == hash, "RPC header changed during audit");
    Ok(checks)
}

pub fn audit_blocks(rpc: &dyn Rpc, blocks: &Blocks, batch_size: usize, workers: usize, output: &Path, report: &mut Value) -> Result<()> {
    ensure!((1..=4).contains(&workers), "workers must be 1..4");
    let mut raw = File::create(output.join("rpc-checks.jsonl"))?;
    // Bound the work in flight, preserving completed blocks in order even when a later block fails.
    for chunk in blocks.values().collect::<Vec<_>>().chunks(workers) {
        let results = thread::scope(|scope| {
            let handles = chunk
                .iter()
                .map(|block| scope.spawn(move || audit_block(rpc, block, batch_size)))
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|h| h.join().unwrap_or_else(|_| Err(anyhow::anyhow!("audit worker panicked"))))
                .collect::<Vec<_>>()
        });
        for result in results {
            for check in result? {
                serde_json::to_writer(&mut raw, &check)?;
                writeln!(raw)?;
                inc(report, "checks", 1);
                inc(report, "token_checks", 1);
                inc(report, "zero_checks", u64::from(check["storage"] == "0"));
                inc(report, "mismatches", u64::from(check["match"] != true));
            }
            raw.flush()?;
            inc(report, "checked_blocks", 1);
            if report["checked_blocks"].as_u64().unwrap_or(0) % 16 == 0 {
                eprintln!(
                    "Audited {} blocks: {} RPC balances, {} mismatches",
                    report["checked_blocks"], report["checks"], report["mismatches"]
                );
            }
        }
    }
    report["checks_sha256"] = json!(sha256(&output.join("rpc-checks.jsonl"))?);
    report["status"] = json!(if report["checks"] != 0 && report["mismatches"] == 0 {
        "rpc_parity"
    } else {
        "mismatch"
    });
    Ok(())
}
