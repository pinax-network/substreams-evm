use common::{bytes_to_string, Encoding};
use proto::pb::erc4626::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{log_key, logs::{set_template_call, set_template_log}, set_clock, transactions::set_template_erc4626_tx};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            // ERC-4626 Deposit
            if let Some(pb::log::Log::Deposit(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("erc4626_deposit", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_call(encoding, log, row);
                set_template_erc4626_tx(encoding, tx, tx_index, row);

                row.set("sender", bytes_to_string(&event.sender, encoding));
                row.set("owner", bytes_to_string(&event.owner, encoding));
                row.set("assets", &event.assets);
                row.set("shares", &event.shares);
            }

            // ERC-4626 Withdraw
            if let Some(pb::log::Log::Withdraw(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("erc4626_withdraw", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_call(encoding, log, row);
                set_template_erc4626_tx(encoding, tx, tx_index, row);

                row.set("sender", bytes_to_string(&event.sender, encoding));
                row.set("receiver", bytes_to_string(&event.receiver, encoding));
                row.set("owner", bytes_to_string(&event.owner, encoding));
                row.set("assets", &event.assets);
                row.set("shares", &event.shares);
            }
        }
    }
}
