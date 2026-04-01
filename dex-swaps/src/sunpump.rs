use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::sunpump as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event, NULL_ADDRESS};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    if let Some(event) = abi::v1::launchpadproxy::events::TokenPurchased::match_and_decode(log) {
        let pool = pools.get(event.token.as_slice())?;

        return Some(pb::Swap {
            protocol: pb::Protocol::Sunpump as i32,
            factory: pool.factory.clone(),
            pool: event.token.to_vec(),
            user: event.buyer.to_vec(),
            input_token: NULL_ADDRESS.to_vec(),
            input_amount: event.trx_amount.to_string(),
            output_token: event.token.to_vec(),
            output_amount: event.token_amount.to_string(),
        });
    }

    if let Some(event) = abi::v1::launchpadproxy::events::TokenSold::match_and_decode(log) {
        let pool = pools.get(event.token.as_slice())?;

        return Some(pb::Swap {
            protocol: pb::Protocol::Sunpump as i32,
            factory: pool.factory.clone(),
            pool: event.token.to_vec(),
            user: event.seller.to_vec(),
            input_token: event.token.to_vec(),
            input_amount: event.token_amount.to_string(),
            output_token: NULL_ADDRESS.to_vec(),
            output_amount: event.trx_amount.to_string(),
        });
    }

    None
}
