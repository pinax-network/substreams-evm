use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::bancor::standardpoolconverter as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    let event = abi::events::Conversion::match_and_decode(log)?;
    let pool = pools.get(log.address.as_slice())?;

    Some(pb::Swap {
        protocol: pb::Protocol::Bancor as i32,
        factory: pool.factory.clone(),
        pool: log.address.clone(),
        user: event.trader.to_vec(),
        input_token: event.source_token.to_vec(),
        input_amount: event.source_amount.to_string(),
        output_token: event.target_token.to_vec(),
        output_amount: event.target_amount.to_string(),
        log_ordinal: log.ordinal,
    })
}
