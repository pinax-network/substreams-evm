//! Read-only, canonical historical diagnosis of balanceOf vs a mapping word.
use crate::{cli::record_run, data::*, rpc::*, survey::mapping_key};
use anyhow::{ensure, Context, Result};
use clap::Args;
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

#[derive(Args)]
pub struct Inspect {
    #[arg(long)]
    pub contract: String,
    #[arg(long)]
    pub address: String,
    #[arg(long)]
    pub balance_slot: String,
    /// Optional scalar dependency to probe while holding the balance word at 0/1.
    #[arg(long)]
    pub zero_dependency_slot: Option<String>,
    #[arg(long)]
    pub block: u64,
    /// Sourcify v2 response to bind reviewed source to this historical runtime.
    #[arg(long)]
    pub source: Option<PathBuf>,
    /// Project-published deployment JSON containing deployedBytecode and metadata.
    #[arg(long, conflicts_with = "source", requires = "artifact_url")]
    pub deployment_artifact: Option<PathBuf>,
    /// Immutable source URL for the supplied deployment artifact.
    #[arg(long, requires = "deployment_artifact")]
    pub artifact_url: Option<String>,
    #[arg(long)]
    pub output: PathBuf,
}

pub fn bind_deployment(artifact: &Value, contract: &str, runtime: &[u8]) -> Result<Value> {
    ensure!(binary(&artifact["address"], 20)? == contract, "deployment artifact address differs");
    let code = text(&artifact["deployedBytecode"])?;
    ensure!(
        hex::decode(code.strip_prefix("0x").context("artifact runtime prefix")?)? == runtime,
        "deployment artifact runtime differs"
    );
    let metadata: Value = serde_json::from_str(text(&artifact["metadata"])?)?;
    let sources = metadata["sources"].as_object().context("missing artifact sources")?;
    ensure!(!sources.is_empty(), "empty artifact sources");
    let mut hashes = serde_json::Map::new();
    for (name, source) in sources {
        let content = text(&source["content"])?;
        let hash = format!("0x{}", hex::encode(erc20_balances_storage::hash(content.as_bytes())));
        ensure!(source["keccak256"] == hash, "artifact source content hash differs");
        hashes.insert(name.clone(), json!(hash));
    }
    Ok(json!({"compiler":metadata["compiler"],"settings":metadata["settings"],"source_hashes":hashes,"storage_layout":artifact["storageLayout"]}))
}

pub fn storage_reads(trace: &Value) -> Result<Vec<Value>> {
    let mut reads = Vec::new();
    for pair in items(trace, "structLogs")?.windows(2) {
        if pair[0]["op"] != "SLOAD" {
            continue;
        }
        ensure!(pair[0]["depth"] == pair[1]["depth"], "SLOAD trace depth changed");
        let key = items(&pair[0], "stack")?.last().context("missing SLOAD key")?;
        let value = items(&pair[1], "stack")?.last().context("missing SLOAD result")?;
        let word = |v: &Value| -> Result<String> {
            let s = text(v)?;
            let value = quantity(&json!(format!("0x{}", s.strip_prefix("0x").unwrap_or(s))))?;
            Ok(format!("0x{value:064x}"))
        };
        reads.push(json!({"pc":pair[0]["pc"],"depth":pair[0]["depth"],"key":word(key)?,"value":word(value)?}));
    }
    Ok(reads)
}

