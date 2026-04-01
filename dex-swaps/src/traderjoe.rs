use proto::pb::dex::swaps::v1 as pb;
use substreams::scalar::BigInt;
use substreams_abis::dex::traderjoe as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;
use crate::utils::is_non_zero;

pub(crate) fn decode_log(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Vec<pb::log::Log> {
    if let Some(event) = abi::lbpair::events::Swap::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };
        let Some(token0) = pool.tokens.first() else {
            return Vec::new();
        };
        let Some(token1) = pool.tokens.get(1) else {
            return Vec::new();
        };
        let (amount_in_x, amount_in_y) = decode_packed_uint128(&event.amounts_in);
        let (amount_out_x, amount_out_y) = decode_packed_uint128(&event.amounts_out);

        let Some((input_token, input_amount, output_token, output_amount)) = (match (is_non_zero(&amount_in_x), is_non_zero(&amount_in_y)) {
            (true, false) => Some((token0.clone(), amount_in_x, token1.clone(), amount_out_y)),
            (false, true) => Some((token1.clone(), amount_in_y, token0.clone(), amount_out_x)),
            _ => None,
        }) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::Traderjoe as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: event.sender.to_vec(),
            input_token,
            input_amount,
            output_token,
            output_amount,
        })];
    }

    if let Some(event) = abi::lbfactory::events::LbPairCreated::match_and_decode(log) {
        return vec![pb::log::Log::Initialize(pb::Initialize {
            protocol: pb::Protocol::Traderjoe as i32,
            factory: log.address.clone(),
            pool: event.lb_pair.to_vec(),
        })];
    }

    Vec::new()
}

fn decode_packed_uint128(bytes: &[u8; 32]) -> (String, String) {
    let x = BigInt::from_unsigned_bytes_be(&bytes[..16]);
    let y = BigInt::from_unsigned_bytes_be(&bytes[16..]);
    (x.to_string(), y.to_string())
}
