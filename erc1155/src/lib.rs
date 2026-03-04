use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc1155::v1 as pb;
use substreams_abis::standard::erc1155::events as erc1155;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);
        for log_view in trx.receipt().logs() {
            let log = log_view.log;

            // TransferSingle event
            if let Some(event) = erc1155::TransferSingle::match_and_decode(log) {
                let event = pb::log::Log::TransferSingle(pb::TransferSingle {
                    operator: event.operator.to_vec(),
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    id: event.id.to_string(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // TransferBatch event
            if let Some(event) = erc1155::TransferBatch::match_and_decode(log) {
                let event = pb::log::Log::TransferBatch(pb::TransferBatch {
                    operator: event.operator.to_vec(),
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    ids: event.ids.iter().map(|id| id.to_string()).collect(),
                    values: event.values.iter().map(|value| value.to_string()).collect(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // ApprovalForAll event
            if let Some(event) = erc1155::ApprovalForAll::match_and_decode(log) {
                let event = pb::log::Log::ApprovalForAll(pb::ApprovalForAll {
                    account: event.account.to_vec(),
                    operator: event.operator.to_vec(),
                    approved: event.approved,
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // URI event
            if let Some(event) = erc1155::Uri::match_and_decode(log) {
                let event = pb::log::Log::Uri(pb::Uri {
                    value: event.value,
                    id: event.id.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    Ok(events)
}
