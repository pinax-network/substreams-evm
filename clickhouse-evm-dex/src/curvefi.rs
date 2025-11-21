use common::{bytes_to_string, Encoding};
use proto::pb::curvefi::v1::{self as curvefi, PlainPoolDeployed};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    foundational_stores::get_pair_created,
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &curvefi::Events, store: &StoreGetProto<PlainPoolDeployed>) {
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

pub fn set_plain_pool_deployed(encoding: &Encoding, value: Option<PlainPoolDeployed>, row: &mut substreams_database_change::tables::Row) {
    if let Some(value) = value {
        row.set("factory", bytes_to_string(&value.factory, encoding));
        row.set("coins", value.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
    } else {
        row.set("factory", "");
        row.set("coins", "");
    }
}

fn process_token_exchange(
    encoding: &Encoding,
    store: &StoreGetProto<PlainPoolDeployed>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::TokenExchange,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_token_exchange", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_plain_pool_deployed(encoding, get_pair_created(store, &log.address), row);

    row.set("buyer", bytes_to_string(&event.buyer, encoding));
    row.set("sold_id", &event.sold_id);
    row.set("tokens_sold", &event.tokens_sold);
    row.set("bought_id", &event.bought_id);
    row.set("tokens_bought", &event.tokens_bought);
}

fn process_add_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<PlainPoolDeployed>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::AddLiquidity,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_add_liquidity", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_plain_pool_deployed(encoding, get_pair_created(store, &log.address), row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amounts", event.token_amounts.join(","));
    row.set("fees", event.fees.join(","));
    row.set("invariant", &event.invariant);
    row.set("token_supply", &event.token_supply);
}

fn process_remove_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<PlainPoolDeployed>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidity,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_remove_liquidity", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_plain_pool_deployed(encoding, get_pair_created(store, &log.address), row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amounts", event.token_amounts.join(","));
    row.set("fees", event.fees.join(","));
    row.set("token_supply", &event.token_supply);
}

fn process_remove_liquidity_one(
    encoding: &Encoding,
    store: &StoreGetProto<PlainPoolDeployed>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityOne,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_remove_liquidity_one", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_plain_pool_deployed(encoding, get_pair_created(store, &log.address), row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amount", &event.token_amount);
    row.set("coin_amount", &event.coin_amount);
}

fn process_remove_liquidity_imbalance(
    encoding: &Encoding,
    store: &StoreGetProto<PlainPoolDeployed>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityImbalance,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_remove_liquidity_imbalance", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_plain_pool_deployed(encoding, get_pair_created(store, &log.address), row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amounts", event.token_amounts.join(","));
    row.set("fees", event.fees.join(","));
    row.set("invariant", &event.invariant);
    row.set("token_supply", &event.token_supply);
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

    row.set("factory", bytes_to_string(&event.factory, encoding));
    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("coins", event.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
    row.set("A", &event.a);
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

    row.set("factory", bytes_to_string(&event.factory, encoding));
    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("coin", bytes_to_string(&event.coin, encoding));
    row.set("base_pool", bytes_to_string(&event.base_pool, encoding));
    row.set("A", &event.a);
    row.set("fee", &event.fee);
    row.set("deployer", bytes_to_string(&event.deployer, encoding));
}
