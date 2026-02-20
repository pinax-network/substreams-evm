use common::{bytes_to_string, Encoding};
use proto::pb::aerodrome::v1::{self as aerodrome, StorePool};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &aerodrome::Events, store: &StoreGetProto<StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(aerodrome::log::Log::Swap(event)) => {
                    process_swap(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(aerodrome::log::Log::Sync(event)) => {
                    process_sync(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(aerodrome::log::Log::Mint(event)) => {
                    process_mint(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(aerodrome::log::Log::Burn(event)) => {
                    process_burn(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(aerodrome::log::Log::Fees(event)) => {
                    process_fees(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(aerodrome::log::Log::Claim(event)) => {
                    process_claim(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(aerodrome::log::Log::PoolCreated(event)) => {
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
    row.set("stable", value.stable);
}

fn process_swap(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &aerodrome::Transaction, log: &aerodrome::Log, tx_index: usize, log_index: usize, event: &aerodrome::Swap) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("aerodrome_swap", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("to", bytes_to_string(&event.to, encoding));
        row.set("amount0_in", &event.amount0_in);
        row.set("amount1_in", &event.amount1_in);
        row.set("amount0_out", &event.amount0_out);
        row.set("amount1_out", &event.amount1_out);
    }
}

fn process_sync(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &aerodrome::Transaction, log: &aerodrome::Log, tx_index: usize, log_index: usize, event: &aerodrome::Sync) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("aerodrome_sync", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("reserve0", &event.reserve0);
        row.set("reserve1", &event.reserve1);
    }
}

fn process_mint(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &aerodrome::Transaction, log: &aerodrome::Log, tx_index: usize, log_index: usize, event: &aerodrome::Mint) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("aerodrome_mint", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_burn(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &aerodrome::Transaction, log: &aerodrome::Log, tx_index: usize, log_index: usize, event: &aerodrome::Burn) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("aerodrome_burn", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("to", bytes_to_string(&event.to, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_fees(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &aerodrome::Transaction, log: &aerodrome::Log, tx_index: usize, log_index: usize, event: &aerodrome::Fees) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("aerodrome_fees", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_claim(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &aerodrome::Transaction, log: &aerodrome::Log, tx_index: usize, log_index: usize, event: &aerodrome::Claim) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("aerodrome_claim", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("recipient", bytes_to_string(&event.recipient, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_pool_created(encoding: &Encoding, tables: &mut Tables, clock: &Clock, tx: &aerodrome::Transaction, log: &aerodrome::Log, tx_index: usize, log_index: usize, event: &aerodrome::PoolCreated) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("aerodrome_pool_created", key);
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    row.set("token0", bytes_to_string(&event.token0, encoding));
    row.set("token1", bytes_to_string(&event.token1, encoding));
    row.set("stable", event.stable);
    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("extra_data", &event.extra_data);
}
