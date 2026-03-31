mod logs;
mod uniswap_v2;

use logs::PoolMetadataMap;
use proto::pb::dex::swaps::v1 as pb;
use substreams::{errors::Error, store::FoundationalStore};
use substreams_ethereum::pb::eth::v2::{Block, TransactionTrace};

#[substreams::handlers::map]
pub fn map_events(block: Block, store: FoundationalStore) -> Result<pb::Events, Error> {
    let pools = logs::get_pools_by_address(&store, &logs::collect_log_addresses(&block));

    Ok(pb::Events {
        transactions: block
            .transactions()
            .filter_map(|tx| process_transaction(tx, &pools))
            .collect(),
    })
}

fn process_transaction(tx: &TransactionTrace, pools: &PoolMetadataMap) -> Option<pb::Transaction> {
    let swaps = tx
        .receipt()
        .logs()
        .filter_map(|log_view| uniswap_v2::decode_swap(tx, log_view.log, pools))
        .collect::<Vec<_>>();

    (!swaps.is_empty()).then(|| pb::Transaction {
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
