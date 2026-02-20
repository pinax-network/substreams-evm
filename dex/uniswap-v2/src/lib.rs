mod store;
use common::create::{CreateLog, CreateTransaction};
use proto::pb::uniswap::v2 as pb;
use substreams_abis::dex::uniswap::v2 as uniswap;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_swaps = 0;
    let mut total_mints = 0;
    let mut total_burns = 0;
    let mut total_syncs = 0;
    let mut total_pair_created = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
                trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
            } else {
                trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
            };
            for (log, call) in logs_with_calls {

            // Swap event
            if let Some(event) = uniswap::pair::events::Swap::match_and_decode(log) {
                total_swaps += 1;
                let event = pb::log::Log::Swap(pb::Swap {
                    sender: event.sender.to_vec(),
                    amount0_in: event.amount0_in.to_string(),
                    amount0_out: event.amount0_out.to_string(),
                    amount1_in: event.amount1_in.to_string(),
                    amount1_out: event.amount1_out.to_string(),
                    to: event.to.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Mint event
            if let Some(event) = uniswap::pair::events::Mint::match_and_decode(log) {
                total_mints += 1;
                let event = pb::log::Log::Mint(pb::Mint {
                    sender: event.sender.to_vec(),
                    amount0: event.amount0.to_string(),
                    amount1: event.amount1.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Burn event
            if let Some(event) = uniswap::pair::events::Burn::match_and_decode(log) {
                total_burns += 1;
                let event = pb::log::Log::Burn(pb::Burn {
                    sender: event.sender.to_vec(),
                    amount0: event.amount0.to_string(),
                    amount1: event.amount1.to_string(),
                    to: event.to.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Sync event
            if let Some(event) = uniswap::pair::events::Sync::match_and_decode(log) {
                total_syncs += 1;
                let event = pb::log::Log::Sync(pb::Sync {
                    reserve0: event.reserve0.to_string(),
                    reserve1: event.reserve1.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PairCreated event
            if let Some(event) = uniswap::factory::events::PairCreated::match_and_decode(log) {
                total_pair_created += 1;
                let event = pb::log::Log::PairCreated(pb::PairCreated {
                    token0: event.token0.to_vec(),
                    token1: event.token1.to_vec(),
                    pair: event.pair.to_vec(),
                    extra_data: event.extra_data.to_string(),
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
    substreams::log::info!("Total Swap events: {}", total_swaps);
    substreams::log::info!("Total Mint events: {}", total_mints);
    substreams::log::info!("Total Burn events: {}", total_burns);
    substreams::log::info!("Total Sync events: {}", total_syncs);
    substreams::log::info!("Total PairCreated events: {}", total_pair_created);
    Ok(events)
}
