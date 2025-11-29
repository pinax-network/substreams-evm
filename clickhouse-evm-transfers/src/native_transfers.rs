use common::{bytes_to_string, Encoding};
use proto::pb::evm::native::transfers::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{
    set_clock,
    transactions::{set_template_tx, tx_key},
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        // Transactions
        let key = tx_key(clock, tx_index);
        let row = tables.create_row("transactions", key);
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);

        row.set("tx_base_fee_per_gas", &tx.base_fee_per_gas);
        row.set("tx_transaction_fee", &tx.transaction_fee);
        row.set("tx_burn_fee", &tx.burn_fee);
        row.set("tx_fee_paid", &tx.fee_paid);

        // Calls
        for (call_index, call) in tx.calls.iter().enumerate() {
            let key = tx_key(clock, tx_index);
            let row = tables.create_row("calls", key);

            set_clock(clock, row);
            set_template_tx(encoding, tx, tx_index, row);

            row.set("call_index", call_index as u32);
            row.set("caller", bytes_to_string(&call.caller, encoding));
            row.set("address", bytes_to_string(&call.address, encoding));
            row.set("value", &call.value);
            row.set("gas_consumed", call.gas_consumed);
            row.set("gas_limit", call.gas_limit);
            row.set("depth", call.depth);
        }
    }
}
