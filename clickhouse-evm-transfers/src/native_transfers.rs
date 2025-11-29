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
        let tx_row = tables.create_row("transactions", key);
        set_clock(clock, tx_row);
        set_template_tx(encoding, tx, tx_index, tx_row);

        tx_row.set("tx_base_fee_per_gas", &tx.base_fee_per_gas);
        tx_row.set("tx_transaction_fee", &tx.transaction_fee);
        tx_row.set("tx_burn_fee", &tx.burn_fee);
        tx_row.set("tx_fee_paid", &tx.fee_paid);

        // Calls
        for (call_index, call) in tx.calls.iter().enumerate() {
            let key = tx_key(clock, tx_index);
            let call_row = tables.create_row("calls", key);

            set_clock(clock, call_row);
            set_template_tx(encoding, tx, tx_index, call_row);

            call_row.set("call_index", call_index as u32);
            call_row.set("caller", bytes_to_string(&call.caller, encoding));
            call_row.set("address", bytes_to_string(&call.address, encoding));
            call_row.set("value", &call.value);
            call_row.set("gas_consumed", call.gas_consumed);
            call_row.set("gas_limit", call.gas_limit);
            call_row.set("depth", call.depth);
        }
    }
}
