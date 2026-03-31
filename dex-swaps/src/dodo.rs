use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::dodo as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

pub(crate) fn decode_swap(_tx: &TransactionTrace, log: &Log) -> Option<pb::Swap> {
    let event = abi::v2::routeproxy::events::OrderHistory::match_and_decode(log)?;

    Some(pb::Swap {
        protocol: pb::Protocol::Dodo as i32,
        factory: log.address.clone(),
        pool: log.address.clone(),
        user: event.sender.to_vec(),
        input_token: event.from_token.to_vec(),
        input_amount: event.from_amount.to_string(),
        output_token: event.to_token.to_vec(),
        output_amount: event.return_amount.to_string(),
        log_ordinal: log.ordinal,
    })
}
