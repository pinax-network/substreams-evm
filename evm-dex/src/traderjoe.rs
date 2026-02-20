use common::{bytes_to_string, Encoding};
use proto::pb::traderjoe::v1::{self as traderjoe, StorePool};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &traderjoe::Events, store: &StoreGetProto<StorePool>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(traderjoe::log::Log::Swap(event)) => {
                    process_swap(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(traderjoe::log::Log::DepositedToBins(event)) => {
                    process_deposited(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(traderjoe::log::Log::WithdrawnFromBins(event)) => {
                    process_withdrawn(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(traderjoe::log::Log::CompositionFees(event)) => {
                    process_composition_fees(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(traderjoe::log::Log::LbPairCreated(event)) => {
                    process_lb_pair_created(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: StorePool, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("token0", bytes_to_string(&value.currency0, encoding));
    row.set("token1", bytes_to_string(&value.currency1, encoding));
    row.set("bin_step", value.bin_step);
}

fn process_swap(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &traderjoe::Transaction, log: &traderjoe::Log, tx_index: usize, log_index: usize, event: &traderjoe::Swap) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("traderjoe_swap", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("to", bytes_to_string(&event.to, encoding));
        row.set("id", event.id);
        row.set("amount_in_x", &event.amount_in_x);
        row.set("amount_in_y", &event.amount_in_y);
        row.set("amount_out_x", &event.amount_out_x);
        row.set("amount_out_y", &event.amount_out_y);
        row.set("volatility_accumulator", event.volatility_accumulator);
        row.set("total_fees_x", &event.total_fees_x);
        row.set("total_fees_y", &event.total_fees_y);
        row.set("protocol_fees_x", &event.protocol_fees_x);
        row.set("protocol_fees_y", &event.protocol_fees_y);
    }
}

fn process_deposited(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &traderjoe::Transaction, log: &traderjoe::Log, tx_index: usize, log_index: usize, event: &traderjoe::DepositedToBins) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("traderjoe_deposited_to_bins", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("to", bytes_to_string(&event.to, encoding));
    }
}

fn process_withdrawn(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &traderjoe::Transaction, log: &traderjoe::Log, tx_index: usize, log_index: usize, event: &traderjoe::WithdrawnFromBins) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("traderjoe_withdrawn_from_bins", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("to", bytes_to_string(&event.to, encoding));
    }
}

fn process_composition_fees(encoding: &Encoding, store: &StoreGetProto<StorePool>, tables: &mut Tables, clock: &Clock, tx: &traderjoe::Transaction, log: &traderjoe::Log, tx_index: usize, log_index: usize, event: &traderjoe::CompositionFees) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("traderjoe_composition_fees", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);
        row.set("sender", bytes_to_string(&event.sender, encoding));
        row.set("id", event.id);
        row.set("total_fees_x", &event.total_fees_x);
        row.set("total_fees_y", &event.total_fees_y);
        row.set("protocol_fees_x", &event.protocol_fees_x);
        row.set("protocol_fees_y", &event.protocol_fees_y);
    }
}

fn process_lb_pair_created(encoding: &Encoding, tables: &mut Tables, clock: &Clock, tx: &traderjoe::Transaction, log: &traderjoe::Log, tx_index: usize, log_index: usize, event: &traderjoe::LbPairCreated) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("traderjoe_lb_pair_created", key);
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    row.set("token_x", bytes_to_string(&event.token_x, encoding));
    row.set("token_y", bytes_to_string(&event.token_y, encoding));
    row.set("bin_step", event.bin_step);
    row.set("lb_pair", bytes_to_string(&event.lb_pair, encoding));
    row.set("pid", event.pid);
}
