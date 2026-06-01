//! Legacy key-value store of DEX pool metadata.
//!
//! Functionally ~1:1 with `evm-dex-foundational-store` (`map_entries`), except the output
//! is a **legacy key-value store** instead of foundational `SinkEntries`:
//!   - key   = hex-encoded pool address (lowercase, no `0x`)
//!   - value = `dex.foundational_store.v1.Pool { tokens[], factory }`
//!   - policy = `set_if_not_exists` (first writer wins, mirrors `if_not_exist: true`)
//!
//! Pool data is extracted directly from a single `sf.ethereum.type.v2.Block` using the
//! `substreams-abis` event decoders — it does NOT import per-protocol spkg packages.
//!
//! Like the foundational store, the payload is intentionally restricted to shared pool
//! metadata only: `tokens[]` plus `factory` when available. Protocol-specific init fields
//! (Aerodrome `stable`, TraderJoe `bin_step`, Kyber `swap_fee_units`/`tick_distance`, …)
//! remain available on the original pool-creation events.
//!
//! Each protocol's pool-creation decoding lives in its own module. A log maps to at most one
//! pool, so each collector returns a single `Option<PoolEntry>` rather than writing directly.

mod aerodrome;
mod balancer;
mod bancor;
mod curvefi;
mod kyber;
mod sunpump;
mod traderjoe;
mod uniswap;

use proto::pb::dex::foundational_store::v1::Pool;
use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsProto};
use substreams::Hex;
use substreams_ethereum::pb::eth::v2::{Block, CallType, Log, TransactionTrace};

/// A decoded pool ready to be written to the store.
pub(crate) struct PoolEntry {
    pub address: Vec<u8>,
    pub tokens: Vec<Vec<u8>>,
    pub factory: Vec<u8>,
}

#[substreams::handlers::store]
pub fn store_pools(block: Block, store: StoreSetIfNotExistsProto<Pool>) {
    for trx in block.transactions() {
        // CurveFi direct deployments expose pool metadata via the constructor calldata,
        // not via a log — decode it once per transaction before walking the logs.
        if let Some(entry) = curvefi::collect_curvefi_init(trx) {
            set_pool(&store, entry);
        }

        // CurveFi factory events identify the pool by the freshly-CREATE'd contract address,
        // which lives in the call traces rather than the event itself.
        let create_address = get_create_address(trx);

        for log in trx_logs(trx) {
            if let Some(entry) = collect_log(log, create_address.as_deref()) {
                set_pool(&store, entry);
            }
        }
    }
}

/// Decode the (at most one) supported pool-creation event from a single log.
fn collect_log(log: &Log, create_address: Option<&[u8]>) -> Option<PoolEntry> {
    uniswap::collect_uniswap_v1(log)
        .or_else(|| uniswap::collect_uniswap_v2(log))
        .or_else(|| uniswap::collect_uniswap_v3(log))
        .or_else(|| uniswap::collect_uniswap_v4(log))
        .or_else(|| aerodrome::collect_aerodrome(log))
        .or_else(|| kyber::collect_kyber_elastic(log))
        .or_else(|| traderjoe::collect_traderjoe(log))
        .or_else(|| balancer::collect_balancer(log))
        .or_else(|| bancor::collect_bancor(log))
        .or_else(|| sunpump::collect_sunpump(log))
        .or_else(|| curvefi::collect_curvefi_factory(log, create_address))
}

/// Write a pool entry keyed by its hex-encoded address. Empty addresses are skipped.
///
/// The ordinal is always `0`: we only ever store the first value seen for a pool and never
/// overwrite it afterwards, so there is no need to track intra-block ordering.
fn set_pool(store: &StoreSetIfNotExistsProto<Pool>, entry: PoolEntry) {
    if entry.address.is_empty() {
        return;
    }
    store.set_if_not_exists(0, Hex::encode(&entry.address), &Pool { tokens: entry.tokens, factory: entry.factory });
}

// ── Block / transaction helpers ───────────────────────────────────────────────────────────

/// Iterate a transaction's logs whether or not call traces are present.
fn trx_logs(trx: &TransactionTrace) -> Vec<&Log> {
    if trx.calls.is_empty() {
        trx.receipt().logs().map(|view| view.log).collect()
    } else {
        trx.logs_with_calls().map(|(log, _)| log).collect()
    }
}

/// Address of the first contract CREATE'd in this transaction, if any.
fn get_create_address(trx: &TransactionTrace) -> Option<Vec<u8>> {
    trx.calls
        .iter()
        .find(|call| call.call_type == CallType::Create as i32)
        .map(|call| call.address.to_vec())
}
