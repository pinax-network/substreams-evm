use common::{bytes_to_string, Encoding};
use proto::pb::uniswap::v2::{self as uniswap, PairCreated};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    foundational_stores::get_pair_created,
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &uniswap::Events, store: &StoreGetProto<PairCreated>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(uniswap::log::Log::Swap(event)) => {
                    process_swap(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Sync(event)) => {
                    process_sync(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Mint(event)) => {
                    process_mint(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Burn(event)) => {
                    process_burn(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::PairCreated(event)) => {
                    process_pair_created(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pair_created(encoding: &Encoding, value: Option<PairCreated>, row: &mut substreams_database_change::tables::Row) {
    if let Some(value) = value {
        row.set("factory", bytes_to_string(&value.factory, encoding));
        row.set("token0", bytes_to_string(&value.token0, encoding));
        row.set("token1", bytes_to_string(&value.token1, encoding));
    } else {
        row.set("factory", "");
        row.set("token0", "");
        row.set("token1", "");
    }
}

fn process_swap(
    encoding: &Encoding,
    store: &StoreGetProto<PairCreated>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Swap,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v2_swap", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pair_created(encoding, get_pair_created(store, &log.address), row);

    row.set("sender", bytes_to_string(&event.sender, encoding));
    row.set("to", bytes_to_string(&event.to, encoding));
    row.set("amount0_in", &event.amount0_in);
    row.set("amount1_in", &event.amount1_in);
    row.set("amount0_out", &event.amount0_out);
    row.set("amount1_out", &event.amount1_out);
}

fn process_sync(
    encoding: &Encoding,
    store: &StoreGetProto<PairCreated>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Sync,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v2_sync", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pair_created(encoding, get_pair_created(store, &log.address), row);

    row.set("reserve0", &event.reserve0);
    row.set("reserve1", &event.reserve1);
}

fn process_mint(
    encoding: &Encoding,
    store: &StoreGetProto<PairCreated>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Mint,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v2_mint", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pair_created(encoding, get_pair_created(store, &log.address), row);

    row.set("sender", bytes_to_string(&event.sender, encoding));
    row.set("amount0", &event.amount0);
    row.set("amount1", &event.amount1);
}

fn process_burn(
    encoding: &Encoding,
    store: &StoreGetProto<PairCreated>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Burn,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v2_burn", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_pair_created(encoding, get_pair_created(store, &log.address), row);

    row.set("sender", bytes_to_string(&event.sender, encoding));
    row.set("amount0", &event.amount0);
    row.set("amount1", &event.amount1);
    row.set("to", bytes_to_string(&event.to, encoding));
}

fn process_pair_created(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::PairCreated,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v2_pair_created", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("token0", bytes_to_string(&event.token0, encoding));
    row.set("token1", bytes_to_string(&event.token1, encoding));
    row.set("pair", bytes_to_string(&event.pair, encoding));
    row.set("extra_data", &event.extra_data);
}
