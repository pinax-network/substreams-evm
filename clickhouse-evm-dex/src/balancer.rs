use common::{bytes_to_string, Encoding};
use proto::pb::balancer::v1::{self as balancer, StorePool};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &balancer::Events, store: &StoreGetProto<StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(balancer::log::Log::VaultSwap(event)) => {
                    process_vault_swap(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::LiquidityAdded(event)) => {
                    process_liquidity_added(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::LiquidityRemoved(event)) => {
                    process_liquidity_removed(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::PoolRegistered(event)) => {
                    process_pool_registered(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::SwapFeePercentage(event)) => {
                    process_swap_fee_percentage(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::ProtocolFeePercentage(event)) => {
                    process_protocol_fee_percentage(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::AggregateSwapFeePercentage(event)) => {
                    process_aggregate_swap_fee_percentage(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: Option<StorePool>, row: &mut substreams_database_change::tables::Row) {
    if let Some(value) = value {
        row.set("factory", bytes_to_string(&value.factory, encoding));
    } else {
        row.set("factory", "");
    }
}

fn process_vault_swap(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::VaultSwap,
) {
    if let Some(pool) = get_store_by_address(store, &event.pool) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("balancer_vault_swap", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, Some(pool), row);

        row.set("pool", bytes_to_string(&event.pool, encoding));
        row.set("token_in", bytes_to_string(&event.token_in, encoding));
        row.set("token_out", bytes_to_string(&event.token_out, encoding));
        row.set("amount_in", &event.amount_in);
        row.set("amount_out", &event.amount_out);
        row.set("swap_fee_percentage", &event.swap_fee_percentage);
        row.set("swap_fee_amount", &event.swap_fee_amount);
    }
}

fn process_liquidity_added(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::LiquidityAdded,
) {
    if let Some(pool) = get_store_by_address(store, &event.pool) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("balancer_liquidity_added", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, Some(pool), row);

        row.set("pool", bytes_to_string(&event.pool, encoding));
        row.set("liquidity_provider", bytes_to_string(&event.liquidity_provider, encoding));
        row.set("kind", event.kind);
        row.set("total_supply", &event.total_supply);
        row.set("amounts_added_raw", event.amounts_added_raw.join(","));
        row.set("swap_fee_amounts_raw", event.swap_fee_amounts_raw.join(","));
    }
}

fn process_liquidity_removed(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::LiquidityRemoved,
) {
    if let Some(pool) = get_store_by_address(store, &event.pool) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("balancer_liquidity_removed", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, Some(pool), row);

        row.set("pool", bytes_to_string(&event.pool, encoding));
        row.set("liquidity_provider", bytes_to_string(&event.liquidity_provider, encoding));
        row.set("kind", event.kind);
        row.set("total_supply", &event.total_supply);
        row.set("amounts_removed_raw", event.amounts_removed_raw.join(","));
        row.set("swap_fee_amounts_raw", event.swap_fee_amounts_raw.join(","));
    }
}

fn process_pool_registered(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::PoolRegistered,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("balancer_pool_registered", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("factory", bytes_to_string(&event.factory, encoding));

    // Serialize token_config as JSON string
    row.set("token_config", serialize_token_config(encoding, &event.token_config));

    row.set("swap_fee_percentage", &event.swap_fee_percentage);
    row.set("pause_window_end_time", &event.pause_window_end_time);

    // Set role accounts
    if let Some(role_accounts) = &event.role_accounts {
        row.set("pause_manager", bytes_to_string(&role_accounts.pause_manager, encoding));
        row.set("swap_fee_manager", bytes_to_string(&role_accounts.swap_fee_manager, encoding));
        row.set("pool_creator", bytes_to_string(&role_accounts.pool_creator, encoding));
    } else {
        row.set("pause_manager", "");
        row.set("swap_fee_manager", "");
        row.set("pool_creator", "");
    }

    // Set hooks config
    if let Some(hooks_config) = &event.hooks_config {
        row.set("enable_hook_adjusted_amounts", hooks_config.enable_hook_adjusted_amounts);
        row.set("should_call_before_initialize", hooks_config.should_call_before_initialize);
        row.set("should_call_after_initialize", hooks_config.should_call_after_initialize);
        row.set("should_call_compute_dynamic_swap_fee", hooks_config.should_call_compute_dynamic_swap_fee);
        row.set("should_call_before_swap", hooks_config.should_call_before_swap);
        row.set("should_call_after_swap", hooks_config.should_call_after_swap);
        row.set("should_call_before_add_liquidity", hooks_config.should_call_before_add_liquidity);
        row.set("should_call_after_add_liquidity", hooks_config.should_call_after_add_liquidity);
        row.set("should_call_before_remove_liquidity", hooks_config.should_call_before_remove_liquidity);
        row.set("should_call_after_remove_liquidity", hooks_config.should_call_after_remove_liquidity);
        row.set("hooks_address", bytes_to_string(&hooks_config.hooks_address, encoding));
    } else {
        row.set("enable_hook_adjusted_amounts", false);
        row.set("should_call_before_initialize", false);
        row.set("should_call_after_initialize", false);
        row.set("should_call_compute_dynamic_swap_fee", false);
        row.set("should_call_before_swap", false);
        row.set("should_call_after_swap", false);
        row.set("should_call_before_add_liquidity", false);
        row.set("should_call_after_add_liquidity", false);
        row.set("should_call_before_remove_liquidity", false);
        row.set("should_call_after_remove_liquidity", false);
        row.set("hooks_address", "");
    }

    // Set liquidity management
    if let Some(liquidity_management) = &event.liquidity_management {
        row.set("disable_unbalanced_liquidity", liquidity_management.disable_unbalanced_liquidity);
        row.set("enable_add_liquidity_custom", liquidity_management.enable_add_liquidity_custom);
        row.set("enable_remove_liquidity_custom", liquidity_management.enable_remove_liquidity_custom);
        row.set("enable_donation", liquidity_management.enable_donation);
    } else {
        row.set("disable_unbalanced_liquidity", false);
        row.set("enable_add_liquidity_custom", false);
        row.set("enable_remove_liquidity_custom", false);
        row.set("enable_donation", false);
    }
}

fn serialize_token_config(encoding: &Encoding, token_configs: &[balancer::TokenConfig]) -> String {
    let configs: Vec<String> = token_configs
        .iter()
        .map(|tc| {
            format!(
                r#"{{"token":"{}","token_type":{},"rate_provider":"{}","paysYieldFees":{}}}"#,
                bytes_to_string(&tc.token, encoding),
                tc.token_type,
                bytes_to_string(&tc.rate_provider, encoding),
                tc.pays_yield_fees
            )
        })
        .collect();
    format!("[{}]", configs.join(","))
}

fn process_swap_fee_percentage(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::SwapFeePercentage,
) {
    let pool = get_store_by_address(store, &log.address);
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("balancer_swap_fee_percentage", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pool(encoding, pool, row);

    row.set("swap_fee_percentage", &event.swap_fee_percentage);
}

fn process_protocol_fee_percentage(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::ProtocolFeePercentage,
) {
    let pool = get_store_by_address(store, &log.address);
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("balancer_protocol_fee_percentage", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pool(encoding, pool, row);

    row.set("fee_type", &event.fee_type);
    row.set("protocol_fee_percentage", &event.protocol_fee_percentage);
}

fn process_aggregate_swap_fee_percentage(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::AggregateSwapFeePercentage,
) {
    if let Some(pool_data) = get_store_by_address(store, &event.pool) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("balancer_aggregate_swap_fee_percentage", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, Some(pool_data), row);

        row.set("pool", bytes_to_string(&event.pool, encoding));
        row.set("aggregate_swap_fee_percentage", &event.aggregate_swap_fee_percentage);
    }
}
