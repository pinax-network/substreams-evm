use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::balancer as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_swap(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Option<pb::Swap> {
    let event = abi::v3::vault::events::Swap::match_and_decode(log)?;
    let pool = pools.get(event.pool.as_slice())?;

    Some(pb::Swap {
        protocol: pb::Protocol::Balancer as i32,
        factory: pool.factory.clone(),
        pool: event.pool.to_vec(),
        user: tx.from.to_vec(),
        input_token: event.token_in.to_vec(),
        input_amount: event.amount_in.to_string(),
        output_token: event.token_out.to_vec(),
        output_amount: event.amount_out.to_string(),
    })
}
