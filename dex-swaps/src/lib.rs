mod logs;
mod sunpump;
mod utils;
mod aerodrome;
mod balancer;
mod bancor;
mod cow;
mod curvefi;
mod dodo;
mod kyber_elastic;
mod traderjoe;
mod uniswap_v1;
mod uniswap_v2;
mod uniswap_v3;
mod uniswap_v4;
mod woofi;

use common::create::{CreateLog, CreateTransaction};
use logs::PoolMetadataMap;
use proto::pb::dex::swaps::v1 as pb;
use substreams::{errors::Error, store::FoundationalStore};
use substreams_ethereum::pb::eth::v2::{Block, Call, Log, TransactionTrace};

#[substreams::handlers::map]
pub fn map_events(block: Block, store: FoundationalStore) -> Result<pb::Events, Error> {
    let pools = logs::get_pools_by_address(&store, &logs::collect_log_addresses(&block));
    let transactions: Vec<pb::Transaction> = block.transactions().filter_map(|tx| process_transaction(tx, &pools)).collect();

    // No transactions, so we can skip returning an empty list of transactions and just return the default Events message.
    if transactions.is_empty() {
        return Ok(pb::Events::default());
    }

    Ok(pb::Events { transactions })
}

fn process_transaction(tx: &TransactionTrace, pools: &PoolMetadataMap) -> Option<pb::Transaction> {
    let mut transaction = pb::Transaction::create_transaction(tx);
    let logs_with_calls: Vec<(&Log, Option<&Call>)> = if tx.calls.is_empty() {
        tx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
    } else {
        tx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
    };

    for (log, call) in logs_with_calls {
        if let Some(swap) = decode_log(tx, log, pools) {
            transaction.logs.push(pb::Log::create_log_with_call(log, pb::log::Log::Swap(swap), call));
        }
    }

    if transaction.logs.is_empty() {
        return None;
    }

    Some(transaction)
}

fn decode_log(
    tx: &TransactionTrace,
    log: &Log,
    pools: &PoolMetadataMap,
) -> Option<pb::Swap> {
    if let Some(swap) = uniswap_v1::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = uniswap_v2::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = uniswap_v3::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = uniswap_v4::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = curvefi::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = balancer::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = bancor::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = cow::decode_swap(tx, log) {
        return Some(swap);
    }

    if let Some(swap) = aerodrome::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = dodo::decode_swap(tx, log) {
        return Some(swap);
    }

    if let Some(swap) = woofi::decode_swap(tx, log) {
        return Some(swap);
    }

    if let Some(swap) = traderjoe::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = kyber_elastic::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    if let Some(swap) = sunpump::decode_swap(tx, log, pools) {
        return Some(swap);
    }

    None
}
