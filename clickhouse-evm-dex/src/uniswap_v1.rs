use common::{bytes_to_string, Encoding};
use proto::pb::uniswap::v1::{self as uniswap, StorePool};
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
                Some(uniswap::log::Log::TokenPurchase(event)) => {
                    process_token_purchase(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::EthPurchase(event)) => {
                    process_eth_purchase(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::AddLiquidity(event)) => {
                    process_add_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::RemoveLiquidity(event)) => {
                    process_remove_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(uniswap::log::Log::NewExchange(event)) => {
                    process_new_exchange(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: StorePool, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("token", bytes_to_string(&value.currency0, encoding));
}

fn process_token_purchase(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::TokenPurchase,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_token_purchase", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("buyer", bytes_to_string(&event.buyer, encoding));
        row.set("eth_sold", &event.eth_sold);
        row.set("tokens_bought", &event.tokens_bought);
    }
}

fn process_eth_purchase(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::EthPurchase,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_eth_purchase", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("buyer", bytes_to_string(&event.buyer, encoding));
        row.set("tokens_sold", &event.tokens_sold);
        row.set("eth_bought", &event.eth_bought);
    }
}

fn process_add_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::AddLiquidity,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_add_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("eth_amount", &event.eth_amount);
        row.set("token_amount", &event.token_amount);
    }
}

fn process_remove_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::RemoveLiquidity,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("uniswap_v1_remove_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
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

    row.set("factory", bytes_to_string(&log.address, encoding));
    row.set("exchange", bytes_to_string(&event.exchange, encoding));
    row.set("token", bytes_to_string(&event.token, encoding));
}
