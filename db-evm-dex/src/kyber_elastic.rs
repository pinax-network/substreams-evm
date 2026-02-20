use common::{bytes_to_string, Encoding};
use proto::pb::kyber_elastic::v1::{self as kyber, StorePool};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &kyber::Events, store: &StoreGetProto<StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(kyber::log::Log::Swap(event)) => {
                    process_swap(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(kyber::log::Log::Mint(event)) => {
                    process_mint(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(kyber::log::Log::Burn(event)) => {
                    process_burn(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(kyber::log::Log::PoolCreated(event)) => {
                    process_pool_created(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: StorePool, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("token0", bytes_to_string(&value.currency0, encoding));
    row.set("token1", bytes_to_string(&value.currency1, encoding));
    row.set("swap_fee_units", value.swap_fee_units);
    row.set("tick_distance", value.tick_distance);
}

fn process_swap(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &kyber::Transaction, log: &kyber::Log, tx_index: usize, log_index: usize, event: &kyber::Swap) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("kyber_elastic_swap", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("recipient", bytes_to_string(&event.recipient, encoding));
        row.set("delta_qty0", &event.delta_qty0);
        row.set("delta_qty1", &event.delta_qty1);
        row.set("sqrt_p", &event.sqrt_p);
        row.set("liquidity", &event.liquidity);
        row.set("current_tick", event.current_tick);
    }
}

fn process_mint(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &kyber::Transaction, log: &kyber::Log, tx_index: usize, log_index: usize, event: &kyber::Mint) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("kyber_elastic_mint", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("owner", bytes_to_string(&event.owner, encoding));
        row.set("tick_lower", event.tick_lower);
        row.set("tick_upper", event.tick_upper);
        row.set("qty", &event.qty);
        row.set("qty0", &event.qty0);
        row.set("qty1", &event.qty1);
    }
}

fn process_burn(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &kyber::Transaction, log: &kyber::Log, tx_index: usize, log_index: usize, event: &kyber::Burn) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("kyber_elastic_burn", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("owner", bytes_to_string(&event.owner, encoding));
        row.set("tick_lower", event.tick_lower);
        row.set("tick_upper", event.tick_upper);
        row.set("qty", &event.qty);
        row.set("qty0", &event.qty0);
        row.set("qty1", &event.qty1);
    }
}

fn process_pool_created(encoding: &Encoding, tables: &mut Tables, clock: &Clock, tx: &kyber::Transaction, log: &kyber::Log, tx_index: usize, log_index: usize, event: &kyber::PoolCreated) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("kyber_elastic_pool_created", key);
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    row.set("token0", bytes_to_string(&event.token0, encoding));
    row.set("token1", bytes_to_string(&event.token1, encoding));
    row.set("swap_fee_units", event.swap_fee_units);
    row.set("tick_distance", event.tick_distance);
    row.set("pool", bytes_to_string(&event.pool, encoding));
}
