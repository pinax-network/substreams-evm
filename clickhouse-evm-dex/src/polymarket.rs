use common::{bytes_to_string, Encoding};
use proto::pb::polymarket::v1 as polymarket;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &polymarket::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(polymarket::log::Log::OrderFilled(event)) => {
                    process_order_filled(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::FeeCharged(event)) => {
                    process_fee_charged(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::NewAdmin(event)) => {
                    process_new_admin(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::NewOperator(event)) => {
                    process_new_operator(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::OrderCancelled(event)) => {
                    process_order_cancelled(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::OrdersMatched(event)) => {
                    process_orders_matched(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::ProxyFactoryUpdated(event)) => {
                    process_proxy_factory_updated(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::RemovedAdmin(event)) => {
                    process_removed_admin(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::RemovedOperator(event)) => {
                    process_removed_operator(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::SafeFactoryUpdated(event)) => {
                    process_safe_factory_updated(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::TokenRegistered(event)) => {
                    process_token_registered(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::TradingPaused(event)) => {
                    process_trading_paused(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(polymarket::log::Log::TradingUnpaused(event)) => {
                    process_trading_unpaused(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

fn process_order_filled(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::OrderFilled,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_order_filled", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("order_hash", common::bytes_to_hex(&event.order_hash));
    row.set("maker", bytes_to_string(&event.maker, encoding));
    row.set("taker", bytes_to_string(&event.taker, encoding));
    row.set("maker_asset_id", &event.maker_asset_id);
    row.set("taker_asset_id", &event.taker_asset_id);
    row.set("maker_amount_filled", &event.maker_amount_filled);
    row.set("taker_amount_filled", &event.taker_amount_filled);
    row.set("fee", &event.fee);
}

fn process_fee_charged(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::FeeCharged,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_fee_charged", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("receiver", bytes_to_string(&event.receiver, encoding));
    row.set("token_id", &event.token_id);
    row.set("amount", &event.amount);
}

fn process_new_admin(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::NewAdmin,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_new_admin", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("new_admin_address", bytes_to_string(&event.new_admin_address, encoding));
    row.set("admin", bytes_to_string(&event.admin, encoding));
}

fn process_new_operator(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::NewOperator,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_new_operator", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("new_operator_address", bytes_to_string(&event.new_operator_address, encoding));
    row.set("admin", bytes_to_string(&event.admin, encoding));
}

fn process_order_cancelled(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::OrderCancelled,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_order_cancelled", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("order_hash", common::bytes_to_hex(&event.order_hash));
}

fn process_orders_matched(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::OrdersMatched,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_orders_matched", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("taker_order_hash", common::bytes_to_hex(&event.taker_order_hash));
    row.set("taker_order_maker", bytes_to_string(&event.taker_order_maker, encoding));
    row.set("maker_asset_id", &event.maker_asset_id);
    row.set("taker_asset_id", &event.taker_asset_id);
    row.set("maker_amount_filled", &event.maker_amount_filled);
    row.set("taker_amount_filled", &event.taker_amount_filled);
}

fn process_proxy_factory_updated(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::ProxyFactoryUpdated,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_proxy_factory_updated", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("old_proxy_factory", bytes_to_string(&event.old_proxy_factory, encoding));
    row.set("new_proxy_factory", bytes_to_string(&event.new_proxy_factory, encoding));
}

fn process_removed_admin(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::RemovedAdmin,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_removed_admin", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("removed_admin", bytes_to_string(&event.removed_admin, encoding));
    row.set("admin", bytes_to_string(&event.admin, encoding));
}

fn process_removed_operator(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::RemovedOperator,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_removed_operator", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("removed_operator", bytes_to_string(&event.removed_operator, encoding));
    row.set("admin", bytes_to_string(&event.admin, encoding));
}

fn process_safe_factory_updated(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::SafeFactoryUpdated,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_safe_factory_updated", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("old_safe_factory", bytes_to_string(&event.old_safe_factory, encoding));
    row.set("new_safe_factory", bytes_to_string(&event.new_safe_factory, encoding));
}

fn process_token_registered(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::TokenRegistered,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_token_registered", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("condition_id", common::bytes_to_hex(&event.condition_id));
    row.set("token0", &event.token0);
    row.set("token1", &event.token1);
}

fn process_trading_paused(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::TradingPaused,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_trading_paused", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("pauser", bytes_to_string(&event.pauser, encoding));
}

fn process_trading_unpaused(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &polymarket::Transaction,
    log: &polymarket::Log,
    tx_index: usize,
    log_index: usize,
    event: &polymarket::TradingUnpaused,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("polymarket_trading_unpaused", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("pauser", bytes_to_string(&event.pauser, encoding));
}
