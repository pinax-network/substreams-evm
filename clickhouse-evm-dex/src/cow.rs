use common::{bytes_to_string, Encoding};
use proto::pb::cow::v1::{self as cow};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &cow::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(cow::log::Log::Trade(event)) => {
                    process_trade(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(cow::log::Log::Settlement(event)) => {
                    process_settlement(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(cow::log::Log::OrderInvalidated(event)) => {
                    process_order_invalidated(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(cow::log::Log::PreSignature(event)) => {
                    process_pre_signature(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

fn process_trade(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &cow::Transaction,
    log: &cow::Log,
    tx_index: usize,
    log_index: usize,
    event: &cow::Trade,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("cow_trade", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    let sell_token_str = bytes_to_string(&event.sell_token, encoding);
    let buy_token_str = bytes_to_string(&event.buy_token, encoding);
    
    // Normalize token0/token1 lexicographically
    let (token0, token1) = if sell_token_str <= buy_token_str {
        (sell_token_str.clone(), buy_token_str.clone())
    } else {
        (buy_token_str.clone(), sell_token_str.clone())
    };

    row.set("owner", bytes_to_string(&event.owner, encoding));
    row.set("sell_token", sell_token_str);
    row.set("buy_token", buy_token_str);
    row.set("sell_amount", &event.sell_amount);
    row.set("buy_amount", &event.buy_amount);
    row.set("fee_amount", &event.fee_amount);
    row.set("order_uid", bytes_to_string(&event.order_uid, encoding));
    row.set("factory", bytes_to_string(&log.address, encoding));
    row.set("token0", token0);
    row.set("token1", token1);
}

fn process_settlement(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &cow::Transaction,
    log: &cow::Log,
    tx_index: usize,
    log_index: usize,
    event: &cow::Settlement,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("cow_settlement", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("solver", bytes_to_string(&event.solver, encoding));
    row.set("factory", bytes_to_string(&log.address, encoding));
}

fn process_order_invalidated(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &cow::Transaction,
    log: &cow::Log,
    tx_index: usize,
    log_index: usize,
    event: &cow::OrderInvalidated,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("cow_order_invalidated", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("owner", bytes_to_string(&event.owner, encoding));
    row.set("order_uid", bytes_to_string(&event.order_uid, encoding));
    row.set("factory", bytes_to_string(&log.address, encoding));
}

fn process_pre_signature(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &cow::Transaction,
    log: &cow::Log,
    tx_index: usize,
    log_index: usize,
    event: &cow::PreSignature,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("cow_pre_signature", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("owner", bytes_to_string(&event.owner, encoding));
    row.set("order_uid", bytes_to_string(&event.order_uid, encoding));
    row.set("signed", if event.signed { 1u8 } else { 0u8 });
    row.set("factory", bytes_to_string(&log.address, encoding));
}
