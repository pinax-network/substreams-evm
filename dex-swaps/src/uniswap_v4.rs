use common::bigint_to_u64;
use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::uniswap::v4 as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_log(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Vec<pb::log::Log> {
    if let Some(event) = abi::poolmanager::events::Swap::match_and_decode(log) {
        let Some(pool) = pools.get(event.id.as_slice()) else {
            return Vec::new();
        };
        let Some(token0) = pool.tokens.first() else {
            return Vec::new();
        };
        let Some(token1) = pool.tokens.get(1) else {
            return Vec::new();
        };

        let amount0 = event.amount0.to_string();
        let amount1 = event.amount1.to_string();
        let Some((input_token, input_amount, output_token, output_amount)) =
            signed_swap_direction(token0, token1, &amount0, &amount1) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::UniswapV4 as i32,
            factory: pool.factory.clone(),
            pool: event.id.to_vec(),
            user: tx.from.to_vec(),
            input_token,
            input_amount,
            output_token,
            output_amount,
        })];
    }

    if let Some(event) = abi::poolmanager::events::Initialize::match_and_decode(log) {
        let factory = pools.get(event.id.as_slice()).map(|pool| pool.factory.clone()).unwrap_or_default();
        let fee = bigint_to_u64(&event.fee).unwrap_or_default() as u32;
        return vec![
            pb::log::Log::Initialize(pb::Initialize {
                protocol: pb::Protocol::UniswapV4 as i32,
                factory: factory.clone(),
                pool: event.id.to_vec(),
            }),
            pb::log::Log::SwapFee(pb::SwapFee {
                protocol: pb::Protocol::UniswapV4 as i32,
                factory,
                pool: event.id.to_vec(),
                fee,
            }),
        ];
    }

    Vec::new()
}

fn unsigned_amount(value: &str) -> Option<&str> {
    value.strip_prefix('-').or(Some(value)).filter(|amount| !amount.is_empty())
}

fn signed_swap_direction(
    token0: &Vec<u8>,
    token1: &Vec<u8>,
    amount0: &str,
    amount1: &str,
) -> Option<(Vec<u8>, String, Vec<u8>, String)> {
    match (
        is_positive(amount0),
        is_negative(amount0),
        is_positive(amount1),
        is_negative(amount1),
    ) {
        (true, false, false, true) => Some((
            token0.clone(),
            amount0.to_string(),
            token1.clone(),
            unsigned_amount(amount1)?.to_string(),
        )),
        (false, true, true, false) => Some((
            token1.clone(),
            amount1.to_string(),
            token0.clone(),
            unsigned_amount(amount0)?.to_string(),
        )),
        _ => None,
    }
}

fn is_positive(value: &str) -> bool {
    !value.is_empty() && !value.starts_with('-') && value.bytes().any(|byte| byte != b'0')
}

fn is_negative(value: &str) -> bool {
    value.starts_with('-') && value[1..].bytes().any(|byte| byte != b'0')
}
