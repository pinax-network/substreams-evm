use common::clickhouse::{log_key, set_clock, set_template_call, set_template_log, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::uniswap::v1::{self as uniswap};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;
use substreams_ethereum::NULL_ADDRESS;

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
                Some(uniswap::log::Log::TokenPurchase(event)) => {
                    process_token_purchase(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::EthPurchase(event)) => {
                    process_eth_purchase(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::AddLiquidity(event)) => {
                    process_add_liquidity(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::RemoveLiquidity(event)) => {
                    process_remove_liquidity(encoding, pools, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::NewExchange(event)) => {
                    process_new_exchange(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: &PoolMetadata, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("token", bytes_to_string(token(&value, 0), encoding));
    row.set("eth", bytes_to_string(&NULL_ADDRESS, encoding));
}

fn process_token_purchase(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::TokenPurchase,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_token_purchase", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("buyer", bytes_to_string(&event.buyer, encoding));
        row.set("eth_sold", &event.eth_sold);
        row.set("tokens_bought", &event.tokens_bought);
    }
}

fn process_eth_purchase(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::EthPurchase,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_eth_purchase", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("buyer", bytes_to_string(&event.buyer, encoding));
        row.set("tokens_sold", &event.tokens_sold);
        row.set("eth_bought", &event.eth_bought);
    }
}

fn process_add_liquidity(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::AddLiquidity,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_add_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("eth_amount", &event.eth_amount);
        row.set("token_amount", &event.token_amount);
    }
}

fn process_remove_liquidity(
    encoding: &Encoding,
    pools: &PoolMetadataMap,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::RemoveLiquidity,
) {
    if let Some(pool) = get_pool_by_address(pools, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_remove_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("eth_amount", &event.eth_amount);
        row.set("token_amount", &event.token_amount);
    }
}

fn process_new_exchange(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::NewExchange,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v1_new_exchange", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("exchange", bytes_to_string(&event.exchange, encoding));
    row.set("token", bytes_to_string(&event.token, encoding));
    row.set("eth", bytes_to_string(&NULL_ADDRESS, encoding));
}
