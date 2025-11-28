use common::{bytes_to_string, Encoding};
use proto::pb::evm::erc20::transfers::v1 as pb;
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
            // Transfer
            if let Some(pb::log::Log::Transfer(transfer)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("erc20_transfers", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tx(encoding, tx, tx_index, row);

                row.set("from", bytes_to_string(&transfer.from, encoding));
                row.set("to", bytes_to_string(&transfer.to, encoding));
                row.set("amount", &transfer.amount);
            }

            // Deposit
            if let Some(pb::log::Log::Deposit(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("weth_deposit", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tx(encoding, tx, tx_index, row);

                row.set("dst", bytes_to_string(&event.dst, encoding));
                row.set("wad", &event.wad);
            }

            // Withdrawal
            if let Some(pb::log::Log::Withdrawal(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("weth_withdrawal", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tx(encoding, tx, tx_index, row);

                row.set("src", bytes_to_string(&event.src, encoding));
                row.set("wad", &event.wad);
            }

            // Approval
            if let Some(pb::log::Log::Approval(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("erc20_approvals", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_tx(encoding, tx, tx_index, row);

                row.set("owner", bytes_to_string(&event.owner, encoding));
                row.set("spender", bytes_to_string(&event.spender, encoding));
                row.set("value", &event.value.to_string());
            }
        }
    }
}
