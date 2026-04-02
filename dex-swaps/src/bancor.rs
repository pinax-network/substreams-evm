use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::bancor::bancorconverterfactory;
use substreams_abis::dex::bancor::converterfactory;
use substreams_abis::dex::bancor::standardpoolconverter as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

use crate::logs::PoolMetadataMap;

pub(crate) fn decode_log(_tx: &TransactionTrace, log: &Log, pools: &PoolMetadataMap) -> Vec<pb::log::Log> {
    if let Some(event) = abi::events::Conversion::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::Swap(pb::Swap {
            protocol: pb::Protocol::Bancor as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            user: event.trader.to_vec(),
            input_token: event.source_token.to_vec(),
            input_amount: event.source_amount.to_string(),
            output_token: event.target_token.to_vec(),
            output_amount: event.target_amount.to_string(),
        })];
    }

    if let Some(event) = abi::events::ConversionFeeUpdate::match_and_decode(log) {
        let Some(pool) = pools.get(log.address.as_slice()) else {
            return Vec::new();
        };

        return vec![pb::log::Log::SwapFee(pb::SwapFee {
            protocol: pb::Protocol::Bancor as i32,
            factory: pool.factory.clone(),
            pool: log.address.clone(),
            fee: event.new_fee.to_u64() as u32,
        })];
    }

    if let Some(event) = converterfactory::events::NewConverter::match_and_decode(log) {
        return vec![pb::log::Log::Initialize(pb::Initialize {
            protocol: pb::Protocol::Bancor as i32,
            factory: log.address.clone(),
            pool: event.converter.to_vec(),
        })];
    }

    if let Some(event) = bancorconverterfactory::events::NewConverter::match_and_decode(log) {
        return vec![pb::log::Log::Initialize(pb::Initialize {
            protocol: pb::Protocol::Bancor as i32,
            factory: log.address.clone(),
            pool: event.converter.to_vec(),
        })];
    }

    Vec::new()
}
