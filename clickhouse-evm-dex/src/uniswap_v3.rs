use common::{bytes_to_string, Encoding};
use proto::pb::uniswap::v3::{self as uniswap, StorePool};
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
                    process_initialize(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Mint(event)) => {
                    process_mint(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Collect(event)) => {
                    process_collect(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Burn(event)) => {
                    process_burn(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Flash(event)) => {
                    process_flash(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::IncreaseObservationCardinalityNext(event)) => {
                    process_increase_observation(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::SetFeeProtocol(event)) => {
                    process_set_fee_protocol(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::CollectProtocol(event)) => {
                    process_collect_protocol(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::PoolCreated(event)) => {
                    process_pool_created(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::OwnerChanged(event)) => {
                    process_owner_changed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::FeeAmountEnabled(event)) => {
                    process_fee_amount_enabled(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool_created(encoding: &Encoding, value: StorePool, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("token0", bytes_to_string(&value.currency0, encoding));
    row.set("token1", bytes_to_string(&value.currency1, encoding));
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
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_swap", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("recipient", bytes_to_string(&event.recipient, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
        row.set("sqrt_price_x96", &event.sqrt_price_x96);
        row.set("liquidity", &event.liquidity);
        row.set("tick", event.tick);
    }
}

fn process_initialize(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Initialize,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_initialize", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("sqrt_price_x96", &event.sqrt_price_x96);
        row.set("tick", event.tick);
    }
}

fn process_mint(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Mint,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_mint", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("owner", bytes_to_string(&event.owner, encoding));
        row.set("tick_lower", event.tick_lower);
        row.set("tick_upper", event.tick_upper);
        row.set("amount", &event.amount);
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_collect(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Collect,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_collect", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("owner", bytes_to_string(&event.owner, encoding));
        row.set("recipient", bytes_to_string(&event.recipient, encoding));
        row.set("tick_lower", event.tick_lower);
        row.set("tick_upper", event.tick_upper);
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_burn(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Burn,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_burn", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("owner", bytes_to_string(&event.owner, encoding));
        row.set("tick_lower", event.tick_lower);
        row.set("tick_upper", event.tick_upper);
        row.set("amount", &event.amount);
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_flash(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Flash,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_flash", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("recipient", bytes_to_string(&event.recipient, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
        row.set("paid0", &event.paid0);
        row.set("paid1", &event.paid1);
    }
}

fn process_increase_observation(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::IncreaseObservationCardinalityNext,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_increase_observation_cardinality_next", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("observation_cardinality_next_old", event.observation_cardinality_next_old);
        row.set("observation_cardinality_next_new", event.observation_cardinality_next_new);
    }
}

fn process_set_fee_protocol(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::SetFeeProtocol,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_set_fee_protocol", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("fee_protocol0_old", event.fee_protocol0_old);
        row.set("fee_protocol1_old", event.fee_protocol1_old);
        row.set("fee_protocol0_new", event.fee_protocol0_new);
        row.set("fee_protocol1_new", event.fee_protocol1_new);
    }
}

fn process_collect_protocol(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::CollectProtocol,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v3_collect_protocol", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool_created(encoding, pool, row);

        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("recipient", bytes_to_string(&event.recipient, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_pool_created(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::PoolCreated,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v3_pool_created", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("token0", bytes_to_string(&event.token0, encoding));
    row.set("token1", bytes_to_string(&event.token1, encoding));
    row.set("fee", event.fee);
    row.set("tick_spacing", event.tick_spacing);
    row.set("pool", bytes_to_string(&event.pool, encoding));
}

fn process_owner_changed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::OwnerChanged,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v3_owner_changed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("old_owner", bytes_to_string(&event.old_owner, encoding));
    row.set("new_owner", bytes_to_string(&event.new_owner, encoding));
}

fn process_fee_amount_enabled(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::FeeAmountEnabled,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v3_fee_amount_enabled", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("fee", event.fee);
    row.set("tick_spacing", event.tick_spacing);
}
