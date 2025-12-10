use common::{bytes_to_string, Encoding};
use proto::pb::uniswap::v4::{self as uniswap, StorePool};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &uniswap::Events, store: &StoreGetProto<StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(uniswap::log::Log::Swap(event)) => {
                    process_swap(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Initialize(event)) => {
                    process_initialize(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::ModifyLiquidity(event)) => {
                    process_modify_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Donate(event)) => {
                    process_donate(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::ProtocolFeeControllerUpdated(event)) => {
                    process_protocol_fee_controller_updated(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::ProtocolFeeUpdated(event)) => {
                    process_protocol_fee_updated(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: StorePool, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("currency0", bytes_to_string(&value.currency0, encoding));
    row.set("currency1", bytes_to_string(&value.currency1, encoding));
}

fn process_swap(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Swap,
) {
    if let Some(pool) = get_store_by_address(store, &event.id) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v4_swap", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("id", bytes_to_string(&event.id, encoding));
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
        row.set("sqrt_price_x96", &event.sqrt_price_x96);
        row.set("liquidity", &event.liquidity);
        row.set("tick", event.tick);
        row.set("fee", &event.fee);
    }
}

fn process_modify_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::ModifyLiquidity,
) {
    if let Some(pool) = get_store_by_address(store, &event.id) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v4_modify_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("id", bytes_to_string(&event.id, encoding));
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("tick_lower", event.tick_lower);
        row.set("tick_upper", event.tick_upper);
        row.set("liquidity_delta", &event.liquidity_delta);
        row.set("salt", bytes_to_string(&event.salt, encoding));
    }
}

fn process_donate(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Donate,
) {
    if let Some(pool) = get_store_by_address(store, &event.id) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v4_donate", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("id", bytes_to_string(&event.id, encoding));
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_protocol_fee_updated(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::ProtocolFeeUpdated,
) {
    if let Some(pool) = get_store_by_address(store, &event.id) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v4_protocol_fee_updated", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("id", bytes_to_string(&event.id, encoding));
        row.set("protocol_fee", event.protocol_fee);
    }
}

fn process_protocol_fee_controller_updated(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::ProtocolFeeControllerUpdated,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v4_protocol_fee_controller_updated", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("protocol_fee_controller", bytes_to_string(&event.protocol_fee_controller, encoding));
}

fn process_initialize(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Initialize,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v4_initialize", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("id", bytes_to_string(&event.id, encoding));
    row.set("currency0", bytes_to_string(&event.currency0, encoding));
    row.set("currency1", bytes_to_string(&event.currency1, encoding));
    row.set("fee", event.fee);
    row.set("tick_spacing", event.tick_spacing);
    row.set("sqrt_price_x96", &event.sqrt_price_x96);
    row.set("tick", event.tick);
    row.set("hooks", bytes_to_string(&event.hooks, encoding));
}
