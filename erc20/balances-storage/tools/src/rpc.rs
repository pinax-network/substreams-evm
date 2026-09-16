use crate::data::*;
use anyhow::{anyhow, bail, ensure, Context, Result};
use primitive_types::U256;
use serde_json::{json, Value};
use std::time::Duration;

pub type Call = (String, Value);
pub trait Rpc: Sync {
    fn request(&self, payload: Value) -> Result<Value>;
    fn call(&self, method: &str, params: Value) -> Result<Value> {
        let row = self.request(json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))?;
        ensure!(row["id"].as_u64() == Some(1), "invalid RPC response ID");
        ensure!(row["error"].is_null() && !row["result"].is_null(), "RPC {method} returned error/null");
        Ok(row["result"].clone())
    }
    fn batch(&self, calls: &[Call]) -> Result<Vec<Value>> {
        batch_results(json!(self.batch_rows(calls)?), calls.len())
    }
    fn batch_rows(&self, calls: &[Call]) -> Result<Vec<Value>> {
        ensure!(!calls.is_empty(), "empty RPC batch");
        let payload = batch_payload(calls);
        // Some gateways do not preserve the array envelope for singleton
        // batches. Send an ordinary single request instead; IDs and errors
        // still pass the same strict validation as multi-request batches.
        let response = if calls.len() == 1 {
            json!([self.request(payload[0].clone())?])
        } else {
            self.request(payload)?
        };
        batch_responses(response, calls.len())
    }
    fn header(&self, height: u64) -> Result<Value> {
        let h = self.call("eth_getBlockByNumber", json!([format!("{height:#x}"), false]))?;
        ensure!(quantity(&h["number"])? == U256::from(height), "RPC returned wrong height");
        Ok(h)
    }
    fn balance(&self, contract: &str, address: &str, reference: Value) -> Result<U256> {
        let (method, params) = balance_request(contract, address, reference);
        balance_result(&self.call(&method, params)?, !contract.is_empty())
    }
}

