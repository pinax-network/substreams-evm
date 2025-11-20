use common::{bytes_to_string, Encoding};
use proto::pb::evm::transfers::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            if let Some(pb::log::Log::Transfer(transfer)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("erc20_transfer", key);

                // TEMPLATE
                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tx(encoding, tx, tx_index, row);

                // Transfer
                row.set("from", bytes_to_string(&transfer.from, encoding));
                row.set("to", bytes_to_string(&transfer.to, encoding));
                row.set("amount", &transfer.amount);
            }
        }
    }
}
