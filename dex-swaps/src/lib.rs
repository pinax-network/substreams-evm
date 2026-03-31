mod logs;
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

use logs::PoolMetadataMap;
use proto::pb::dex::swaps::v1 as pb;
use substreams::{errors::Error, store::FoundationalStore};
use substreams_ethereum::pb::eth::v2::{Block, TransactionTrace};

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
    let mut swaps = Vec::new();

    for log_view in tx.receipt().logs() {
        if let Some(swap) = decode_log(tx, log_view.log, pools) {
            swaps.push(swap);
        }
    }

    if swaps.is_empty() {
        return None;
    }

    Some(pb::Transaction {
        hash: tx.hash.to_vec(),
        from: tx.from.to_vec(),
        to: (!tx.to.is_empty()).then(|| tx.to.to_vec()),
        nonce: tx.nonce,
        gas_price: tx.clone().gas_price.unwrap_or_default().with_decimal(0).to_string(),
        gas_limit: tx.gas_limit,
        gas_used: tx.receipt().receipt.cumulative_gas_used,
        value: tx.clone().value.unwrap_or_default().with_decimal(0).to_string(),
        swaps,
    })
}

fn decode_log(
    tx: &TransactionTrace,
    log: &substreams_ethereum::pb::eth::v2::Log,
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

    None
}
