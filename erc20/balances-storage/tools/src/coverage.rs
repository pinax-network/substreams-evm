//! Native sink-state experiment. Bootstrap RPC is explicit and measured; the
//! production map remains stateless/RPC-free. Only the test's observed holders
//! are checkpointed, never represented as the complete global holder universe.
use crate::{cli::record_run, data::*, rpc::*, survey::mapping_key};
use anyhow::{ensure, Context, Result};
use clap::Args;
use erc20_balances_storage::layout;
use prost::Message;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use substreams_ethereum::pb::eth::v2 as eth;

#[derive(Args)]
pub struct Coverage {
    #[arg(long)]
    pub ranking: PathBuf,
    /// Complete consecutive Extended blocks, not sparse active-block samples.
    #[arg(long)]
    pub block_dir: PathBuf,
    #[arg(long)]
    pub layouts: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Default)]
pub struct HolderState {
    pub observed: Balances,
    pub seeded: Balances,
    pub tokens: BTreeMap<String, Value>,
    last: Option<(u64, String)>,
}
impl HolderState {
    pub fn new(checkpoint: Balances) -> Self {
        Self {
            seeded: checkpoint,
            ..Default::default()
        }
    }
    pub fn apply(&mut self, height: u64, hash: &str, parent: &str, emitted: &Balances, reference: &Balances) -> Result<()> {
        if let Some((previous, previous_hash)) = &self.last {
            ensure!(previous.checked_add(1) == Some(height) && previous_hash == parent, "holder replay gap/fork");
        }
        for (key, value) in emitted {
            self.observed.insert(key.clone(), *value);
            self.seeded.insert(key.clone(), *value);
        }
        for (key, value) in reference {
            let r = self.tokens.entry(key.0.clone()).or_insert_with(||json!({"contract":key.0,"reference_rows":0,"emitted_rows":0,"carry_forward_matches":0,"unseeded_unknown_rows":0,"unseeded_unknown_nonzero_rows":0,"unseeded_value_mismatches":0,"seeded_matches":0,"seeded_unknown_rows":0,"seeded_value_mismatches":0}));
            inc(r, "reference_rows", 1);
            if emitted.contains_key(key) {
                inc(r, "emitted_rows", 1);
            }
            match self.observed.get(key) {
                Some(actual) if actual != value => inc(r, "unseeded_value_mismatches", 1),
                Some(_) if !emitted.contains_key(key) => inc(r, "carry_forward_matches", 1),
                None => {
                    inc(r, "unseeded_unknown_rows", 1);
                    inc(r, "unseeded_unknown_nonzero_rows", u64::from(!value.is_zero()));
                }
                _ => {}
            }
            match self.seeded.get(key) {
                Some(actual) if actual == value => inc(r, "seeded_matches", 1),
                Some(_) => inc(r, "seeded_value_mismatches", 1),
                None => inc(r, "seeded_unknown_rows", 1),
            }
        }
        self.last = Some((height, hash.to_string()));
        Ok(())
    }
}

