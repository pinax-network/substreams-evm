use common::{bytes_to_string, Encoding};
use proto::pb::cow::v1::{self as cow};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{logs::set_template_log, set_clock, transactions::set_template_tx};

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

fn log_key(clock: &Clock, tx_index: usize, log_index: usize) -> [(&'static str, String); 5] {
    [
        ("block_num", clock.number.to_string()),
        ("block_hash", format!("0x{}", clock.id)),
        ("tx_index", tx_index.to_string()),
        ("log_index", log_index.to_string()),
        ("timestamp", clock.timestamp.as_ref().map(|t| t.seconds.to_string()).unwrap_or_default()),
    ]
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

    row.set("owner", bytes_to_string(&event.owner, encoding));
    row.set("sell_token", bytes_to_string(&event.sell_token, encoding));
    row.set("buy_token", bytes_to_string(&event.buy_token, encoding));
    row.set("sell_amount", &event.sell_amount);
    row.set("buy_amount", &event.buy_amount);
    row.set("fee_amount", &event.fee_amount);
    row.set("order_uid", bytes_to_string(&event.order_uid, encoding));
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
}
