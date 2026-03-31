use proto::pb::dex::swaps::v1 as pb;
use substreams::scalar::BigInt;
use substreams_abis::dex::traderjoe as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    let event = abi::lbpair::events::Swap::match_and_decode(log)?;
    let pool = pools.get(log.address.as_slice())?;
    let token0 = pool.tokens.first()?;
    let token1 = pool.tokens.get(1)?;
    let (amount_in_x, amount_in_y) = decode_packed_uint128(&event.amounts_in);
    let (amount_out_x, amount_out_y) = decode_packed_uint128(&event.amounts_out);

    let (input_token, input_amount, output_token, output_amount) = match (is_non_zero(&amount_in_x), is_non_zero(&amount_in_y)) {
        (true, false) => (token0.clone(), amount_in_x, token1.clone(), amount_out_y),
        (false, true) => (token1.clone(), amount_in_y, token0.clone(), amount_out_x),
        _ => return None,
    };

    Some(pb::Swap {
        protocol: pb::Protocol::Traderjoe as i32,
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

fn decode_packed_uint128(bytes: &[u8; 32]) -> (String, String) {
    let x = BigInt::from_unsigned_bytes_be(&bytes[..16]);
    let y = BigInt::from_unsigned_bytes_be(&bytes[16..]);
    (x.to_string(), y.to_string())
}

fn is_non_zero(value: &str) -> bool {
    !value.is_empty() && value.bytes().any(|byte| byte != b'0')
}
