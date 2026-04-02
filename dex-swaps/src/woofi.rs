use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::woofi as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

pub(crate) fn decode_log(_tx: &TransactionTrace, log: &Log) -> Vec<pb::log::Log> {
    let Some(event) = abi::wooppv2::events::WooSwap::match_and_decode(log) else {
        return Vec::new();
    };

    vec![pb::log::Log::Swap(pb::Swap {
        protocol: pb::Protocol::Woofi as i32,
        factory: log.address.clone(),
        pool: log.address.clone(),
        user: event.from.to_vec(),
        input_token: event.from_token.to_vec(),
        input_amount: event.from_amount.to_string(),
        output_token: event.to_token.to_vec(),
        output_amount: event.to_amount.to_string(),
    })]
}
