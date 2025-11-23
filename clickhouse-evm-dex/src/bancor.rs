use core::panic;

use common::{bytes_to_string, Encoding};
use proto::pb::bancor::v1::{self as bancor, NewConverter};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &bancor::Events, store: &StoreGetProto<NewConverter>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(bancor::log::Log::Conversion(event)) => {
                    process_conversion(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::LiquidityAdded(event)) => {
                    process_liquidity_added(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::LiquidityRemoved(event)) => {
                    process_liquidity_removed(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::TokenRateUpdate(event)) => {
                    process_token_rate_update(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::Activation(event)) => {
                    process_activation(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::NewConverter(event)) => {
                    process_new_converter(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

fn set_pool(encoding: &Encoding, value: Option<NewConverter>, row: &mut substreams_database_change::tables::Row) {
    if let Some(value) = value {
        row.set("factory", bytes_to_string(&value.factory, encoding));
        row.set("converter_type", value.converter_type);
    } else {
        row.set("factory", "");
        row.set("converter_type", 0u32);
    }
}

fn process_conversion(
    encoding: &Encoding,
    store: &StoreGetProto<NewConverter>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::Conversion,
) {
    let pool = get_store_by_address(store, &log.address);
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_conversion", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pool(encoding, pool, row);

    row.set("source_token", bytes_to_string(&event.source_token, encoding));
    row.set("target_token", bytes_to_string(&event.target_token, encoding));
    row.set("trader", bytes_to_string(&event.trader, encoding));
    row.set("source_amount", &event.source_amount);
    row.set("target_amount", &event.target_amount);
    row.set("conversion_fee", &event.conversion_fee);
}

fn process_liquidity_added(
    encoding: &Encoding,
    store: &StoreGetProto<NewConverter>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::LiquidityAdded,
) {
    let pool = get_store_by_address(store, &log.address);
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_liquidity_added", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pool(encoding, pool, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("reserve_token", bytes_to_string(&event.reserve_token, encoding));
    row.set("amount", &event.amount);
    row.set("new_balance", &event.new_balance);
    row.set("new_supply", &event.new_supply);
}

fn process_liquidity_removed(
    encoding: &Encoding,
    store: &StoreGetProto<NewConverter>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::LiquidityRemoved,
) {
    let pool = get_store_by_address(store, &log.address);
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_liquidity_removed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pool(encoding, pool, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("reserve_token", bytes_to_string(&event.reserve_token, encoding));
    row.set("amount", &event.amount);
    row.set("new_balance", &event.new_balance);
    row.set("new_supply", &event.new_supply);
}

fn process_token_rate_update(
    encoding: &Encoding,
    store: &StoreGetProto<NewConverter>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::TokenRateUpdate,
) {
    let pool = get_store_by_address(store, &log.address);
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_token_rate_update", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pool(encoding, pool, row);

    row.set("token1", bytes_to_string(&event.token1, encoding));
    row.set("token2", bytes_to_string(&event.token2, encoding));
    row.set("rate_n", &event.rate_n);
    row.set("rate_d", &event.rate_d);
}

fn process_activation(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::Activation,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_activation", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("activated", event.activated);
    row.set("anchor", bytes_to_string(&event.anchor, encoding));
    row.set("converter_type", event.converter_type);
}

fn process_new_converter(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::NewConverter,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_new_converter", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("converter_type", event.converter_type);
    row.set("converter", bytes_to_string(&event.converter, encoding));
    row.set("owner", bytes_to_string(&event.owner, encoding));
}
