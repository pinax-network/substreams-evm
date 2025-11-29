use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::evm::native::transfers::v1 as native_pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

/// Generate a key for a transaction-based native transfer
pub fn native_tx_key(clock: &Clock, tx_index: usize) -> [(&'static str, String); 6] {
    let seconds = clock.timestamp.as_ref().expect("clock.timestamp is required").seconds;
    [
        ("minute", (seconds / 60).to_string()),
        ("timestamp", seconds.to_string()),
        ("block_num", clock.number.to_string()),
        ("tx_index", tx_index.to_string()),
        ("call_index", "0".to_string()), // No call index for transactions
        ("block_hash", format!("0x{}", &clock.id)),
    ]
}

/// Generate a key for a call-based native transfer
pub fn native_call_key(clock: &Clock, tx_index: usize, call_index: usize) -> [(&'static str, String); 6] {
    let seconds = clock.timestamp.as_ref().expect("clock.timestamp is required").seconds;
    [
        ("minute", (seconds / 60).to_string()),
        ("timestamp", seconds.to_string()),
        ("block_num", clock.number.to_string()),
        ("tx_index", tx_index.to_string()),
        ("call_index", call_index.to_string()),
        ("block_hash", format!("0x{}", &clock.id)),
    ]
}

/// Set template transaction fields for native transfers
pub fn set_native_template_tx(encoding: &Encoding, tx: &native_pb::Transaction, tx_index: usize, row: &mut substreams_database_change::tables::Row) {
    let tx_to = match &tx.to {
        Some(addr) => bytes_to_string(addr, encoding),
        None => "".to_string(),
    };
    row.set("tx_index", tx_index as u32);
    row.set("tx_hash", bytes_to_hex(&tx.hash));
    row.set("tx_from", bytes_to_string(&tx.from, encoding));
    row.set("tx_to", tx_to);
    row.set("tx_nonce", tx.nonce);
    row.set("tx_gas_price", &tx.gas_price);
    row.set("tx_gas_limit", tx.gas_limit);
    row.set("tx_gas_used", tx.gas_used);
    row.set("tx_value", &tx.value);
}

/// Set call-specific template fields
pub fn set_template_call(call: &native_pb::Call, call_index: usize, row: &mut substreams_database_change::tables::Row) {
    row.set("call_index", call_index as u32);
    row.set("call_gas_consumed", call.gas_consumed);
    row.set("call_gas_limit", call.gas_limit);
    row.set("call_depth", call.depth);
}

/// Process native transfer events from native-transfers substream
pub fn process_native_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &native_pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        // Process transaction-level value transfers (value > 0)
        if tx.value != "0" && !tx.value.is_empty() {
            let key = native_tx_key(clock, tx_index);
            let row = tables.create_row("native_transfers", key);

            set_clock(clock, row);
            set_native_template_tx(encoding, tx, tx_index, row);

            // Set call fields to default for transaction-type transfers
            row.set("call_index", 0u32);
            row.set("call_gas_consumed", 0u64);
            row.set("call_gas_limit", 0u64);
            row.set("call_depth", 0u32);

            // Transfer details
            row.set("type", "transaction");
            let from = bytes_to_string(&tx.from, encoding);
            let tx_to = match &tx.to {
                Some(addr) => bytes_to_string(addr, encoding),
                None => "".to_string(),
            };
            // Handle None 'to' address cases (withdrew unstaked asset, claim rewards, etc.)
            if tx.to.is_none() {
                row.set("from", tx_to);
                row.set("to", from);
            } else {
                row.set("from", from);
                row.set("to", tx_to);
            }
            row.set("amount", &tx.value);
        }

        // Process call-level value transfers
        for (call_index, call) in tx.calls.iter().enumerate() {
            if call.value != "0" && !call.value.is_empty() {
                let key = native_call_key(clock, tx_index, call_index);
                let row = tables.create_row("native_transfers", key);

                set_clock(clock, row);
                set_native_template_tx(encoding, tx, tx_index, row);
                set_template_call(call, call_index, row);

                // Transfer details
                row.set("type", "call");
                row.set("from", bytes_to_string(&call.caller, encoding));
                row.set("to", bytes_to_string(&call.address, encoding));
                row.set("amount", &call.value);
            }
        }
    }
}

/// Process transactions with fee information for the transactions table
pub fn process_transaction_fees(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &native_pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        let seconds = clock.timestamp.as_ref().expect("clock.timestamp is required").seconds;
        let key: [(&str, String); 5] = [
            ("minute", (seconds / 60).to_string()),
            ("timestamp", seconds.to_string()),
            ("block_num", clock.number.to_string()),
            ("tx_index", tx_index.to_string()),
            ("block_hash", format!("0x{}", &clock.id)),
        ];
        let row = tables.create_row("transactions", key);

        set_clock(clock, row);
        set_native_template_tx(encoding, tx, tx_index, row);

        // Fee information
        row.set("base_fee_per_gas", &tx.base_fee_per_gas);
        row.set("transaction_fee", &tx.transaction_fee);
        row.set("burn_fee", &tx.burn_fee);
        row.set("fee_paid", &tx.fee_paid);
    }
}
