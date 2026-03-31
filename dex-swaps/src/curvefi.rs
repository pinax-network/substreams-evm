use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::curvefi as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    if let Some(event) = abi::pool::events::TokenExchange::match_and_decode(log) {
        let pool = pools.get(log.address.as_slice())?;
        let input_token = parse_curve_token(&pool.tokens, &event.sold_id.to_string())?;
        let output_token = parse_curve_token(&pool.tokens, &event.bought_id.to_string())?;

        return Some(pb::Swap {
            protocol: pb::Protocol::Curvefi as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: event.buyer.to_vec(),
            input_token,
            input_amount: event.tokens_sold.to_string(),
            output_token,
            output_amount: event.tokens_bought.to_string(),
            log_ordinal: log.ordinal,
        });
    }

    if let Some(event) = abi::cryptoswap::events::TokenExchange::match_and_decode(log) {
        let pool = pools.get(log.address.as_slice())?;
        let input_token = parse_curve_token(&pool.tokens, &event.sold_id.to_string())?;
        let output_token = parse_curve_token(&pool.tokens, &event.bought_id.to_string())?;

        return Some(pb::Swap {
            protocol: pb::Protocol::Curvefi as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: event.buyer.to_vec(),
            input_token,
            input_amount: event.tokens_sold.to_string(),
            output_token,
            output_amount: event.tokens_bought.to_string(),
            log_ordinal: log.ordinal,
        });
    }

    None
}

fn parse_curve_token(tokens: &[Vec<u8>], index: &str) -> Option<Vec<u8>> {
    let index = index.parse::<usize>().ok()?;
    tokens.get(index).cloned()
}
