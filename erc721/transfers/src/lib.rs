use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc721::transfers::v1 as pb;
use substreams_abis::standard::erc721::events as erc721_events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);
        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> =
            if trx.calls.is_empty() {
                trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
            } else {
                trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
            };

        for (log, call) in logs_with_calls {
            if let Some(event) = erc721_events::Transfer::match_and_decode(log) {
                let event = pb::log::Log::Transfer(pb::Transfer {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    token_id: event.token_id.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = erc721_events::Approval::match_and_decode(log) {
                let event = pb::log::Log::Approval(pb::Approval {
                    owner: event.owner.to_vec(),
                    approved: event.approved.to_vec(),
                    token_id: event.token_id.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = erc721_events::ApprovalForAll::match_and_decode(log) {
                let event = pb::log::Log::ApprovalForAll(pb::ApprovalForAll {
                    owner: event.owner.to_vec(),
                    operator: event.operator.to_vec(),
                    approved: event.approved,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    Ok(events)
}
