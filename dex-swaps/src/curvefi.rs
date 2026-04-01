use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::curvefi as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_log(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Vec<pb::log::Log> {
    if let Some(event) = abi::pool::events::TokenExchange::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };
        let Some(input_token) = parse_curve_token(&pool.tokens, &event.sold_id.to_string()) else {
            return Vec::new();
        };
        let Some(output_token) = parse_curve_token(&pool.tokens, &event.bought_id.to_string()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::Curvefi as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: event.buyer.to_vec(),
            input_token,
            input_amount: event.tokens_sold.to_string(),
            output_token,
            output_amount: event.tokens_bought.to_string(),
        })];
    }

    if let Some(event) = abi::cryptoswap::events::TokenExchange::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };
        let Some(input_token) = parse_curve_token(&pool.tokens, &event.sold_id.to_string()) else {
            return Vec::new();
        };
        let Some(output_token) = parse_curve_token(&pool.tokens, &event.bought_id.to_string()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::Curvefi as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: event.buyer.to_vec(),
            input_token,
            input_amount: event.tokens_sold.to_string(),
            output_token,
            output_amount: event.tokens_bought.to_string(),
        })];
    }

    if let Some(event) = abi::pool::events::CommitNewFee::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::SwapFee(pb::SwapFee {
            protocol: pb::Protocol::Curvefi as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            fee: event.fee.to_u64() as u32,
        })];
    }

    if let Some(event) = abi::pool::events::NewFee::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::SwapFee(pb::SwapFee {
            protocol: pb::Protocol::Curvefi as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            fee: event.fee.to_u64() as u32,
        })];
    }

    if let Some(event) = abi::factory::events::PlainPoolDeployed::match_and_decode(log) {
        let pool = get_create_address(_tx).unwrap_or_default();
        return vec![
            pb::log::Log::Initialize(pb::Initialize {
                protocol: pb::Protocol::Curvefi as i32,
                factory: log.address.clone(),
                pool: pool.clone(),
            }),
            pb::log::Log::SwapFee(pb::SwapFee {
                protocol: pb::Protocol::Curvefi as i32,
                factory: log.address.clone(),
                pool,
                fee: event.fee.to_u64() as u32,
            }),
        ];
    }

    if let Some(event) = abi::factory::events::MetaPoolDeployed::match_and_decode(log) {
        let pool = get_create_address(_tx).unwrap_or_default();
        return vec![
            pb::log::Log::Initialize(pb::Initialize {
                protocol: pb::Protocol::Curvefi as i32,
                factory: log.address.clone(),
                pool: pool.clone(),
            }),
            pb::log::Log::SwapFee(pb::SwapFee {
                protocol: pb::Protocol::Curvefi as i32,
                factory: log.address.clone(),
                pool,
                fee: event.fee.to_u64() as u32,
            }),
        ];
    }

    Vec::new()
}

fn parse_curve_token(tokens: &[Vec<u8>], index: &str) -> Option<Vec<u8>> {
    let index = index.parse::<usize>().ok()?;
    tokens.get(index).cloned()
}

fn get_create_address(tx: &TransactionTrace) -> Option<Vec<u8>> {
    tx.calls.iter().find(|call| call.state_reverted == false && !call.address.is_empty()).map(|call| call.address.clone())
}
