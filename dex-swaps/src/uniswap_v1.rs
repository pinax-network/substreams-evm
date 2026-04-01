use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::uniswap::v1 as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event, NULL_ADDRESS};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    if let Some(event) = abi::exchange::events::TokenPurchase::match_and_decode(log) {
        let pool = pools.get(log.address.as_slice())?;
        let token = pool.tokens.first()?;

        return Some(pb::Swap {
            protocol: pb::Protocol::UniswapV1 as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: tx.from.to_vec(),
            input_token: NULL_ADDRESS.to_vec(),
            input_amount: event.eth_sold.to_string(),
            output_token: token.clone(),
            output_amount: event.tokens_bought.to_string(),
        });
    }

    if let Some(event) = abi::exchange::events::EthPurchase::match_and_decode(log) {
        let pool = pools.get(log.address.as_slice())?;
        let token = pool.tokens.first()?;

        return Some(pb::Swap {
            protocol: pb::Protocol::UniswapV1 as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: tx.from.to_vec(),
            input_token: token.clone(),
            input_amount: event.tokens_sold.to_string(),
            output_token: NULL_ADDRESS.to_vec(),
            output_amount: event.eth_bought.to_string(),
        });
    }

    None
}