pub fn run(args: Inspect) -> Result<bool> {
    let contract = binary(&json!(args.contract), 20)?;
    let address = binary(&json!(args.address), 20)?;
    let slot = binary(&json!(args.balance_slot), 32)?;
    ensure!(args.block > 0, "positive block required");
    record_run(
        &args.output,
        json!({"status":"incomplete","contract":contract,"address":address,"block":args.block,"balance_slot":slot}),
        |report| {
            let rpc = HttpRpc::from_env();
            ensure_finalized(&rpc, args.block.checked_add(1).context("range overflow")?)?;
            let header = rpc.header(args.block)?;
            let hash = binary(&header["hash"], 32)?;
            let reference = block_ref(&hash);
            let key = mapping_key(&address, &slot)?;
            let balance = rpc.balance(&contract, &address, reference.clone())?;
            let storage = quantity(&rpc.call("eth_getStorageAt", json!([contract, key, reference]))?)?;
            let code = rpc.call("eth_getCode", json!([contract, reference]))?;
            let bytes = hex::decode(text(&code)?.strip_prefix("0x").context("runtime hex")?)?;
            ensure!(!bytes.is_empty(), "empty runtime");
            if let Some(path) = &args.source {
                let source: Value = serde_json::from_slice(&fs::read(path)?)?;
                ensure!(binary(&source["address"], 20)? == contract, "source contract differs");
                ensure!(source["chainId"] == "56" || source["chainId"] == 56, "source chain differs");
                ensure!(
                    source["runtimeMatch"] == "match" || source["runtimeMatch"] == "exact_match",
                    "source runtime not verified"
                );
                let source_code = text(&source["runtimeBytecode"]["onchainBytecode"])?;
                ensure!(
                    hex::decode(source_code.strip_prefix("0x").context("source runtime hex")?)? == bytes,
                    "historical runtime differs from verified source"
                );
                report["source_sha256"] = json!(sha256(path)?);
                report["source_runtime_match"] = json!(true);
                report["source_url"] = json!(format!("https://sourcify.dev/server/v2/contract/56/{contract}?fields=all"));
                report["compilation"] = source["compilation"].clone();
                report["source_storage_layout"] = source["storageLayout"].clone();
                report["source_proxy_resolution"] = source["proxyResolution"].clone();
            }
            if let Some(path) = &args.deployment_artifact {
                let artifact: Value = serde_json::from_slice(&fs::read(path)?)?;
                report["deployment_artifact"] = bind_deployment(&artifact, &contract, &bytes)?;
                report["artifact_sha256"] = json!(sha256(path)?);
                report["artifact_url"] = json!(args.artifact_url);
                report["artifact_runtime_match"] = json!(true);
            }
            fs::write(args.output.join("runtime.hex"), text(&code)?)?;
            report["hash"] = json!(hash);
            report["storage_key"] = json!(key);
            report["balance_of"] = json!(balance.to_string());
            report["storage_word"] = json!(storage.to_string());
            report["runtime_keccak256"] = json!(format!("0x{}", hex::encode(erc20_balances_storage::hash(&bytes))));
            write_report(&args.output, report)?;
            let (_, params) = balance_request(&contract, &address, reference.clone());
            let call = params[0].clone();
            let trace = rpc.call(
                "debug_traceCall",
                json!([call,reference,{"enableMemory":true,"disableStack":false,"disableStorage":false,"enableReturnData":true,"timeout":"10s"}]),
            )?;
            fs::write(args.output.join("trace.json"), serde_json::to_vec(&trace)?)?;
            ensure!(trace["failed"] == false, "balanceOf trace failed");
            let returned = text(&trace["returnValue"])?;
            ensure!(
                balance_result(&json!(format!("0x{}", returned.strip_prefix("0x").unwrap_or(returned))), true)? == balance,
                "trace disagrees with balanceOf"
            );
            report["storage_reads"] = json!(storage_reads(&trace)?);
            report["execution_contexts"] = crate::trace_context::inspect(&trace, &contract)?;
            report["execution_code"] = json!(crate::trace_context::validate_code(&rpc, &reference, &report["execution_contexts"])?);
            report["trace_sha256"] = json!(sha256(&args.output.join("trace.json"))?);
            // State overrides simulate a call only; they never send a transaction.
            let mut overrides = Vec::new();
            for amount in [primitive_types::U256::zero(), 1.into(), 123.into(), primitive_types::U256::MAX] {
                let override_state = json!({contract.clone():{"stateDiff":{key.clone():format!("0x{amount:064x}")}}});
                let actual = rpc.call("eth_call", json!([call, reference, override_state]))?;
                overrides.push(json!({"overridden_mapping_word":amount.to_string(),"balance_of":balance_result(&actual,true)?.to_string()}));
            }
            report["read_only_state_overrides"] = json!(overrides);
            if let Some(dependency) = &args.zero_dependency_slot {
                let dependency = binary(&json!(dependency), 32)?;
                ensure!(dependency != key, "dependency cannot be the tested holder key");
                let mut checks = Vec::new();
                for (word, global) in [(0_u64, 0.into()), (0, 17.into()), (1, 17.into()), (0, primitive_types::U256::MAX)] {
                    let overrides =
                        json!({contract.clone():{"stateDiff":{key.clone():format!("0x{word:064x}"),dependency.clone():format!("0x{global:064x}")}}});
                    let response = rpc.call("eth_call", json!([call, reference, overrides]))?;
                    checks.push(
                        json!({"mapping_word":word.to_string(),"dependency_word":global.to_string(),"balance_of":balance_result(&response,true)?.to_string()}),
                    );
                }
                report["zero_dependency_slot"] = json!(dependency);
                report["read_only_dependency_overrides"] = json!(checks);
            }
            ensure!(rpc.header(args.block)?["hash"] == header["hash"], "header changed during inspection");
            report["status"] = json!("inspected");
            Ok(())
        },
    )
}