#[derive(Clone)]
pub struct HttpRpc {
    agent: ureq::Agent,
    url: String,
    key: Option<String>,
}
impl HttpRpc {
    pub fn from_env() -> Self {
        Self::new(
            std::env::var("RPC_URL").unwrap_or_else(|_| "https://bsc.rpc.pinax.network".into()),
            std::env::var("RPC_API_KEY").or_else(|_| std::env::var("SUBSTREAMS_API_KEY")).ok(),
        )
    }
    pub fn new(url: String, key: Option<String>) -> Self {
        Self {
            agent: ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).redirects(0).build(),
            url,
            key,
        }
    }
}
impl Rpc for HttpRpc {
    fn request(&self, payload: Value) -> Result<Value> {
        let mut request = self.agent.post(&self.url);
        if let Some(key) = &self.key {
            request = request.set("X-Api-Key", key);
        }
        let response = match request.send_json(payload) {
            Ok(response) => response,
            Err(ureq::Error::Status(code, _)) => bail!("RPC HTTP {code}"),
            Err(_) => bail!("RPC transport failed"),
        };
        response.into_json().map_err(|_| anyhow!("invalid RPC JSON response"))
    }
}
pub fn quantity(value: &Value) -> Result<U256> {
    let s = text(value)?.strip_prefix("0x").context("invalid RPC balance encoding")?;
    ensure!(
        !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid RPC uint256"
    );
    U256::from_str_radix(s, 16).map_err(|_| anyhow!("invalid RPC uint256"))
}
pub fn balance_result(value: &Value, token: bool) -> Result<U256> {
    if token {
        // Match the reference's ethabi Uint(256) decoder: one complete leading
        // word is required, but trailing return data is permitted. Venus's
        // delegator returns 96 bytes for balanceOf, with its value first.
        let encoded = text(value)?.strip_prefix("0x").context("invalid ABI hex prefix")?;
        let bytes = hex::decode(encoded).context("invalid ABI return data")?;
        ensure!(bytes.len() >= 32, "balanceOf must return a complete uint256 word");
        return Ok(U256::from_big_endian(&bytes[..32]));
    }
    quantity(value)
}
pub fn block_ref(hash: &str) -> Value {
    json!({"blockHash":hash,"requireCanonical":true})
}
pub fn balance_request(contract: &str, address: &str, reference: Value) -> Call {
    if contract.is_empty() {
        ("eth_getBalance".into(), json!([address, reference]))
    } else {
        (
            "eth_call".into(),
            json!([{"to":contract,"data":format!("0x70a08231{:0>64}", address.trim_start_matches("0x"))}, reference]),
        )
    }
}
pub fn batch_payload(calls: &[Call]) -> Value {
    json!(calls
        .iter()
        .enumerate()
        .map(|(id, (method, params))| json!({"jsonrpc":"2.0","id":id,"method":method,"params":params}))
        .collect::<Vec<_>>())
}
pub fn batch_responses(response: Value, count: usize) -> Result<Vec<Value>> {
    let rows = response.as_array().context("invalid RPC batch")?;
    ensure!(count > 0 && rows.len() == count, "incomplete RPC batch");
    let mut ordered = vec![None; count];
    for row in rows {
        let id = row["id"].as_u64().context("invalid RPC batch ID")?;
        ensure!(id < count as u64 && ordered[id as usize].is_none(), "duplicate or invalid RPC batch ID");
        ordered[id as usize] = Some(row.clone());
    }
    ordered.into_iter().map(|r| r.context("missing RPC batch ID")).collect()
}
pub fn batch_results(response: Value, count: usize) -> Result<Vec<Value>> {
    batch_responses(response, count)?
        .into_iter()
        .map(|row| {
            ensure!(row["error"].is_null() && !row["result"].is_null(), "RPC batch error/null result");
            Ok(row["result"].clone())
        })
        .collect()
}
pub fn ensure_finalized(rpc: &dyn Rpc, stop: u64) -> Result<()> {
    ensure!(quantity(&rpc.call("eth_chainId", json!([]))?)? == U256::from(56), "BSC chain ID required");
    let head = rpc.call("eth_getBlockByNumber", json!(["finalized", false]))?;
    ensure!(U256::from(stop - 1) <= quantity(&head["number"])?, "range is not finalized");
    Ok(())
}
pub fn qualify_runtime(rpc: &dyn Rpc, start: u64, stop: u64, layouts: &[erc20_balances_storage::layout::VerifiedLayout]) -> Result<()> {
    for height in [start - 1, stop - 1] {
        let h = rpc.header(height)?;
        for layout in layouts {
            let contract = format!("0x{}", hex::encode(&layout.contract));
            let code = rpc.call("eth_getCode", json!([contract, block_ref(text(&h["hash"])?)]))?;
            let bytes = hex::decode(text(&code)?.strip_prefix("0x").context("invalid runtime hex")?)?;
            ensure!(
                !bytes.is_empty() && erc20_balances_storage::hash(&bytes) == layout.code_hash,
                "unqualified runtime for configured token"
            );
            if let Some(rule) = &layout.zero_balance {
                if let Some(slot) = rule.storage_slot {
                    let actual = rpc.call(
                        "eth_getStorageAt",
                        json!([contract, format!("0x{}", hex::encode(slot)), block_ref(text(&h["hash"])?)]),
                    )?;
                    ensure!(
                        binary(&actual, 32)? == format!("0x{}", hex::encode(rule.value)),
                        "unqualified zero-balance dependency value"
                    );
                }
            }
            if let Some(proxy) = &layout.proxy {
                let slot = format!("0x{}", hex::encode(proxy.implementation_slot));
                let target = rpc.call("eth_getStorageAt", json!([contract, slot, block_ref(text(&h["hash"])?)]))?;
                let mut expected = vec![0; 12];
                expected.extend_from_slice(&proxy.implementation);
                ensure!(
                    binary(&target, 32)? == format!("0x{}", hex::encode(expected)),
                    "unqualified proxy implementation"
                );
                let implementation = format!("0x{}", hex::encode(&proxy.implementation));
                let code = rpc.call("eth_getCode", json!([implementation, block_ref(text(&h["hash"])?)]))?;
                let bytes = hex::decode(text(&code)?.strip_prefix("0x").context("invalid implementation runtime hex")?)?;
                ensure!(
                    !bytes.is_empty() && erc20_balances_storage::hash(&bytes) == proxy.code_hash,
                    "unqualified implementation runtime"
                );
            }
        }
    }
    Ok(())
}

/// Events has no block metadata. Bind captured heights to finalized RPC headers,
/// then recheck boundaries after auditing; no extra Substreams module is needed.
pub fn bind_headers(rpc: &dyn Rpc, events: &Blocks) -> Result<Blocks> {
    let mut blocks = Blocks::new();
    for (height, event) in events {
        let h = rpc.header(*height)?;
        let mut block = event.clone();
        ensure!(block.is_object(), "invalid Events object");
        block["number"] = json!(height);
        block["hash"] = json!(binary(&h["hash"], 32)?);
        block["parentHash"] = json!(binary(&h["parentHash"], 32)?);
        blocks.insert(*height, block);
    }
    validate_blocks(&blocks)?;
    Ok(blocks)
}
