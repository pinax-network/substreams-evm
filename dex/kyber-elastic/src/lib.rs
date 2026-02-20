mod store;
use common::create::{CreateLog, CreateTransaction};
use common::bigint_to_i32;
use proto::pb::kyber_elastic::v1 as pb;
use substreams_abis::dex::kyber::elastic;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_swaps = 0;
    let mut total_mints = 0;
    let mut total_burns = 0;
    let mut total_burn_r_tokens = 0;
    let mut total_flash = 0;
    let mut total_initialize = 0;
    let mut total_pool_created = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        for log_view in trx.receipt().logs() {
            let log = log_view.log;

            // Swap event (Pool)
            if let Some(event) = elastic::pool::events::Swap::match_and_decode(log) {
                total_swaps += 1;
                let event = pb::log::Log::Swap(pb::Swap {
                    sender: event.sender.to_vec(),
                    recipient: event.recipient.to_vec(),
                    delta_qty0: event.delta_qty0.to_string(),
                    delta_qty1: event.delta_qty1.to_string(),
                    sqrt_p: event.sqrt_p.to_string(),
                    liquidity: event.liquidity.to_string(),
                    current_tick: bigint_to_i32(&event.current_tick).unwrap_or_default(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // Mint event (Pool)
            if let Some(event) = elastic::pool::events::Mint::match_and_decode(log) {
                total_mints += 1;
                let event = pb::log::Log::Mint(pb::Mint {
                    sender: event.sender.to_vec(),
                    owner: event.owner.to_vec(),
                    tick_lower: bigint_to_i32(&event.tick_lower).unwrap_or_default(),
                    tick_upper: bigint_to_i32(&event.tick_upper).unwrap_or_default(),
                    qty: event.qty.to_string(),
                    qty0: event.qty0.to_string(),
                    qty1: event.qty1.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // Burn event (Pool)
            if let Some(event) = elastic::pool::events::Burn::match_and_decode(log) {
                total_burns += 1;
                let event = pb::log::Log::Burn(pb::Burn {
                    owner: event.owner.to_vec(),
                    tick_lower: bigint_to_i32(&event.tick_lower).unwrap_or_default(),
                    tick_upper: bigint_to_i32(&event.tick_upper).unwrap_or_default(),
                    qty: event.qty.to_string(),
                    qty0: event.qty0.to_string(),
                    qty1: event.qty1.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // BurnRTokens event (Pool)
            if let Some(event) = elastic::pool::events::BurnRTokens::match_and_decode(log) {
                total_burn_r_tokens += 1;
                let event = pb::log::Log::BurnRTokens(pb::BurnRTokens {
                    owner: event.owner.to_vec(),
                    qty: event.qty.to_string(),
                    qty0: event.qty0.to_string(),
                    qty1: event.qty1.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // Flash event (Pool)
            if let Some(event) = elastic::pool::events::Flash::match_and_decode(log) {
                total_flash += 1;
                let event = pb::log::Log::Flash(pb::Flash {
                    sender: event.sender.to_vec(),
                    recipient: event.recipient.to_vec(),
                    qty0: event.qty0.to_string(),
                    qty1: event.qty1.to_string(),
                    paid0: event.paid0.to_string(),
                    paid1: event.paid1.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // Initialize event (Pool)
            if let Some(event) = elastic::pool::events::Initialize::match_and_decode(log) {
                total_initialize += 1;
                let event = pb::log::Log::Initialize(pb::Initialize {
                    sqrt_p: event.sqrt_p.to_string(),
                    tick: bigint_to_i32(&event.tick).unwrap_or_default(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // PoolCreated event (Factory)
            if let Some(event) = elastic::factory::events::PoolCreated::match_and_decode(log) {
                total_pool_created += 1;
                let event = pb::log::Log::PoolCreated(pb::PoolCreated {
                    token0: event.token0.to_vec(),
                    token1: event.token1.to_vec(),
                    swap_fee_units: event.swap_fee_units.to_u64() as u32,
                    tick_distance: bigint_to_i32(&event.tick_distance).unwrap_or_default(),
                    pool: event.pool.to_vec(),
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
    substreams::log::info!("Total Swap events: {}", total_swaps);
    substreams::log::info!("Total Mint events: {}", total_mints);
    substreams::log::info!("Total Burn events: {}", total_burns);
    substreams::log::info!("Total BurnRTokens events: {}", total_burn_r_tokens);
    substreams::log::info!("Total Flash events: {}", total_flash);
    substreams::log::info!("Total Initialize events: {}", total_initialize);
    substreams::log::info!("Total PoolCreated events: {}", total_pool_created);
    Ok(events)
}
