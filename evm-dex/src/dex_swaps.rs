/// Processes unified `DexSwaps` output and writes rows directly to the
/// `swaps` and `swaps_flash` ClickHouse/PostgreSQL tables.
///
/// This replaces the ClickHouse materialized-view layer that previously
/// converted protocol-specific swap tables into the `swaps` table.  Moving
/// the conversion into Substreams means the logic is tested, versioned, and
/// cached at the protocol layer rather than scattered across many MVs.
use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::dex::v1::{DexSwap, DexSwapFlash, DexSwaps};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_dex_swaps(encoding: &Encoding, tables: &mut Tables, clock: &Clock, dex_swaps: &DexSwaps) {
    for swap in &dex_swaps.swaps {
        write_swap(encoding, tables, clock, swap);
    }
    for flash in &dex_swaps.swaps_flash {
        write_swap_flash(encoding, tables, clock, flash);
    }
}
fn write_swap(encoding: &Encoding, tables: &mut Tables, clock: &Clock, swap: &DexSwap) {
    let seconds = clock.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0);
    let key = [
        ("minute", (seconds / 60).to_string()),
        ("timestamp", seconds.to_string()),
        ("block_num", clock.number.to_string()),
        ("tx_index", swap.tx_index.to_string()),
        ("log_index", swap.log_index.to_string()),
        ("block_hash", format!("0x{}", &clock.id)),
    ];

    let row = tables.create_row("swaps", key);

    // block
    set_clock(clock, row);

    // transaction
    row.set("tx_index", swap.tx_index);
    row.set("tx_hash", bytes_to_hex(&swap.tx_hash));
    row.set("tx_from", bytes_to_string(&swap.tx_from, encoding));
    row.set("tx_to", swap.tx_to.as_deref().map(|b| bytes_to_string(b, encoding)).unwrap_or_default());
    row.set("tx_nonce", swap.tx_nonce);
    row.set("tx_gas_price", &swap.tx_gas_price);
    row.set("tx_gas_limit", swap.tx_gas_limit);
    row.set("tx_gas_used", swap.tx_gas_used);
    row.set("tx_value", &swap.tx_value);

    // call (defaults to empty when not available)
    let call = swap.call.as_ref();
    row.set("call_caller", call.map(|c| bytes_to_string(&c.caller, encoding)).unwrap_or_default());
    row.set("call_index", call.map(|c| c.index).unwrap_or_default());
    row.set("call_begin_ordinal", call.map(|c| c.begin_ordinal).unwrap_or_default());
    row.set("call_end_ordinal", call.map(|c| c.end_ordinal).unwrap_or_default());
    row.set("call_address", call.map(|c| bytes_to_string(&c.address, encoding)).unwrap_or_default());
    row.set("call_value", call.map(|c| c.value.as_str()).unwrap_or_default());
    row.set("call_gas_consumed", call.map(|c| c.gas_consumed).unwrap_or_default());
    row.set("call_gas_limit", call.map(|c| c.gas_limit).unwrap_or_default());
    row.set("call_depth", call.map(|c| c.depth).unwrap_or_default());
    row.set("call_parent_index", call.map(|c| c.parent_index).unwrap_or_default());
    row.set("call_type", call.map(|c| call_type_name(c.call_type)).unwrap_or("CALL_TYPE_UNSPECIFIED"));

    // log
    row.set("log_index", swap.log_index);
    row.set("log_block_index", swap.log_block_index);
    row.set("log_address", bytes_to_string(&swap.log_address, encoding));
    row.set("log_ordinal", swap.log_ordinal);
    row.set("log_topic0", swap.log_topics.first().map(|t| bytes_to_hex(t)).unwrap_or_default());

    // swap fields
    row.set("protocol", &swap.protocol);
    row.set("factory", bytes_to_string(&swap.factory, encoding));
    row.set("pool", bytes_to_string(&swap.pool, encoding));
    row.set("user", bytes_to_string(&swap.user, encoding));
    row.set("input_contract", bytes_to_string(&swap.input_contract, encoding));
    row.set("input_amount", &swap.input_amount);
    row.set("output_contract", bytes_to_string(&swap.output_contract, encoding));
    row.set("output_amount", &swap.output_amount);
}

