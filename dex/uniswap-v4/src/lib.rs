mod store;
use common::{bigint_to_i32, bigint_to_u64};
use proto::pb::dex::uniswap::v4 as pb;
use substreams_abis::evm::uniswap::v4 as uniswap;
use substreams_ethereum::pb::eth::v2::{Block, Log};
use substreams_ethereum::Event;

fn create_log(log: &Log, event: pb::log::Log) -> pb::Log {
    pb::Log {
        address: log.address.to_vec(),
        ordinal: log.ordinal,
        topics: log.topics.iter().map(|t| t.to_vec()).collect(),
        data: log.data.to_vec(),
        log: Some(event),
    }
}

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_swaps = 0;
    let mut total_initialize = 0;
    let mut total_modify_liquidity = 0;
    let mut total_donate = 0;
    let mut total_protocol_fee_controller_updated = 0;
    let mut total_protocol_fee_updated = 0;

    for trx in block.transactions() {
        let gas_price = trx.clone().gas_price.unwrap_or_default().with_decimal(0).to_string();
        let value = trx.clone().value.unwrap_or_default().with_decimal(0);
        let to = if trx.to.is_empty() { None } else { Some(trx.to.to_vec()) };
        let mut transaction = pb::Transaction {
            from: trx.from.to_vec(),
            to,
            hash: trx.hash.to_vec(),
            nonce: trx.nonce,
            gas_price,
            gas_limit: trx.gas_limit,
            gas_used: trx.receipt().receipt.cumulative_gas_used,
            value: value.to_string(),
            logs: vec![],
        };

        for log_view in trx.receipt().logs() {
            let log = log_view.log;

            // Swap event
            if let Some(event) = uniswap::poolmanager::events::Swap::match_and_decode(log) {
                total_swaps += 1;
                let event = pb::log::Log::Swap(pb::Swap {
                    id: event.id.to_vec(),
                    sender: event.sender.to_vec(),
                    amount0: event.amount0.to_string(),
                    amount1: event.amount1.to_string(),
                    sqrt_price_x96: event.sqrt_price_x96.to_string(),
                    liquidity: event.liquidity.to_string(),
                    tick: bigint_to_i32(&event.tick).unwrap_or_default(),
                    fee: event.fee.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // Initialize event
            if let Some(event) = uniswap::poolmanager::events::Initialize::match_and_decode(log) {
                total_initialize += 1;
                let event = pb::log::Log::Initialize(pb::Initialize {
                    factory: log.address.to_vec(),
                    id: event.id.to_vec(),
                    currency0: event.currency0.to_vec(),
                    currency1: event.currency1.to_vec(),
                    fee: bigint_to_u64(&event.fee).unwrap_or_default(),
                    tick_spacing: bigint_to_i32(&event.tick_spacing).unwrap_or_default(),
                    hooks: None, // NOT IMPLEMENTED
                    sqrt_price_x96: event.sqrt_price_x96.to_string(),
                    tick: bigint_to_i32(&event.tick).unwrap_or_default(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // ModifyLiquidity event
            if let Some(event) = uniswap::poolmanager::events::ModifyLiquidity::match_and_decode(log) {
                total_modify_liquidity += 1;
                let event = pb::log::Log::ModifyLiquidity(pb::ModifyLiquidity {
                    id: event.id.to_vec(),
                    sender: event.sender.to_vec(),
                    tick_lower: bigint_to_i32(&event.tick_lower).unwrap_or_default(),
                    tick_upper: bigint_to_i32(&event.tick_upper).unwrap_or_default(),
                    liquidity_delta: event.liquidity_delta.to_string(),
                    salt: event.salt.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // Donate event
            if let Some(event) = uniswap::poolmanager::events::Donate::match_and_decode(log) {
                total_donate += 1;
                let event = pb::log::Log::Donate(pb::Donate {
                    id: event.id.to_vec(),
                    sender: event.sender.to_vec(),
                    amount0: event.amount0.to_string(),
                    amount1: event.amount1.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // ProtocolFeeControllerUpdated event
            if let Some(event) = uniswap::poolmanager::events::ProtocolFeeControllerUpdated::match_and_decode(log) {
                total_protocol_fee_controller_updated += 1;
                let event = pb::log::Log::ProtocolFeeControllerUpdated(pb::ProtocolFeeControllerUpdated {
                    protocol_fee_controller: event.protocol_fee_controller.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // ProtocolFeeUpdated event
            if let Some(event) = uniswap::poolmanager::events::ProtocolFeeUpdated::match_and_decode(log) {
                total_protocol_fee_updated += 1;
                let event = pb::log::Log::ProtocolFeeUpdated(pb::ProtocolFeeUpdated {
                    id: event.id.to_vec(),
                    protocol_fee: bigint_to_u64(&event.protocol_fee).unwrap_or_default(),
                });
                transaction.logs.push(create_log(log, event));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total Swap events: {}", total_swaps);
    substreams::log::info!("Total Initialize events: {}", total_initialize);
    substreams::log::info!("Total ModifyLiquidity events: {}", total_modify_liquidity);
    substreams::log::info!("Total Donate events: {}", total_donate);
    substreams::log::info!("Total ProtocolFeeControllerUpdated events: {}", total_protocol_fee_controller_updated);
    substreams::log::info!("Total ProtocolFeeUpdated events: {}", total_protocol_fee_updated);
    Ok(events)
}
