use proto::pb::dex::swaps::v1 as pb;
use substreams_abis::dex::cow::gpv2settlement as abi;
use substreams_ethereum::{pb::eth::v2::{Log, TransactionTrace}, Event};

pub(crate) fn decode_swap(_tx: &TransactionTrace, log: &Log) -> Option<pb::Swap> {
    let event = abi::events::Trade::match_and_decode(log)?;

    Some(pb::Swap {
        protocol: pb::Protocol::Cow as i32,
        factory: log.address.clone(),
        pool: log.address.clone(),
        user: event.owner.to_vec(),
        input_token: event.sell_token.to_vec(),
        input_amount: event.sell_amount.to_string(),
        output_token: event.buy_token.to_vec(),
        output_amount: event.buy_amount.to_string(),
        log_ordinal: log.ordinal,
    })
}
