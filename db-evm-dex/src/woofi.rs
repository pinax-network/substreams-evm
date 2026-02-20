use common::{bytes_to_string, Encoding};
use proto::pb::woofi::v1 as woofi;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &woofi::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(woofi::log::Log::WooSwap(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("woofi_woo_swap", key);
                set_clock(clock, row);
                set_template_tx(encoding, tx, tx_index, row);
                set_template_log(encoding, log, log_index, row);
                row.set("from_token", bytes_to_string(&event.from_token, encoding));
                row.set("to_token", bytes_to_string(&event.to_token, encoding));
                row.set("from_amount", &event.from_amount);
                row.set("to_amount", &event.to_amount);
                row.set("from", bytes_to_string(&event.from, encoding));
                row.set("to", bytes_to_string(&event.to, encoding));
                row.set("rebate_to", bytes_to_string(&event.rebate_to, encoding));
                row.set("swap_vol", &event.swap_vol);
                row.set("swap_fee", &event.swap_fee);
            }
        }
    }
}
