use common::create::{CreateLog, CreateTransaction};
use proto::pb::woofi::v1 as pb;
use substreams_abis::dex::woofi;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_woo_swaps = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
                trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
            } else {
                trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
            };
            for (log, call) in logs_with_calls {

            // WooSwap event
            if let Some(event) = woofi::wooppv2::events::WooSwap::match_and_decode(log) {
                total_woo_swaps += 1;
                let event = pb::log::Log::WooSwap(pb::WooSwap {
                    from_token: event.from_token.to_vec(),
                    to_token: event.to_token.to_vec(),
                    from_amount: event.from_amount.to_string(),
                    to_amount: event.to_amount.to_string(),
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    rebate_to: event.rebate_to.to_vec(),
                    swap_vol: event.swap_vol.to_string(),
                    swap_fee: event.swap_fee.to_string(),
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
    substreams::log::info!("Total WooSwap events: {}", total_woo_swaps);
    Ok(events)
}
