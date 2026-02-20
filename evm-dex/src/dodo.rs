use common::{bytes_to_string, Encoding};
use proto::pb::dodo::v1 as dodo;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &dodo::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(dodo::log::Log::OrderHistory(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("dodo_order_history", key);
                set_clock(clock, row);
                set_template_tx(encoding, tx, tx_index, row);
                set_template_log(encoding, log, log_index, row);
                row.set("from_token", bytes_to_string(&event.from_token, encoding));
                row.set("to_token", bytes_to_string(&event.to_token, encoding));
                row.set("sender", bytes_to_string(&event.sender, encoding));
                row.set("from_amount", &event.from_amount);
                row.set("return_amount", &event.return_amount);
            }
        }
    }
}
