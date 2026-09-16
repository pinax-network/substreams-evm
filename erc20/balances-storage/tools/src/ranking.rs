use crate::{
    capture::{self, Stream},
    cli::{record_run, Rank},
    data::*,
    rpc::*,
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Activity {
    rows: u64,
    blocks: u64,
    holders: BTreeSet<String>,
    heights: Vec<u64>,
}

pub fn sample_heights(report: &Value, count: usize) -> Result<Vec<u64>> {
    ensure!((2..=64).contains(&count), "choose 2..64 samples per token");
    let mut heights = BTreeSet::new();
    for token in items(report, "selected_tokens")? {
        let active = items(token, "heights")?.iter().map(number).collect::<Result<Vec<_>>>()?;
        ensure!(
            !active.is_empty() && active.windows(2).all(|w| w[0] < w[1]) && active[0] > 0,
            "invalid active heights"
        );
        let take = count.min(active.len());
        for i in 0..take {
            heights.insert(active[if take == 1 { 0 } else { i * (active.len() - 1) / (take - 1) }]);
        }
    }
    ensure!(!heights.is_empty(), "no selected token heights");
    Ok(heights.into_iter().collect())
}

/// Rank the reference's actual output, not transfers, market cap or a token list.
pub fn rank(blocks: &Blocks) -> Result<Vec<Value>> {
    let mut tokens = BTreeMap::<String, Activity>::new();
    for (height, block) in blocks {
        let mut seen = BTreeSet::new();
        for ((contract, holder), _) in candidate_rows(block)? {
            let token = tokens.entry(contract.clone()).or_default();
            token.rows += 1;
            token.holders.insert(holder);
            if seen.insert(contract) {
                token.blocks += 1;
                token.heights.push(*height);
            }
        }
    }
    let mut tokens = tokens.into_iter().collect::<Vec<_>>();
    tokens.sort_by(|a, b| b.1.rows.cmp(&a.1.rows).then(a.0.cmp(&b.0)));
    Ok(tokens
        .into_iter()
        .enumerate()
        .map(|(i, (contract, a))| {
            json!({
                "rank":i+1,"contract":contract,"reference_rows":a.rows,"active_blocks":a.blocks,
                "unique_holders":a.holders.len(),"heights":a.heights,
            })
        })
        .collect())
}

pub fn run(args: Rank) -> Result<bool> {
    ensure!(
        (1..=10000).contains(&args.blocks) && (1..=100).contains(&args.top) && args.timeout > 0,
        "invalid ranking bounds"
    );
    record_run(
        &args.output,
        json!({"status":"incomplete","chain_id":56,"ranking":"emitted RPC balance rows descending; contract ascending breaks ties"}),
        |report| {
            let rpc = HttpRpc::from_env();
            let finalized = quantity(&rpc.call("eth_getBlockByNumber", json!(["finalized", false]))?["number"])?;
            ensure!(finalized <= u64::MAX.into(), "height overflow");
            let start = args.start.unwrap_or(finalized.low_u64().checked_sub(args.blocks + 32).context("head too low")?);
            ensure!(start > 0, "positive start required");
            let stop = start.checked_add(args.blocks).context("range overflow")?;
            ensure_finalized(&rpc, stop)?;
            let first = rpc.header(start)?;
            let last = rpc.header(stop - 1)?;
            report["start"] = json!(start);
            report["stop_exclusive"] = json!(stop);
            report["blocks"] = json!(args.blocks);
            report["first_hash"] = first["hash"].clone();
            report["last_hash"] = last["hash"].clone();
            let path = args.output.join("reference.jsonl");
            report["capture"] = capture::stream_events(
                &Stream {
                    start,
                    blocks: args.blocks,
                    endpoint: &args.endpoint,
                    timeout: args.timeout,
                    package: &args.reference,
                    params: None,
                },
                &path,
            )?;
            let events = read_stream(&path, start, stop, "map_events")?;
            let tokens = rank(&events)?;
            ensure!(!tokens.is_empty(), "no RPC balance rows");
            ensure!(
                rpc.header(start)?["hash"] == first["hash"] && rpc.header(stop - 1)?["hash"] == last["hash"],
                "reference boundary changed"
            );
            report["reference_sha256"] = json!(sha256(&path)?);
            report["reference_rows"] = json!(tokens.iter().map(|t| t["reference_rows"].as_u64().unwrap()).sum::<u64>());
            report["token_count"] = json!(tokens.len());
            report["selected_tokens"] = json!(tokens.iter().take(args.top).cloned().collect::<Vec<_>>());
            report["tokens"] = json!(tokens);
            report["status"] = json!("ranked");
            Ok(())
        },
    )
}
