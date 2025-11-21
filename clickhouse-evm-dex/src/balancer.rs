use common::{bytes_to_string, Encoding};
use proto::pb::balancer::v1::{self as balancer};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &balancer::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(balancer::log::Log::VaultSwap(event)) => {
                    process_vault_swap(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::LiquidityAdded(event)) => {
                    process_liquidity_added(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::LiquidityRemoved(event)) => {
                    process_liquidity_removed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(balancer::log::Log::PoolRegistered(event)) => {
                    process_pool_registered(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

fn process_vault_swap(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::VaultSwap,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("balancer_vault_swap", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("token_in", bytes_to_string(&event.token_in, encoding));
    row.set("token_out", bytes_to_string(&event.token_out, encoding));
    row.set("amount_in", &event.amount_in);
    row.set("amount_out", &event.amount_out);
    row.set("swap_fee_percentage", &event.swap_fee_percentage);
    row.set("swap_fee_amount", &event.swap_fee_amount);
}

fn process_liquidity_added(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::LiquidityAdded,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("balancer_liquidity_added", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("liquidity_provider", bytes_to_string(&event.liquidity_provider, encoding));
    row.set("kind", event.kind);
    row.set("total_supply", &event.total_supply);
    row.set("amounts_added_raw", event.amounts_added_raw.join(","));
    row.set("swap_fee_amounts_raw", event.swap_fee_amounts_raw.join(","));
}

fn process_liquidity_removed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::LiquidityRemoved,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("balancer_liquidity_removed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("liquidity_provider", bytes_to_string(&event.liquidity_provider, encoding));
    row.set("kind", event.kind);
    row.set("total_supply", &event.total_supply);
    row.set("amounts_removed_raw", event.amounts_removed_raw.join(","));
    row.set("swap_fee_amounts_raw", event.swap_fee_amounts_raw.join(","));
}

fn process_pool_registered(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &balancer::Transaction,
    log: &balancer::Log,
    tx_index: usize,
    log_index: usize,
    event: &balancer::PoolRegistered,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("balancer_pool_registered", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("factory", bytes_to_string(&event.factory, encoding));
}
