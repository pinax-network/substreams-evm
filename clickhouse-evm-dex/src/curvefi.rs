use common::{bytes_to_string, Encoding};
use proto::pb::curvefi::v1::{self as curvefi, StorePool};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &curvefi::Events, store: &StoreGetProto<StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(curvefi::log::Log::TokenExchange(event)) => {
                    process_token_exchange(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::AddLiquidity(event)) => {
                    process_add_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidity(event)) => {
                    process_remove_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidityOne(event)) => {
                    process_remove_liquidity_one(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidityImbalance(event)) => {
                    process_remove_liquidity_imbalance(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::PlainPoolDeployed(event)) => {
                    process_plain_pool_deployed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::MetaPoolDeployed(event)) => {
                    process_meta_pool_deployed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: StorePool, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("coins", value.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
}

fn parse_coin(encoding: &Encoding, id: String, coins: &Vec<Vec<u8>>) -> Option<String> {
    if let Ok(index) = id.parse::<usize>() {
        return coins.get(index).map(|c| bytes_to_string(c, encoding));
    }
    None
}

fn process_token_exchange(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::TokenExchange,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        // proceed only if pool details are available
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_token_exchange", key);

        // compute token indices from `coins` of the pool
        let sold_token = parse_coin(encoding, event.sold_id.clone(), &pool.coins);
        let bought_token = parse_coin(encoding, event.bought_id.clone(), &pool.coins);
        if sold_token.is_none() || bought_token.is_none() {
            return;
        }

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("buyer", bytes_to_string(&event.buyer, encoding));
        row.set("sold_id", &event.sold_id);
        row.set("sold_amount", &event.tokens_sold);
        row.set("sold_token", sold_token.unwrap());
        row.set("bought_id", &event.bought_id);
        row.set("bought_amount", &event.tokens_bought);
        row.set("bought_token", bought_token.unwrap());
    }
}

fn process_add_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::AddLiquidity,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_add_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("fees", event.fees.join(","));
        row.set("invariant", &event.invariant);
        row.set("token_supply", &event.token_supply);
    }
}

fn process_remove_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidity,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_remove_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("fees", event.fees.join(","));
        row.set("token_supply", &event.token_supply);
    }
}

fn process_remove_liquidity_one(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityOne,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_remove_liquidity_one", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amount", &event.token_amount);
        row.set("coin_amount", &event.coin_amount);
    }
}

fn process_remove_liquidity_imbalance(
    encoding: &Encoding,
    store: &StoreGetProto<StorePool>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityImbalance,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_remove_liquidity_imbalance", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("fees", event.fees.join(","));
        row.set("invariant", &event.invariant);
        row.set("token_supply", &event.token_supply);
    }
}

fn process_plain_pool_deployed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::PlainPoolDeployed,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_plain_pool_deployed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("coins", event.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
    row.set("a", &event.a);
    row.set("fee", &event.fee);
    row.set("deployer", bytes_to_string(&event.deployer, encoding));
}

fn process_meta_pool_deployed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::MetaPoolDeployed,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_meta_pool_deployed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("coin", bytes_to_string(&event.coin, encoding));
    row.set("base_pool", bytes_to_string(&event.base_pool, encoding));
    row.set("a", &event.a);
    row.set("fee", &event.fee);
    row.set("deployer", bytes_to_string(&event.deployer, encoding));
}
