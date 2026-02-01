use common::{bytes_to_string, Encoding};
use proto::pb::native::transfers::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{call_key, reward_key, set_clock, transactions::set_template_native_tx, tx_key};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    // Block Rewards
    for (index, event) in events.block_rewards.iter().enumerate() {
        let key = reward_key(clock, index);
        let row = tables.create_row("block_rewards", key);
        set_clock(clock, row);
        row.set("miner", bytes_to_string(&event.miner, encoding));
        row.set("value", &event.value);
        row.set("reason", &event.reason().as_str_name().to_string());
    }

    // Validator Withdrawals (post-Shanghai)
    for (index, event) in events.withdrawals.iter().enumerate() {
        let key = reward_key(clock, index);
        let row = tables.create_row("withdrawals", key);
        set_clock(clock, row);
        row.set("address", bytes_to_string(&event.address, encoding));
        row.set("value", &event.value);
    }

    // Selfdestructs
    for (index, event) in events.selfdestructs.iter().enumerate() {
        let key = reward_key(clock, index);
        let row = tables.create_row("selfdestructs", key);
        set_clock(clock, row);
        row.set("tx_hash", bytes_to_string(&event.tx_hash, encoding));
        row.set("from_address", bytes_to_string(&event.from, encoding));
        row.set("to_address", bytes_to_string(&event.to, encoding));
        row.set("value", &event.value);
    }

    // Genesis Balances (block 0)
    for (index, event) in events.genesis_balances.iter().enumerate() {
        let key = reward_key(clock, index);
        let row = tables.create_row("genesis_balances", key);
        set_clock(clock, row);
        row.set("address", bytes_to_string(&event.address, encoding));
        row.set("value", &event.value);
    }

    // DAO hard fork transfers
    for (index, event) in events.dao_transfers.iter().enumerate() {
        let key = reward_key(clock, index);
        let row = tables.create_row("dao_transfers", key);
        set_clock(clock, row);
        row.set("address", bytes_to_string(&event.address, encoding));
        row.set("old_value", &event.old_value);
        row.set("new_value", &event.new_value);
        row.set("reason", &event.reason().as_str_name().to_string());
    }

    for (tx_index, tx) in events.transactions.iter().enumerate() {
        // Transactions
        let key = tx_key(clock, tx_index);
        let tx_row = tables.create_row("transactions", key);
        set_clock(clock, tx_row);
        set_template_native_tx(encoding, tx, tx_index, tx_row);

        // Calls
        for (call_index, call) in tx.calls.iter().enumerate() {
            let key = call_key(clock, tx_index, call_index);
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
            call_row.set("call_type", pb::CallType::try_from(call.call_type).unwrap_or_default().as_str_name());
        }
    }
}
