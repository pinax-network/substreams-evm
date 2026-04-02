use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::balancer as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;
use crate::utils::fixed_1e18_to_bps;

pub(crate) fn decode_log(tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Vec<pb::log::Log> {
    if let Some(event) = abi::v3::vault::events::Swap::match_and_decode(log) {
        let Some(pool) = pools.get(event.pool.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::Balancer as i32,
            factory: pool.factory.clone(),
            pool: event.pool.to_vec(),
            user: tx.from.to_vec(),
            input_token: event.token_in.to_vec(),
            input_amount: event.amount_in.to_string(),
            output_token: event.token_out.to_vec(),
            output_amount: event.amount_out.to_string(),
        })];
    }

    if let Some(event) = abi::v3::vault::events::PoolRegistered::match_and_decode(log) {
        return vec![pb::log::Log::Initialize(pb::Initialize {
            protocol: pb::Protocol::Balancer as i32,
            factory: event.factory.to_vec(),
            pool: event.pool.to_vec(),
        })];
    }

    if let Some(event) = abi::v2::weightedpool::events::SwapFeePercentageChanged::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::SwapFee(pb::SwapFee {
            protocol: pb::Protocol::Balancer as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            fee: fixed_1e18_to_bps(&event.swap_fee_percentage),
        })];
    }

    if let Some(event) = abi::v3::vault::events::AggregateSwapFeePercentageChanged::match_and_decode(log) {
        let Some(pool) = pools.get(event.pool.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::SwapFee(pb::SwapFee {
            protocol: pb::Protocol::Balancer as i32,
            factory: pool.factory.clone(),
            pool: event.pool.to_vec(),
            fee: fixed_1e18_to_bps(&event.aggregate_swap_fee_percentage),
        })];
    }

    Vec::new()
}
