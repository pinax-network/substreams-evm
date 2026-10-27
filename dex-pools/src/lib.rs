//! Reusable EVM pool metadata decoding with the existing workspace ABI contract.
//!
//! [`project`] preserves the legacy pool store's transaction/log scan order and
//! extraction semantics. It returns candidates; first-write persistence belongs
//! to the caller. [`decode_log_bytes`] accepts bytes from other protobuf models
//! and copies only the supplied log into the legacy SDK's input type.
//!
//! This crate defines no Substreams handlers, stores, output protobufs or RPC
//! calls. Its ABI 1.5 / Ethereum 0.11 dependencies still include Substreams 0.7,
//! including the existing ABI error-logging behavior; it is not SDK-free.

mod aerodrome;
mod balancer;
mod bancor;
mod curvefi;
mod kyber;
mod sunpump;
mod traderjoe;
mod uniswap;

use ethereum::CallType;
pub use ethereum::{Block, Log, TransactionTrace};
pub use substreams_ethereum::pb::eth::v2 as ethereum;

/// Pool metadata observed by the existing protocol collectors.
///
/// `address` is the existing store key: usually a 20-byte address, but a 32-byte
/// pool ID for Uniswap V4. Empty token lists and absent factories are preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolEntry {
    pub address: Vec<u8>,
    pub tokens: Vec<Vec<u8>>,
    pub factory: Vec<u8>,
}

/// Decode metadata in the same order that the legacy store attempted writes.
///
/// Only successful transactions are visited. Receipt logs are used when calls
/// are absent; otherwise the SDK supplies non-reverted call logs in ordinal
/// order. Curve constructor metadata is considered before that transaction's
/// logs. Empty keys are filtered, while repeated keys remain in observation
/// order so a caller can apply the existing first-write policy.
///
/// This compatibility API retains existing Curve heuristics: direct deployment
/// constructor tails are not authenticated by bytecode, and factory logs use the
/// transaction's first CREATE address without associating it to the emitter.
/// As in the original extractor, receipt-only transactions must contain a receipt.
pub fn project(block: &Block) -> Vec<PoolEntry> {
    let mut entries = Vec::new();
    for trx in block.transactions() {
        if let Some(entry) = decode_curvefi_init(trx) {
            push_nonempty(&mut entries, entry);
        }
        let create_address = first_create_address(trx);
        for log in transaction_logs(trx) {
            if let Some(entry) = decode_log(log, create_address.as_deref()) {
                push_nonempty(&mut entries, entry);
            }
        }
    }
    entries
}

/// Decode the first matching supported pool event from one legacy-model log.
///
/// The caller selects committed logs and supplies Curve's creation-address
/// context. This function does not prove execution, authenticate factories,
/// remove duplicate keys, or apply the store's empty-key filter.
pub fn decode_log(log: &Log, create_address: Option<&[u8]>) -> Option<PoolEntry> {
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

/// Decode a log borrowed from another transport/model, without converting a block.
///
/// ABI 1.5 accepts an owned legacy `Log`, so this adapter copies the emitter,
/// topics and data once. Its unused index/ordinal fields default to zero; this
/// only affects an ABI failure's diagnostic log index. Output matches
/// [`decode_log`] for the same bytes and creation-address context.
pub fn decode_log_bytes(address: &[u8], topics: &[&[u8]], data: &[u8], create_address: Option<&[u8]>) -> Option<PoolEntry> {
    let log = Log {
        address: address.to_vec(),
        topics: topics.iter().map(|topic| topic.to_vec()).collect(),
        data: data.to_vec(),
        ..Default::default()
    };
    decode_log(&log, create_address)
}

/// Decode the existing direct Curve StableSwap constructor-tail heuristic.
///
/// This low-level helper does not check transaction success. [`project`] does.
pub fn decode_curvefi_init(transaction: &TransactionTrace) -> Option<PoolEntry> {
    curvefi::collect_curvefi_init(transaction)
}

fn push_nonempty(entries: &mut Vec<PoolEntry>, entry: PoolEntry) {
    if !entry.address.is_empty() {
        entries.push(entry);
    }
}

fn transaction_logs(transaction: &TransactionTrace) -> Vec<&Log> {
    if transaction.calls.is_empty() {
        transaction.receipt().logs().map(|view| view.log).collect()
    } else {
        transaction.logs_with_calls().map(|(log, _)| log).collect()
    }
}

fn first_create_address(transaction: &TransactionTrace) -> Option<Vec<u8>> {
    transaction
        .calls
        .iter()
        .find(|call| call.call_type == CallType::Create as i32)
        .map(|call| call.address.to_vec())
}
