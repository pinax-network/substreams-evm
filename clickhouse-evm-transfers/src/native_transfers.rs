use common::{bytes_to_string, Encoding};
use proto::pb::native::transfers::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{clock_key, set_clock, transactions::set_template_native_tx};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for event in &events.block_rewards {
        let row = tables.create_row("block_rewards", clock.number.to_string());
        set_clock(clock, row);
        row.set("miner", bytes_to_string(&event.miner, encoding));
        row.set("value", &event.value);
        row.set("reason", &event.reason().as_str_name().to_string());
    }

    for (tx_index, tx) in events.transactions.iter().enumerate() {
        // Transactions
        let key = clock_key(clock);
        let tx_row = tables.create_row("transactions", key);
        set_clock(clock, tx_row);
        set_template_native_tx(encoding, tx, tx_index, tx_row);

        // Calls
        for (call_index, call) in tx.calls.iter().enumerate() {
            let key = clock_key(clock);
            let call_row = tables.create_row("calls", key);

            set_clock(clock, call_row);
            set_template_native_tx(encoding, tx, tx_index, call_row);

            call_row.set("call_index", call_index as u64);
            call_row.set("call_begin_ordinal", call.begin_ordinal);
            call_row.set("call_end_ordinal", call.end_ordinal);
            call_row.set("call_parent_index", call.parent_index);
            call_row.set("call_caller", bytes_to_string(&call.caller, encoding));
            call_row.set("call_address", bytes_to_string(&call.address, encoding));
            call_row.set("call_value", &call.value);
            call_row.set("call_gas_consumed", call.gas_consumed);
            call_row.set("call_gas_limit", call.gas_limit);
            call_row.set("call_depth", call.depth);
        }
    }
}
