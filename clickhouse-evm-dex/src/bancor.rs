use common::{bytes_to_string, Encoding};
use proto::pb::bancor::v1::{self as bancor};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{logs::set_template_log, set_clock, transactions::set_template_tx};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &bancor::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(bancor::log::Log::Conversion(event)) => {
                    process_conversion(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::LiquidityAdded(event)) => {
                    process_liquidity_added(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::LiquidityRemoved(event)) => {
                    process_liquidity_removed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(bancor::log::Log::TokenRateUpdate(event)) => {
                    process_token_rate_update(encoding, tables, clock, tx, log, tx_index, log_index, event);
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

fn process_conversion(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::Conversion,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_conversion", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("source_token", bytes_to_string(&event.source_token, encoding));
    row.set("target_token", bytes_to_string(&event.target_token, encoding));
    row.set("trader", bytes_to_string(&event.trader, encoding));
    row.set("source_amount", &event.source_amount);
    row.set("target_amount", &event.target_amount);
    row.set("conversion_fee", &event.conversion_fee);
}

fn process_liquidity_added(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::LiquidityAdded,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_liquidity_added", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("reserve_token", bytes_to_string(&event.reserve_token, encoding));
    row.set("amount", &event.amount);
    row.set("new_balance", &event.new_balance);
    row.set("new_supply", &event.new_supply);
}

fn process_liquidity_removed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::LiquidityRemoved,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_liquidity_removed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("reserve_token", bytes_to_string(&event.reserve_token, encoding));
    row.set("amount", &event.amount);
    row.set("new_balance", &event.new_balance);
    row.set("new_supply", &event.new_supply);
}

fn process_token_rate_update(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &bancor::Transaction,
    log: &bancor::Log,
    tx_index: usize,
    log_index: usize,
    event: &bancor::TokenRateUpdate,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("bancor_token_rate_update", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("token1", bytes_to_string(&event.token1, encoding));
    row.set("token2", bytes_to_string(&event.token2, encoding));
    row.set("rate_n", &event.rate_n);
    row.set("rate_d", &event.rate_d);
}
