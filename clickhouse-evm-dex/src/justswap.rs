use common::{bytes_to_string, Encoding};
use proto::pb::justswap::v1::{self as justswap, NewExchange};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    foundational_stores::get_new_exchange,
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

// JustSwap Processing
pub fn process_events(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    events: &justswap::Events,
    store: &StoreGetProto<NewExchange>,
) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(justswap::log::Log::TokenPurchase(swap)) => {
                    process_justswap_token_purchase(encoding, store, tables, clock, tx, log, tx_index, log_index, swap);
                }
                Some(justswap::log::Log::TrxPurchase(swap)) => {
                    process_justswap_trx_purchase(encoding, store, tables, clock, tx, log, tx_index, log_index, swap);
                }
                Some(justswap::log::Log::AddLiquidity(event)) => {
                    process_justswap_add_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(justswap::log::Log::RemoveLiquidity(event)) => {
                    process_justswap_remove_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(justswap::log::Log::Snapshot(event)) => {
                    process_justswap_snapshot(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(justswap::log::Log::NewExchange(event)) => {
                    process_justswap_new_exchange(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {} // Ignore other event types
            }
        }
    }
}

pub fn set_new_exchange(encoding: &Encoding, value: Option<NewExchange>, row: &mut substreams_database_change::tables::Row) {
    if let Some(value) = value {
        row.set("factory", bytes_to_string(&value.factory, encoding));
        row.set("token", bytes_to_string(&value.token, encoding));
        substreams::log::info!(
            "NewExchange found: factory={}, token={}",
            bytes_to_string(&value.factory, encoding),
            bytes_to_string(&value.token, encoding),
        );
    } else {
        row.set("factory", "");
        row.set("token", "");
        substreams::log::info!("NewExchange not found");
    }
}

fn process_justswap_token_purchase(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &justswap::Transaction,
    log: &justswap::Log,
    tx_index: usize,
    log_index: usize,
    swap: &justswap::TokenPurchase,
) {
    // Create the row and populate common fields
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("justswap_token_purchase", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Set NewExchange event data
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    // Swap info - TRX -> Token
    row.set("buyer", bytes_to_string(&swap.buyer, encoding));
    row.set("trx_sold", &swap.trx_sold);
    row.set("tokens_bought", &swap.tokens_bought);
}

fn process_justswap_trx_purchase(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &justswap::Transaction,
    log: &justswap::Log,
    tx_index: usize,
    log_index: usize,
    swap: &justswap::TrxPurchase,
) {
    // Create the row and populate common fields
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("justswap_trx_purchase", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Set NewExchange event data
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    // Swap info - Token -> TRX
    row.set("buyer", bytes_to_string(&swap.buyer, encoding));

    // Token is input, TRX is output
    row.set("tokens_sold", &swap.tokens_sold);
    row.set("trx_bought", &swap.trx_bought);
}

fn process_justswap_add_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &justswap::Transaction,
    log: &justswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &justswap::AddLiquidity,
) {
    // Create the row and populate common fields
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("justswap_add_liquidity", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Set NewExchange event data
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    // Event info
    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("trx_amount", &event.trx_amount);
    row.set("token_amount", &event.token_amount);
}

fn process_justswap_remove_liquidity(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &justswap::Transaction,
    log: &justswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &justswap::RemoveLiquidity,
) {
    // Create the row and populate common fields
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("justswap_remove_liquidity", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Set NewExchange event data
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    // Event info
    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("trx_amount", &event.trx_amount);
    row.set("token_amount", &event.token_amount);
}

fn process_justswap_snapshot(
    encoding: &Encoding,
    store: &StoreGetProto<NewExchange>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &justswap::Transaction,
    log: &justswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &justswap::Snapshot,
) {
    // Create the row and populate common fields
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("justswap_snapshot", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Set NewExchange event data
    set_new_exchange(encoding, get_new_exchange(store, &log.address), row);

    // Event info
    row.set("operator", bytes_to_string(&event.operator, encoding));
    row.set("trx_balance", &event.trx_balance);
    row.set("token_balance", &event.token_balance);
}

fn process_justswap_new_exchange(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &justswap::Transaction,
    log: &justswap::Log,
    tx_index: usize,
    log_index: usize,
    event: &justswap::NewExchange,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("justswap_new_exchange", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("exchange", bytes_to_string(&event.exchange, encoding));
    row.set("token", bytes_to_string(&event.token, encoding));
}
