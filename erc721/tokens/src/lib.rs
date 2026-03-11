use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc721::tokens::v1 as pb;
use substreams_abis::tokens::erc20::cryptopunks::events as cryptopunks_events;
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
            if let Some(event) = cryptopunks_events::Assign::match_and_decode(log) {
                let event = pb::log::Log::Assign(pb::Assign {
                    to: event.to.to_vec(),
                    punk_index: event.punk_index.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = cryptopunks_events::PunkTransfer::match_and_decode(log) {
                let event = pb::log::Log::PunkTransfer(pb::PunkTransfer {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    punk_index: event.punk_index.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = cryptopunks_events::PunkBought::match_and_decode(log) {
                let event = pb::log::Log::PunkBought(pb::PunkBought {
                    punk_index: event.punk_index.to_string(),
                    value: Some(event.value.to_string()),
                    from_address: event.from_address.to_vec(),
                    to_address: event.to_address.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = cryptopunks_events::PunkOffered::match_and_decode(log) {
                let event = pb::log::Log::PunkOffered(pb::PunkOffered {
                    punk_index: event.punk_index.to_string(),
                    min_value: event.min_value.to_string(),
                    to_address: event.to_address.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = cryptopunks_events::PunkBidEntered::match_and_decode(log) {
                let event = pb::log::Log::PunkBidEntered(pb::PunkBidEntered {
                    punk_index: event.punk_index.to_string(),
                    value: event.value.to_string(),
                    from_address: event.from_address.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = cryptopunks_events::PunkBidWithdrawn::match_and_decode(log) {
                let event = pb::log::Log::PunkBidWithdrawn(pb::PunkBidWithdrawn {
                    punk_index: event.punk_index.to_string(),
                    value: event.value.to_string(),
                    from_address: event.from_address.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = cryptopunks_events::PunkNoLongerForSale::match_and_decode(log) {
                let event = pb::log::Log::PunkNoLongerForSale(pb::PunkNoLongerForSale {
                    punk_index: event.punk_index.to_string(),
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
