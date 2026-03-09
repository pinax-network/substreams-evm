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
        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
            trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
        } else {
            trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
        };

        for (log, call) in logs_with_calls {

            // TransferSingle event
            if let Some(event) = erc1155::TransferSingle::match_and_decode(log) {
                let event = pb::log::Log::TransferSingle(pb::TransferSingle {
                    operator: event.operator.to_vec(),
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    id: event.id.to_string(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
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
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ApprovalForAll event
            if let Some(event) = erc1155::ApprovalForAll::match_and_decode(log) {
                let event = pb::log::Log::ApprovalForAll(pb::ApprovalForAll {
                    account: event.account.to_vec(),
                    operator: event.operator.to_vec(),
                    approved: event.approved,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // URI event
            if let Some(event) = erc1155::Uri::match_and_decode(log) {
                let event = pb::log::Log::Uri(pb::Uri {
                    value: event.value,
                    id: event.id.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    Ok(events)
}
