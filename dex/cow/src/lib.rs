use common::create::{CreateLog, CreateTransaction};
use proto::pb::cow::v1 as pb;
use substreams_abis::dex::cow::gpv2settlement as cow;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_interactions = 0;
    let mut total_order_invalidated = 0;
    let mut total_pre_signatures = 0;
    let mut total_settlements = 0;
    let mut total_trades = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        for log_view in trx.receipt().logs() {
            let log = log_view.log;

            // Interaction event
            if let Some(event) = cow::events::Interaction::match_and_decode(log) {
                total_interactions += 1;
                let event = pb::log::Log::Interaction(pb::Interaction {
                    target: event.target.to_vec(),
                    value: event.value.to_string(),
                    selector: event.selector.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // OrderInvalidated event
            if let Some(event) = cow::events::OrderInvalidated::match_and_decode(log) {
                total_order_invalidated += 1;
                let event = pb::log::Log::OrderInvalidated(pb::OrderInvalidated {
                    owner: event.owner.to_vec(),
                    order_uid: event.order_uid.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // PreSignature event
            if let Some(event) = cow::events::PreSignature::match_and_decode(log) {
                total_pre_signatures += 1;
                let event = pb::log::Log::PreSignature(pb::PreSignature {
                    owner: event.owner.to_vec(),
                    order_uid: event.order_uid.to_vec(),
                    signed: event.signed,
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // Settlement event
            if let Some(event) = cow::events::Settlement::match_and_decode(log) {
                total_settlements += 1;
                let event = pb::log::Log::Settlement(pb::Settlement { solver: event.solver.to_vec() });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // Trade event
            if let Some(event) = cow::events::Trade::match_and_decode(log) {
                total_trades += 1;
                let event = pb::log::Log::Trade(pb::Trade {
                    owner: event.owner.to_vec(),
                    sell_token: event.sell_token.to_vec(),
                    buy_token: event.buy_token.to_vec(),
                    sell_amount: event.sell_amount.to_string(),
                    buy_amount: event.buy_amount.to_string(),
                    fee_amount: event.fee_amount.to_string(),
                    order_uid: event.order_uid.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total Interaction events: {}", total_interactions);
    substreams::log::info!("Total OrderInvalidated events: {}", total_order_invalidated);
    substreams::log::info!("Total PreSignature events: {}", total_pre_signatures);
    substreams::log::info!("Total Settlement events: {}", total_settlements);
    substreams::log::info!("Total Trade events: {}", total_trades);
    Ok(events)
}
