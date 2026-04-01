use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::uniswap::v1 as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event, NULL_ADDRESS};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_log(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Vec<pb::log::Log> {
    if let Some(event) = abi::exchange::events::TokenPurchase::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };
        let Some(token) = pool.tokens.first() else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::UniswapV1 as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: tx.from.to_vec(),
            input_token: NULL_ADDRESS.to_vec(),
            input_amount: event.eth_sold.to_string(),
            output_token: token.clone(),
            output_amount: event.tokens_bought.to_string(),
        })];
    }

    if let Some(event) = abi::exchange::events::EthPurchase::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };
        let Some(token) = pool.tokens.first() else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::UniswapV1 as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: tx.from.to_vec(),
            input_token: token.clone(),
            input_amount: event.tokens_sold.to_string(),
            output_token: NULL_ADDRESS.to_vec(),
            output_amount: event.eth_bought.to_string(),
        })];
    }

    if let Some(event) = abi::factory::events::NewExchange::match_and_decode(log) {
        return vec![
            pb::log::Log::Initialize(pb::Initialize {
                protocol: pb::Protocol::UniswapV1 as i32,
                factory: log.address.clone(),
                pool: event.exchange.to_vec(),
            }),
            pb::log::Log::SwapFee(pb::SwapFee {
                protocol: pb::Protocol::UniswapV1 as i32,
                factory: log.address.clone(),
                pool: event.exchange.to_vec(),
                fee: 3000,
            }),
        ];
    }

    Vec::new()
}
