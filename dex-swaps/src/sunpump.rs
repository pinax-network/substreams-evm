use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::sunpump as abi;
use substreams_abis::dex::sunpump::legacy::launchpad::events::TokenCreate as TokenCreateLegacy;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event, NULL_ADDRESS};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_log(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Vec<pb::log::Log> {
    if let Some(event) = abi::v1::launchpadproxy::events::TokenPurchased::match_and_decode(log) {
        let Some(pool) = pools.get(event.token.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::Sunpump as i32,
            factory: pool.factory.clone(),
            pool: event.token.to_vec(),
            user: event.buyer.to_vec(),
            input_token: NULL_ADDRESS.to_vec(),
            input_amount: event.trx_amount.to_string(),
            output_token: event.token.to_vec(),
            output_amount: event.token_amount.to_string(),
        })];
    }

    if let Some(event) = abi::v1::launchpadproxy::events::TokenSold::match_and_decode(log) {
        let Some(pool) = pools.get(event.token.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::Sunpump as i32,
            factory: pool.factory.clone(),
            pool: event.token.to_vec(),
            user: event.seller.to_vec(),
            input_token: event.token.to_vec(),
            input_amount: event.token_amount.to_string(),
            output_token: NULL_ADDRESS.to_vec(),
            output_amount: event.trx_amount.to_string(),
        })];
    }

    if let Some(event) = abi::v1::launchpadproxy::events::TokenCreate::match_and_decode(log) {
        return vec![pb::log::Log::Initialize(pb::Initialize {
            protocol: pb::Protocol::Sunpump as i32,
            factory: log.address.clone(),
            pool: event.token_address.to_vec(),
        })];
    }

    if let Some(event) = TokenCreateLegacy::match_and_decode(log) {
        return vec![pb::log::Log::Initialize(pb::Initialize {
            protocol: pb::Protocol::Sunpump as i32,
            factory: log.address.clone(),
            pool: event.token_address.to_vec(),
        })];
    }

    if let Some(event) = abi::v1::launchpadproxy::events::PurchaseFeeSet::match_and_decode(log) {
        return vec![pb::log::Log::SwapFee(pb::SwapFee {
            protocol: pb::Protocol::Sunpump as i32,
            factory: log.address.clone(),
            pool: log.address.clone(),
            fee: event.new_fee.to_u64() as u32,
        })];
    }

    if let Some(event) = abi::v1::launchpadproxy::events::SaleFeeSet::match_and_decode(log) {
        return vec![pb::log::Log::SwapFee(pb::SwapFee {
            protocol: pb::Protocol::Sunpump as i32,
            factory: log.address.clone(),
            pool: log.address.clone(),
            fee: event.new_fee.to_u64() as u32,
        })];
    }

    Vec::new()
}
