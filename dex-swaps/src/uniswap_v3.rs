use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::uniswap::v3 as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    let event = abi::pool::events::Swap::match_and_decode(log)?;
    let pool = pools.get(log.address.as_slice())?;
    let token0 = pool.tokens.first()?;
    let token1 = pool.tokens.get(1)?;

    let amount0 = event.amount0.to_string();
    let amount1 = event.amount1.to_string();
    let (input_token, input_amount, output_token, output_amount) = signed_swap_direction(token0, token1, &amount0, &amount1)?;

    Some(pb::Swap {
        protocol: pb::Protocol::UniswapV3 as i32,
        factory: pool.factory.clone(),
        pool: log.address.clone(),
        user: tx.from.to_vec(),
        input_token,
        input_amount,
        output_token,
        output_amount,
        log_ordinal: log.ordinal,
    })
}

fn signed_swap_direction(
    token0: &Vec<u8>,
    token1: &Vec<u8>,
    amount0: &str,
    amount1: &str,
) -> Option<(Vec<u8>, String, Vec<u8>, String)> {
    match (is_positive(amount0), is_negative(amount0), is_positive(amount1), is_negative(amount1)) {
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

fn unsigned_amount(value: &str) -> Option<&str> {
    value.strip_prefix('-').or(Some(value)).filter(|amount| !amount.is_empty())
}

fn is_positive(value: &str) -> bool {
    !value.is_empty() && !value.starts_with('-') && value.bytes().any(|byte| byte != b'0')
}

fn is_negative(value: &str) -> bool {
    value.starts_with('-') && value[1..].bytes().any(|byte| byte != b'0')
}
