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
