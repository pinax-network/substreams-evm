use common::bigint_to_u64;
use proto::pb::bancor::v1 as pb;
use substreams_abis::evm::bancor::bancorconverterfactory;
use substreams_abis::evm::bancor::converterfactory;
use substreams_abis::evm::bancor::converterregistry;
use substreams_abis::evm::bancor::standardpoolconverter as bancor;
use substreams_ethereum::pb::eth::v2::{Block, Log};
use substreams_ethereum::Event;

pub mod store;

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
    let mut total_activations = 0;
    let mut total_conversions = 0;
    let mut total_conversion_fee_updates = 0;
    let mut total_liquidity_added = 0;
    let mut total_liquidity_removed = 0;
    let mut total_owner_updates = 0;
    let mut total_token_rate_updates = 0;
    let mut total_converter_anchor_added = 0;
    let mut total_converter_anchor_removed = 0;
    let mut total_convertible_token_added = 0;
    let mut total_convertible_token_removed = 0;
    let mut total_liquidity_pool_added = 0;
    let mut total_liquidity_pool_removed = 0;
    let mut total_smart_token_added = 0;
    let mut total_smart_token_removed = 0;
    let mut total_new_converter = 0;

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

            // Activation event
            if let Some(event) = bancor::events::Activation::match_and_decode(log) {
                total_activations += 1;
                let event = pb::log::Log::Activation(pb::Activation {
                    factory: log.address.to_vec(),
                    converter_type: bigint_to_u64(&event.converter_type).unwrap_or_default() as u32,
                    anchor: event.anchor.to_vec(),
                    activated: event.activated,
                });
                transaction.logs.push(create_log(log, event));
            }

            // Conversion event
            if let Some(event) = bancor::events::Conversion::match_and_decode(log) {
                total_conversions += 1;
                let event = pb::log::Log::Conversion(pb::Conversion {
                    source_token: event.source_token.to_vec(),
                    target_token: event.target_token.to_vec(),
                    trader: event.trader.to_vec(),
                    source_amount: event.source_amount.to_string(),
                    target_amount: event.target_amount.to_string(),
                    conversion_fee: event.conversion_fee.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // ConversionFeeUpdate event
            if let Some(event) = bancor::events::ConversionFeeUpdate::match_and_decode(log) {
                total_conversion_fee_updates += 1;
                let event = pb::log::Log::ConversionFeeUpdate(pb::ConversionFeeUpdate {
                    prev_fee: bigint_to_u64(&event.prev_fee).unwrap_or_default() as u32,
                    new_fee: bigint_to_u64(&event.new_fee).unwrap_or_default() as u32,
                });
                transaction.logs.push(create_log(log, event));
            }

            // LiquidityAdded event
            if let Some(event) = bancor::events::LiquidityAdded::match_and_decode(log) {
                total_liquidity_added += 1;
                let event = pb::log::Log::LiquidityAdded(pb::LiquidityAdded {
                    provider: event.provider.to_vec(),
                    reserve_token: event.reserve_token.to_vec(),
                    amount: event.amount.to_string(),
                    new_balance: event.new_balance.to_string(),
                    new_supply: event.new_supply.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // LiquidityRemoved event
            if let Some(event) = bancor::events::LiquidityRemoved::match_and_decode(log) {
                total_liquidity_removed += 1;
                let event = pb::log::Log::LiquidityRemoved(pb::LiquidityRemoved {
                    provider: event.provider.to_vec(),
                    reserve_token: event.reserve_token.to_vec(),
                    amount: event.amount.to_string(),
                    new_balance: event.new_balance.to_string(),
                    new_supply: event.new_supply.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // OwnerUpdate event
            if let Some(event) = bancor::events::OwnerUpdate::match_and_decode(log) {
                total_owner_updates += 1;
                let event = pb::log::Log::OwnerUpdate(pb::OwnerUpdate {
                    prev_owner: event.prev_owner.to_vec(),
                    new_owner: event.new_owner.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // TokenRateUpdate event
            if let Some(event) = bancor::events::TokenRateUpdate::match_and_decode(log) {
                total_token_rate_updates += 1;
                let event = pb::log::Log::TokenRateUpdate(pb::TokenRateUpdate {
                    token1: event.token1.to_vec(),
                    token2: event.token2.to_vec(),
                    rate_n: event.rate_n.to_string(),
                    rate_d: event.rate_d.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // ConverterAnchorAdded event
            if let Some(event) = converterregistry::events::ConverterAnchorAdded::match_and_decode(log) {
                total_converter_anchor_added += 1;
                let event = pb::log::Log::ConverterAnchorAdded(pb::ConverterAnchorAdded { anchor: event.anchor.to_vec() });
                transaction.logs.push(create_log(log, event));
            }

            // ConverterAnchorRemoved event
            if let Some(event) = converterregistry::events::ConverterAnchorRemoved::match_and_decode(log) {
                total_converter_anchor_removed += 1;
                let event = pb::log::Log::ConverterAnchorRemoved(pb::ConverterAnchorRemoved { anchor: event.anchor.to_vec() });
                transaction.logs.push(create_log(log, event));
            }

            // ConvertibleTokenAdded event
            if let Some(event) = converterregistry::events::ConvertibleTokenAdded::match_and_decode(log) {
                total_convertible_token_added += 1;
                let event = pb::log::Log::ConvertibleTokenAdded(pb::ConvertibleTokenAdded {
                    convertible_token: event.convertible_token.to_vec(),
                    smart_token: event.smart_token.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // ConvertibleTokenRemoved event
            if let Some(event) = converterregistry::events::ConvertibleTokenRemoved::match_and_decode(log) {
                total_convertible_token_removed += 1;
                let event = pb::log::Log::ConvertibleTokenRemoved(pb::ConvertibleTokenRemoved {
                    convertible_token: event.convertible_token.to_vec(),
                    smart_token: event.smart_token.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // LiquidityPoolAdded event
            if let Some(event) = converterregistry::events::LiquidityPoolAdded::match_and_decode(log) {
                total_liquidity_pool_added += 1;
                let event = pb::log::Log::LiquidityPoolAdded(pb::LiquidityPoolAdded {
                    liquidity_pool: event.liquidity_pool.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // LiquidityPoolRemoved event
            if let Some(event) = converterregistry::events::LiquidityPoolRemoved::match_and_decode(log) {
                total_liquidity_pool_removed += 1;
                let event = pb::log::Log::LiquidityPoolRemoved(pb::LiquidityPoolRemoved {
                    liquidity_pool: event.liquidity_pool.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // SmartTokenAdded event
            if let Some(event) = converterregistry::events::SmartTokenAdded::match_and_decode(log) {
                total_smart_token_added += 1;
                let event = pb::log::Log::SmartTokenAdded(pb::SmartTokenAdded {
                    smart_token: event.smart_token.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // SmartTokenRemoved event
            if let Some(event) = converterregistry::events::SmartTokenRemoved::match_and_decode(log) {
                total_smart_token_removed += 1;
                let event = pb::log::Log::SmartTokenRemoved(pb::SmartTokenRemoved {
                    smart_token: event.smart_token.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // NewConverter event
            if let Some(event) = converterfactory::events::NewConverter::match_and_decode(log) {
                total_new_converter += 1;
                let event = pb::log::Log::NewConverter(pb::NewConverter {
                    factory: log.address.to_vec(),
                    converter_type: bigint_to_u64(&event.converter_type).unwrap_or_default() as u32,
                    converter: event.converter.to_vec(),
                    owner: event.owner.to_vec(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // NewConverter event from BancorConverterFactory (Legacy)
            if let Some(event) = bancorconverterfactory::events::NewConverter::match_and_decode(log) {
                total_new_converter += 1;
                let event = pb::log::Log::NewConverter(pb::NewConverter {
                    factory: log.address.to_vec(),
                    converter_type: 1 as u32,
                    converter: event.converter.to_vec(),
                    owner: event.owner.to_vec(),
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
    substreams::log::info!("Total Activation events: {}", total_activations);
    substreams::log::info!("Total Conversion events: {}", total_conversions);
    substreams::log::info!("Total ConversionFeeUpdate events: {}", total_conversion_fee_updates);
    substreams::log::info!("Total LiquidityAdded events: {}", total_liquidity_added);
    substreams::log::info!("Total LiquidityRemoved events: {}", total_liquidity_removed);
    substreams::log::info!("Total OwnerUpdate events: {}", total_owner_updates);
    substreams::log::info!("Total TokenRateUpdate events: {}", total_token_rate_updates);
    substreams::log::info!("Total ConverterAnchorAdded events: {}", total_converter_anchor_added);
    substreams::log::info!("Total ConverterAnchorRemoved events: {}", total_converter_anchor_removed);
    substreams::log::info!("Total ConvertibleTokenAdded events: {}", total_convertible_token_added);
    substreams::log::info!("Total ConvertibleTokenRemoved events: {}", total_convertible_token_removed);
    substreams::log::info!("Total LiquidityPoolAdded events: {}", total_liquidity_pool_added);
    substreams::log::info!("Total LiquidityPoolRemoved events: {}", total_liquidity_pool_removed);
    substreams::log::info!("Total SmartTokenAdded events: {}", total_smart_token_added);
    substreams::log::info!("Total SmartTokenRemoved events: {}", total_smart_token_removed);
    substreams::log::info!("Total NewConverter events: {}", total_new_converter);
    Ok(events)
}
