use common::{bytes_to_string, Encoding};
use proto::pb::uniswap::v1::{self as uniswap, NewExchange};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    foundational_stores::get_new_exchange,
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &uniswap::Events, store: &StoreGetProto<NewExchange>) {
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

pub fn set_new_exchange(encoding: &Encoding, value: Option<NewExchange>, row: &mut substreams_database_change::tables::Row) {
    if let Some(value) = value {
        row.set("factory", bytes_to_string(&value.factory, encoding));
        row.set("token", bytes_to_string(&value.token, encoding));
    } else {
        row.set("factory", "");
        row.set("token", "");
    }
}

fn process_token_purchase(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::TokenPurchase,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v1_token_purchase", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    row.set("buyer", bytes_to_string(&event.buyer, encoding));
    row.set("eth_sold", &event.eth_sold);
    row.set("tokens_bought", &event.tokens_bought);
}

fn process_eth_purchase(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::EthPurchase,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v1_eth_purchase", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    row.set("buyer", bytes_to_string(&event.buyer, encoding));
    row.set("tokens_sold", &event.tokens_sold);
    row.set("eth_bought", &event.eth_bought);
}

fn process_add_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::AddLiquidity,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v1_add_liquidity", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("eth_amount", &event.eth_amount);
    row.set("token_amount", &event.token_amount);
}

fn process_remove_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &uniswap::Transaction,
    log: &uniswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &uniswap::RemoveLiquidity,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("uniswap_v1_remove_liquidity", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("eth_amount", &event.eth_amount);
    row.set("token_amount", &event.token_amount);
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

    row.set("exchange", bytes_to_string(&event.exchange, encoding));
    row.set("token", bytes_to_string(&event.token, encoding));
}
