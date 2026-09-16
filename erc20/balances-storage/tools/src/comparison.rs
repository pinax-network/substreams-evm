use crate::{
    data::*,
    rpc::{block_ref, Rpc},
};
use anyhow::{ensure, Result};
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::{collections::BTreeSet, path::Path};
pub type State = Balances;
pub struct Comparison {
    pub report: Value,
    pub state: State,
    pub changed: BTreeSet<Key>,
}
pub fn reference_rows(block: &Value) -> Result<State> {
    candidate_rows(block)
}

pub fn compare(candidate: &Blocks, reference: &Blocks, start: u64, stop: u64, database: &Path) -> Result<Comparison> {
    ensure!(
        candidate.keys().copied().eq(start..stop) && reference.keys().copied().eq(start..stop),
        "different block ranges"
    );
    validate_blocks(candidate)?;
    let mut db = Connection::open(database)?;
    let tx = db.transaction()?;
    tx.execute_batch("CREATE TABLE observations(side TEXT,block_num INTEGER,block_hash TEXT,contract TEXT,address TEXT,balance TEXT,PRIMARY KEY(side,block_num,contract,address));
        CREATE TABLE differences(block_num INTEGER,contract TEXT,address TEXT,candidate TEXT,reference TEXT,kind TEXT);")?;
    let (mut ours, mut theirs, mut changed) = (State::new(), State::new(), BTreeSet::new());
    let (mut emitted_contracts, mut reference_contracts) = (BTreeSet::new(), BTreeSet::new());
    let mut report = json!({"start":start,"stop_exclusive":stop,"blocks":stop-start,"storage_updates":0,"reference_updates":0,
        "snapshot_comparisons":0,"differences":0,"candidate_only_keys":0,"reference_only_updates":0,"candidate_only_updates":0,"shared_updates":0});
    for (height, block) in candidate {
        let hash = binary(&block["hash"], 32)?;
        let new = candidate_rows(block)?;
        let old = reference_rows(&reference[height])?;
        inc(&mut report, "storage_updates", new.len() as u64);
        inc(&mut report, "reference_updates", old.len() as u64);
        for key in old.keys() {
            reference_contracts.insert(key.0.clone());
            inc(&mut report, if new.contains_key(key) { "shared_updates" } else { "reference_only_updates" }, 1);
        }
        inc(
            &mut report,
            "candidate_only_updates",
            new.keys().filter(|k| !old.contains_key(*k)).count() as u64,
        );
        for (key, value) in &new {
            ours.insert(key.clone(), *value);
            changed.insert(key.clone());
            emitted_contracts.insert(key.0.clone());
        }
        for (key, value) in &old {
            theirs.insert(key.clone(), *value);
        }
        // Only observed values are carried forward; reference-only holders never seed candidate state.
        for (side, values) in [("storage", &new), ("rpc-package", &old)] {
            for ((contract, address), value) in values {
                tx.execute(
                    "INSERT INTO observations VALUES(?,?,?,?,?,?)",
                    params![side, height, hash, contract, address, value.to_string()],
                )?;
            }
        }
        for (key, value) in &ours {
            if let Some(reference_value) = theirs.get(key) {
                inc(&mut report, "snapshot_comparisons", 1);
                if value != reference_value {
                    inc(&mut report, "differences", 1);
                    tx.execute(
                        "INSERT INTO differences VALUES(?,?,?,?,?,?)",
                        params![height, key.0, key.1, value.to_string(), reference_value.to_string(), "value"],
                    )?;
                }
            }
        }
        report["final_hash"] = json!(hash);
    }
    for (key, value) in &ours {
        if !theirs.contains_key(key) {
            inc(&mut report, "candidate_only_keys", 1);
            tx.execute(
                "INSERT INTO differences VALUES(?,?,?,?,?,?)",
                params![stop - 1, key.0, key.1, value.to_string(), Option::<String>::None, "reference_unobserved"],
            )?;
        }
    }
    report["distinct_storage_keys"] = json!(changed.len());
    report["reference_contracts_without_candidate_updates"] = json!(reference_contracts.difference(&emitted_contracts).count());
    report["status"] = json!(if report["differences"] != 0 {
        "mismatch"
    } else if report["candidate_only_keys"] != 0 || report["reference_only_updates"] != 0 || report["candidate_only_updates"] != 0 {
        "coverage_gap"
    } else {
        "bounded_parity"
    });
    tx.commit()?;
    Ok(Comparison { report, state: ours, changed })
}
pub fn audit_differences(rpc: &dyn Rpc, database: &Path, candidate: &Blocks, limit: usize) -> Result<Value> {
    let db = Connection::open(database)?;
    let mut statement =
        db.prepare("SELECT block_num,contract,address,candidate,reference FROM differences WHERE kind='value' ORDER BY block_num,contract,address")?;
    let rows = statement.query_map([], |r| {
        Ok((
            r.get::<_, u64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    let mut checks = Vec::new();
    let mut report = json!({"unchecked":0,"agrees_with_storage":0,"agrees_with_reference":0,"agrees_with_neither":0});
    for (index, row) in rows.enumerate() {
        let (height, contract, address, ours, theirs) = row?;
        if index >= limit {
            inc(&mut report, "unchecked", 1);
            continue;
        }
        let hash = binary(&candidate[&height]["hash"], 32)?;
        ensure!(binary(&rpc.header(height)?["hash"], 32)? == hash, "mismatch audit block differs from RPC");
        let actual = rpc.balance(&contract, &address, block_ref(&hash))?;
        ensure!(binary(&rpc.header(height)?["hash"], 32)? == hash, "mismatch audit RPC header changed");
        let agrees = if actual == uint(&json!(ours))? {
            "storage"
        } else if actual == uint(&json!(theirs))? {
            "reference"
        } else {
            "neither"
        };
        inc(&mut report, &format!("agrees_with_{agrees}"), 1);
        checks.push(json!({"block":height,"hash":hash,"contract":contract,"address":address,"storage":ours,"reference":theirs,"rpc":actual.to_string(),"agrees_with":agrees}));
    }
    report["checks"] = json!(checks);
    Ok(report)
}