pub fn run(args: Coverage) -> Result<bool> {
    record_run(
        &args.output,
        json!({"status":"incomplete","scope":"Bounded off-chain holder-state replay; explicit RPC checkpoint, no production bootstrap or additional map", "checkpoint_scope":"Only configured holders observed in the reference test range", "processing_rpc_calls":0}),
        |report| {
            let layouts = layout::parse(&fs::read_to_string(&args.layouts)?)?;
            ensure!(!layouts.is_empty(), "layouts required");
            let configured = layouts
                .iter()
                .map(|l| (format!("0x{}", hex::encode(&l.contract)), l))
                .collect::<BTreeMap<_, _>>();
            let ranking: Value = serde_json::from_slice(&fs::read(&args.ranking)?)?;
            ensure!(ranking["status"] == "ranked", "completed ranking required");
            let path = args.ranking.parent().context("ranking parent")?.join("reference.jsonl");
            ensure!(sha256(&path)? == text(&ranking["reference_sha256"])?, "reference digest changed");
            let reference = read_stream(&path, number(&ranking["start"])?, number(&ranking["stop_exclusive"])?, "map_events")?;
            let mut blocks = BTreeMap::<u64, eth::Block>::new();
            for entry in fs::read_dir(&args.block_dir)? {
                let path = entry?.path();
                if path.extension().is_none_or(|e| e != "pb") {
                    continue;
                }
                let b = eth::Block::decode(fs::read(&path)?.as_slice())?;
                ensure!(blocks.insert(b.number, b).is_none(), "duplicate block");
            }
            let start = *blocks.first_key_value().context("no blocks")?.0;
            let stop = blocks.last_key_value().unwrap().0.checked_add(1).context("range overflow")?;
            ensure!(start > 0 && blocks.keys().copied().eq(start..stop), "holder coverage needs consecutive blocks");
            ensure!(
                start >= number(&ranking["start"])? && stop <= number(&ranking["stop_exclusive"])?,
                "blocks outside reference range"
            );
            let mut refs = BTreeMap::new();
            let mut holders = BTreeSet::new();
            for height in start..stop {
                let rows: Balances = candidate_rows(&reference[&height])?
                    .into_iter()
                    .filter(|(key, _)| configured.contains_key(&key.0))
                    .collect();
                holders.extend(rows.keys().cloned());
                refs.insert(height, rows);
            }
            ensure!(!holders.is_empty(), "no configured reference holders");
            let rpc = HttpRpc::from_env();
            ensure_finalized(&rpc, stop)?;
            qualify_runtime(&rpc, start, stop, &layouts)?;
            let initial = rpc.header(start - 1)?;
            let initial_hash = binary(&initial["hash"], 32)?;
            report["start"] = json!(start);
            report["stop_exclusive"] = json!(stop);
            report["checkpoint_hash"] = json!(initial_hash);
            report["checkpoint_holders"] = json!(holders.len());
            report["layouts_sha256"] = json!(sha256(&args.layouts)?);
            report["reference_sha256"] = ranking["reference_sha256"].clone();
            let mut checkpoint = Balances::new();
            let mut file = File::create(args.output.join("checkpoint.jsonl"))?;
            let mut per_token = BTreeMap::<String, Value>::new();
            for chunk in holders.iter().collect::<Vec<_>>().chunks(25) {
                let mut calls = Vec::new();
                let mut keys = Vec::new();
                for (contract, address) in chunk {
                    let key = mapping_key(address, &format!("0x{}", hex::encode(configured[contract].balance_slot)))?;
                    calls.push(("eth_getStorageAt".into(), json!([contract, key, block_ref(&initial_hash)])));
                    calls.push(balance_request(contract, address, block_ref(&initial_hash)));
                    keys.push(key);
                }
                let responses = rpc.batch(&calls)?;
                for (i, key) in chunk.iter().enumerate() {
                    let word = quantity(&responses[i * 2])?;
                    let balance = balance_result(&responses[i * 2 + 1], true)?;
                    let projected = uint(&json!(configured[&key.0].project_amount(&word.to_string())))?;
                    ensure!(projected == balance, "checkpoint projection does not equal balanceOf");
                    checkpoint.insert((*key).clone(), projected);
                    let stats = per_token
                        .entry(key.0.clone())
                        .or_insert_with(|| json!({"holders":0,"nonzero_holders":0,"zero_word_fallback_holders":0}));
                    inc(stats, "holders", 1);
                    inc(stats, "nonzero_holders", u64::from(!projected.is_zero()));
                    inc(stats, "zero_word_fallback_holders", u64::from(word.is_zero() && !projected.is_zero()));
                    writeln!(
                        file,
                        "{}",
                        json!({"contract":key.0,"address":key.1,"hash":initial_hash,"storage_key":keys[i],"storage":word.to_string(),"projected":projected.to_string(),"rpc":balance.to_string()})
                    )?;
                }
            }
            file.flush()?;
            report["checkpoint_rpc_reads"] = json!(holders.len() * 2);
            report["checkpoint_sha256"] = json!(sha256(&args.output.join("checkpoint.jsonl"))?);
            report["checkpoint_tokens"] = json!(per_token);
            write_report(&args.output, report)?;
            let mut state = HolderState::new(checkpoint);
            let mut rows_file = File::create(args.output.join("holder-checks.jsonl"))?;
            let mut captured = Vec::new();
            for (height, block) in &blocks {
                let digest = format!("0x{}", hex::encode(&block.hash));
                let parent = format!("0x{}", hex::encode(&block.header.as_ref().context("missing header")?.parent_hash));
                ensure!(*height != start || parent == initial_hash, "checkpoint is not capture parent");
                ensure!(rpc.header(*height)?["hash"] == digest, "RPC/capture fork");
                let events = erc20_balances_storage::project(block, &layouts)?;
                let emitted = candidate_rows(
                    &json!({"balances":events.balances.into_iter().map(|b|json!({"contract":b.contract.map(|c|format!("0x{}",hex::encode(c))),"address":format!("0x{}",hex::encode(b.address)),"amount":b.amount})).collect::<Vec<_>>()}),
                )?;
                state.apply(*height, &digest, &parent, &emitted, &refs[height])?;
                for (key, expected) in &refs[height] {
                    writeln!(
                        rows_file,
                        "{}",
                        json!({"block":height,"hash":digest,"contract":key.0,"address":key.1,"rpc_reference":expected.to_string(),"unseeded":state.observed.get(key).map(|v|v.to_string()),"seeded":state.seeded.get(key).map(|v|v.to_string()),"emitted_this_block":emitted.contains_key(key)})
                    )?;
                }
                captured.push(json!({"block":height,"hash":digest}));
            }
            rows_file.flush()?;
            ensure!(
                rpc.header(start - 1)?["hash"] == initial["hash"] && rpc.header(stop - 1)?["hash"] == format!("0x{}", hex::encode(&blocks[&(stop - 1)].hash)),
                "coverage boundaries changed"
            );
            report["captured"] = json!(captured);
            report["tokens"] = json!(state.tokens.values().collect::<Vec<_>>());
            let seeded_ok = state.tokens.len() == layouts.len()
                && state
                    .tokens
                    .values()
                    .all(|v| v["seeded_value_mismatches"] == 0 && v["seeded_unknown_rows"] == 0 && v["reference_rows"].as_u64().unwrap_or(0) > 0);
            report["status"] = json!(if seeded_ok { "bounded_parity" } else { "mismatch" });
            report["holder_checks_sha256"] = json!(sha256(&args.output.join("holder-checks.jsonl"))?);
            report["event_row_parity_claimed"] = json!(false);
            Ok(())
        },
    )
}
