use common::clickhouse::{log_key, set_clock, set_template_call, set_template_log, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::uniswap::v2::{self as uniswap};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::store::{collect_address, get_pool_by_address, token, PoolMetadata, PoolMetadataMap};

pub fn collect_pool_addresses(events: &uniswap::Events, addresses: &mut std::collections::HashSet<Vec<u8>>) {
    for trx in &events.transactions {
        for log in &trx.logs {
            collect_address(addresses, &log.address);
        }
    }
}

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &uniswap::Events, pools: &PoolMetadataMap) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(uniswap::log::Log::Swap(event)) => {
                    process_swap(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Sync(event)) => {
                    process_sync(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Mint(event)) => {
                    process_mint(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::Burn(event)) => {
                    process_burn(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::PairCreated(event)) => {
                    process_pair_created(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: &PoolMetadata, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("token0", bytes_to_string(token(&value, 0), encoding));
    row.set("token1", bytes_to_string(token(&value, 1), encoding));
}

fn process_swap(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Swap,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v2_swap", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("to", bytes_to_string(&event.to, encoding));
        row.set("amount0_in", &event.amount0_in);
        row.set("amount1_in", &event.amount1_in);
        row.set("amount0_out", &event.amount0_out);
        row.set("amount1_out", &event.amount1_out);
    }
}

fn process_sync(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Sync,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v2_sync", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("reserve0", &event.reserve0);
        row.set("reserve1", &event.reserve1);
    }
}

fn process_mint(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Mint,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v2_mint", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
    }
}

fn process_burn(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::Burn,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v2_burn", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("amount0", &event.amount0);
        row.set("amount1", &event.amount1);
        row.set("to", bytes_to_string(&event.to, encoding));
    }
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
    set_template_call(encoding, log, row);

    row.set("token0", bytes_to_string(&event.token0, encoding));
    row.set("token1", bytes_to_string(&event.token1, encoding));
    row.set("pair", bytes_to_string(&event.pair, encoding));
    row.set("extra_data", &event.extra_data);
}
