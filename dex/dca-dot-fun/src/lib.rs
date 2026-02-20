mod store;

use common::create::{CreateLog, CreateTransaction};
use proto::pb::dca_dot_fun::v1 as pb;
use substreams_abis::dex::dca_dot_fun;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_fill_orders = 0;
    let mut total_create_orders = 0;
    let mut total_cancel_orders = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
                trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
            } else {
                trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
            };
            for (log, call) in logs_with_calls {

            // FillOrder event (swap execution)
            if let Some(event) = dca_dot_fun::dcadotfun::events::FillOrder::match_and_decode(log) {
                total_fill_orders += 1;
                let event = pb::log::Log::FillOrder(pb::FillOrder {
                    order_id: event.order_id.to_string(),
                    caller: event.caller.to_vec(),
                    recipient: event.recipient.to_vec(),
                    fill_amount: event.fill_amount.to_string(),
                    amount_of_token_out: event.amount_of_token_out.to_string(),
                    protocol_fee: event.protocol_fee.to_string(),
                    token_in_price: event.token_in_price.to_string(),
                    token_out_price: event.token_out_price.to_string(),
                    scaling_factor: event.scaling_factor.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // CreateOrder event
            if let Some(event) = dca_dot_fun::dcadotfun::events::CreateOrder::match_and_decode(log) {
                total_create_orders += 1;
                let event = pb::log::Log::CreateOrder(pb::CreateOrder {
                    order_id: event.order_id.to_string(),
                    creator: event.creator.to_vec(),
                    recipient: event.recipient.to_vec(),
                    token_in: event.token_in.to_vec(),
                    token_out: event.token_out.to_vec(),
                    spend_amount: event.spend_amount.to_string(),
                    repeats: event.repeats.to_string(),
                    slippage: event.slippage.to_string(),
                    freq_interval: event.freq_interval.to_string(),
                    scaling_interval: event.scaling_interval.to_string(),
                    last_run: event.last_run.to_string(),
                    protocol_fee: event.protocol_fee.to_string(),
                    vault: event.vault.to_vec(),
                    stake_asset_in: event.stake_asset_in,
                    stake_asset_out: event.stake_asset_out,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // CancelOrder event
            if let Some(event) = dca_dot_fun::dcadotfun::events::CancelOrder::match_and_decode(log) {
                total_cancel_orders += 1;
                let event = pb::log::Log::CancelOrder(pb::CancelOrder {
                    order_id: event.order_id.to_string(),
                    vault: event.vault.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total FillOrder events: {}", total_fill_orders);
    substreams::log::info!("Total CreateOrder events: {}", total_create_orders);
    substreams::log::info!("Total CancelOrder events: {}", total_cancel_orders);
    Ok(events)
}
