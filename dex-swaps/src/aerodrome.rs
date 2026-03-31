use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::aerodrome as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    let event = abi::pool::events::Swap::match_and_decode(log)?;
    let pool = pools.get(log.address.as_slice())?;
    let token0 = pool.tokens.first()?;
    let token1 = pool.tokens.get(1)?;

    let amount0_in = event.amount0_in.to_string();
    let amount0_out = event.amount0_out.to_string();
    let amount1_in = event.amount1_in.to_string();
    let amount1_out = event.amount1_out.to_string();

    let (input_token, input_amount, output_token, output_amount) = match (
        is_non_zero(&amount0_in),
        is_non_zero(&amount0_out),
        is_non_zero(&amount1_in),
        is_non_zero(&amount1_out),
    ) {
        (true, false, false, true) => (token0.clone(), amount0_in, token1.clone(), amount1_out),
        (false, true, true, false) => (token1.clone(), amount1_in, token0.clone(), amount0_out),
        _ => return None,
    };

    Some(pb::Swap {
        protocol: pb::Protocol::Aerodrome as i32,
        factory: pool.factory.clone(),
        pool: log.address.clone(),
        user: event.sender.to_vec(),
        input_token,
        input_amount,
        output_token,
        output_amount,
        log_ordinal: log.ordinal,
    })
}

fn is_non_zero(value: &str) -> bool {
    !value.is_empty() && value.bytes().any(|byte| byte != b'0')
}