/// Writes a single `DexSwapFlash` into the `swaps_flash` table.
fn write_swap_flash(encoding: &Encoding, tables: &mut Tables, clock: &Clock, flash: &DexSwapFlash) {
    let seconds = clock.timestamp.as_ref().map(|t| t.seconds).unwrap_or(0);
    let key = [
        ("minute", (seconds / 60).to_string()),
        ("timestamp", seconds.to_string()),
        ("block_num", clock.number.to_string()),
        ("tx_index", flash.tx_index.to_string()),
        ("log_index", flash.log_index.to_string()),
        ("block_hash", format!("0x{}", &clock.id)),
    ];

    let row = tables.create_row("swaps_flash", key);

    // block
    set_clock(clock, row);

    // transaction
    row.set("tx_index", flash.tx_index);
    row.set("tx_hash", bytes_to_hex(&flash.tx_hash));
    row.set("tx_from", bytes_to_string(&flash.tx_from, encoding));
    row.set("tx_to", flash.tx_to.as_deref().map(|b| bytes_to_string(b, encoding)).unwrap_or_default());
    row.set("tx_nonce", flash.tx_nonce);
    row.set("tx_gas_price", &flash.tx_gas_price);
    row.set("tx_gas_limit", flash.tx_gas_limit);
    row.set("tx_gas_used", flash.tx_gas_used);
    row.set("tx_value", &flash.tx_value);

    // call
    let call = flash.call.as_ref();
    row.set("call_caller", call.map(|c| bytes_to_string(&c.caller, encoding)).unwrap_or_default());
    row.set("call_index", call.map(|c| c.index).unwrap_or_default());
    row.set("call_begin_ordinal", call.map(|c| c.begin_ordinal).unwrap_or_default());
    row.set("call_end_ordinal", call.map(|c| c.end_ordinal).unwrap_or_default());
    row.set("call_address", call.map(|c| bytes_to_string(&c.address, encoding)).unwrap_or_default());
    row.set("call_value", call.map(|c| c.value.as_str()).unwrap_or_default());
    row.set("call_gas_consumed", call.map(|c| c.gas_consumed).unwrap_or_default());
    row.set("call_gas_limit", call.map(|c| c.gas_limit).unwrap_or_default());
    row.set("call_depth", call.map(|c| c.depth).unwrap_or_default());
    row.set("call_parent_index", call.map(|c| c.parent_index).unwrap_or_default());
    row.set("call_type", call.map(|c| call_type_name(c.call_type)).unwrap_or("CALL_TYPE_UNSPECIFIED"));

    // log
    row.set("log_index", flash.log_index);
    row.set("log_block_index", flash.log_block_index);
    row.set("log_address", bytes_to_string(&flash.log_address, encoding));
    row.set("log_ordinal", flash.log_ordinal);
    row.set("log_topic0", flash.log_topics.first().map(|t| bytes_to_hex(t)).unwrap_or_default());

    // flash swap fields
    row.set("protocol", &flash.protocol);
    row.set("factory", bytes_to_string(&flash.factory, encoding));
    row.set("pool", bytes_to_string(&flash.pool, encoding));
    row.set("user", bytes_to_string(&flash.user, encoding));
    row.set("token0", bytes_to_string(&flash.token0, encoding));
    row.set("token1", bytes_to_string(&flash.token1, encoding));
    row.set("amount0_in", &flash.amount0_in);
    row.set("amount1_in", &flash.amount1_in);
    row.set("amount0_out", &flash.amount0_out);
    row.set("amount1_out", &flash.amount1_out);
}

fn call_type_name(call_type: i32) -> &'static str {
    match call_type {
        1 => "CALL_TYPE_CALL",
        2 => "CALL_TYPE_CALLCODE",
        3 => "CALL_TYPE_DELEGATE",
        4 => "CALL_TYPE_STATIC",
        5 => "CALL_TYPE_CREATE",
        _ => "CALL_TYPE_UNSPECIFIED",
    }
}
