use common::create::{CreateLog, CreateTransaction};
use proto::pb::balancer::v1 as pb;
use substreams_abis::dex::balancer;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

pub mod store;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();

    // V2 WeightedPool counters
    let mut total_swap_fee_percentage = 0;
    let mut total_paused = 0;
    let mut total_recovery_mode = 0;
    let mut total_protocol_fee_percentage = 0;

    // V3 StablePool counters
    let mut total_amp_update_started = 0;
    let mut total_amp_update_stopped = 0;

    // V3 Vault counters
    let mut total_vault_swaps = 0;
    let mut total_liquidity_added = 0;
    let mut total_liquidity_removed = 0;
    let mut total_pool_initialized = 0;
    let mut total_pool_registered = 0;
    let mut total_pool_paused = 0;
    let mut total_pool_recovery_mode = 0;
    let mut total_aggregate_swap_fee_percentage = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
                trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
            } else {
                trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
            };
            for (log, call) in logs_with_calls {

            // ===== V2 WeightedPool Events =====
            // SwapFeePercentageChanged event
            if let Some(event) = balancer::v2::weightedpool::events::SwapFeePercentageChanged::match_and_decode(log) {
                total_swap_fee_percentage += 1;
                let event = pb::log::Log::SwapFeePercentage(pb::SwapFeePercentage {
                    swap_fee_percentage: event.swap_fee_percentage.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PausedStateChanged event
            if let Some(event) = balancer::v2::weightedpool::events::PausedStateChanged::match_and_decode(log) {
                total_paused += 1;
                let event = pb::log::Log::Paused(pb::Paused { paused: event.paused });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RecoveryModeStateChanged event
            if let Some(event) = balancer::v2::weightedpool::events::RecoveryModeStateChanged::match_and_decode(log) {
                total_recovery_mode += 1;
                let event = pb::log::Log::RecoveryMode(pb::RecoveryMode { enabled: event.enabled });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ProtocolFeePercentageCacheUpdated event
            if let Some(event) = balancer::v2::weightedpool::events::ProtocolFeePercentageCacheUpdated::match_and_decode(log) {
                total_protocol_fee_percentage += 1;
                let event = pb::log::Log::ProtocolFeePercentage(pb::ProtocolFeePercentage {
                    fee_type: event.fee_type.to_string(),
                    protocol_fee_percentage: event.protocol_fee_percentage.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ===== V3 StablePool Events =====

            // AmpUpdateStarted event
            if let Some(event) = balancer::v3::stablepool::events::AmpUpdateStarted::match_and_decode(log) {
                total_amp_update_started += 1;
                let event = pb::log::Log::AmpUpdateStarted(pb::AmpUpdateStarted {
                    start_value: event.start_value.to_string(),
                    end_value: event.end_value.to_string(),
                    start_time: event.start_time.to_string(),
                    end_time: event.end_time.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // AmpUpdateStopped event
            if let Some(event) = balancer::v3::stablepool::events::AmpUpdateStopped::match_and_decode(log) {
                total_amp_update_stopped += 1;
                let event = pb::log::Log::AmpUpdateStopped(pb::AmpUpdateStopped {
                    current_value: event.current_value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ===== V3 Vault Events =====

            // Swap event
            if let Some(event) = balancer::v3::vault::events::Swap::match_and_decode(log) {
                total_vault_swaps += 1;
                let event = pb::log::Log::VaultSwap(pb::VaultSwap {
                    pool: event.pool.to_vec(),
                    token_in: event.token_in.to_vec(),
                    token_out: event.token_out.to_vec(),
                    amount_in: event.amount_in.to_string(),
                    amount_out: event.amount_out.to_string(),
                    swap_fee_percentage: event.swap_fee_percentage.to_string(),
                    swap_fee_amount: event.swap_fee_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LiquidityAdded event
            if let Some(event) = balancer::v3::vault::events::LiquidityAdded::match_and_decode(log) {
                total_liquidity_added += 1;
                let event = pb::log::Log::LiquidityAdded(pb::LiquidityAdded {
                    pool: event.pool.to_vec(),
                    liquidity_provider: event.liquidity_provider.to_vec(),
                    kind: event.kind.to_u64().min(u32::MAX as u64) as u32,
                    total_supply: event.total_supply.to_string(),
                    amounts_added_raw: event.amounts_added_raw.iter().map(|v| v.to_string()).collect(),
                    swap_fee_amounts_raw: event.swap_fee_amounts_raw.iter().map(|v| v.to_string()).collect(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LiquidityRemoved event
            if let Some(event) = balancer::v3::vault::events::LiquidityRemoved::match_and_decode(log) {
                total_liquidity_removed += 1;
                let event = pb::log::Log::LiquidityRemoved(pb::LiquidityRemoved {
                    pool: event.pool.to_vec(),
                    liquidity_provider: event.liquidity_provider.to_vec(),
                    kind: event.kind.to_u64().min(u32::MAX as u64) as u32,
                    total_supply: event.total_supply.to_string(),
                    amounts_removed_raw: event.amounts_removed_raw.iter().map(|v| v.to_string()).collect(),
                    swap_fee_amounts_raw: event.swap_fee_amounts_raw.iter().map(|v| v.to_string()).collect(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PoolInitialized event
            if let Some(event) = balancer::v3::vault::events::PoolInitialized::match_and_decode(log) {
                total_pool_initialized += 1;
                let event = pb::log::Log::PoolInitialized(pb::PoolInitialized { pool: event.pool.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PoolRegistered event
            if let Some(event) = balancer::v3::vault::events::PoolRegistered::match_and_decode(log) {
                total_pool_registered += 1;
                let event = pb::log::Log::PoolRegistered(pb::PoolRegistered {
                    pool: event.pool.to_vec(),
                    factory: event.factory.to_vec(),
                    token_config: event
                        .token_config
                        .iter()
                        .map(|(token, token_type, rate_provider, pays_yield_fees)| pb::TokenConfig {
                            token: token.to_vec(),
                            token_type: token_type.to_u64().min(u32::MAX as u64) as u32,
                            rate_provider: rate_provider.to_vec(),
                            pays_yield_fees: *pays_yield_fees,
                        })
                        .collect(),
                    swap_fee_percentage: event.swap_fee_percentage.to_string(),
                    pause_window_end_time: event.pause_window_end_time.to_string(),
                    role_accounts: Some(pb::RoleAccounts {
                        pause_manager: event.role_accounts.0.to_vec(),
                        swap_fee_manager: event.role_accounts.1.to_vec(),
                        pool_creator: event.role_accounts.2.to_vec(),
                    }),
                    hooks_config: Some(pb::HooksConfig {
                        enable_hook_adjusted_amounts: event.hooks_config.0,
                        should_call_before_initialize: event.hooks_config.1,
                        should_call_after_initialize: event.hooks_config.2,
                        should_call_compute_dynamic_swap_fee: event.hooks_config.3,
                        should_call_before_swap: event.hooks_config.4,
                        should_call_after_swap: event.hooks_config.5,
                        should_call_before_add_liquidity: event.hooks_config.6,
                        should_call_after_add_liquidity: event.hooks_config.7,
                        should_call_before_remove_liquidity: event.hooks_config.8,
                        should_call_after_remove_liquidity: event.hooks_config.9,
                        hooks_address: event.hooks_config.10.to_vec(),
                    }),
                    liquidity_management: Some(pb::LiquidityManagement {
                        disable_unbalanced_liquidity: event.liquidity_management.0,
                        enable_add_liquidity_custom: event.liquidity_management.1,
                        enable_remove_liquidity_custom: event.liquidity_management.2,
                        enable_donation: event.liquidity_management.3,
                    }),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PoolPausedStateChanged event
            if let Some(event) = balancer::v3::vault::events::PoolPausedStateChanged::match_and_decode(log) {
                total_pool_paused += 1;
                let event = pb::log::Log::PoolPaused(pb::PoolPaused {
                    pool: event.pool.to_vec(),
                    paused: event.paused,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PoolRecoveryModeStateChanged event
            if let Some(event) = balancer::v3::vault::events::PoolRecoveryModeStateChanged::match_and_decode(log) {
                total_pool_recovery_mode += 1;
                let event = pb::log::Log::PoolRecoveryMode(pb::PoolRecoveryMode {
                    pool: event.pool.to_vec(),
                    enabled: event.recovery_mode,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // AggregateSwapFeePercentageChanged event
            if let Some(event) = balancer::v3::vault::events::AggregateSwapFeePercentageChanged::match_and_decode(log) {
                total_aggregate_swap_fee_percentage += 1;
                let event = pb::log::Log::AggregateSwapFeePercentage(pb::AggregateSwapFeePercentage {
                    pool: event.pool.to_vec(),
                    aggregate_swap_fee_percentage: event.aggregate_swap_fee_percentage.to_string(),
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
    substreams::log::info!("===== V2 WeightedPool Events =====");
    substreams::log::info!("Total SwapFeePercentage events: {}", total_swap_fee_percentage);
    substreams::log::info!("Total Paused events: {}", total_paused);
    substreams::log::info!("Total RecoveryMode events: {}", total_recovery_mode);
    substreams::log::info!("Total ProtocolFeePercentageCacheUpdated events: {}", total_protocol_fee_percentage);
    substreams::log::info!("===== V3 StablePool Events =====");
    substreams::log::info!("Total AmpUpdateStarted events: {}", total_amp_update_started);
    substreams::log::info!("Total AmpUpdateStopped events: {}", total_amp_update_stopped);
    substreams::log::info!("===== V3 Vault Events =====");
    substreams::log::info!("Total VaultSwap events: {}", total_vault_swaps);
    substreams::log::info!("Total LiquidityAdded events: {}", total_liquidity_added);
    substreams::log::info!("Total LiquidityRemoved events: {}", total_liquidity_removed);
    substreams::log::info!("Total PoolInitialized events: {}", total_pool_initialized);
    substreams::log::info!("Total PoolRegistered events: {}", total_pool_registered);
    substreams::log::info!("Total PoolPaused events: {}", total_pool_paused);
    substreams::log::info!("Total PoolRecoveryMode events: {}", total_pool_recovery_mode);
    substreams::log::info!("Total AggregateSwapFeePercentageChanged events: {}", total_aggregate_swap_fee_percentage);
    Ok(events)
}
