use core::panic;

use prost_types::Timestamp;
mod erc20_balances;
mod native_balances;
use proto::pb::evm::balances::v1 as pb;
use substreams::{errors::Error, pb::substreams::Clock};
use substreams_database_change::pb::database::DatabaseChanges;

#[substreams::handlers::map]
pub fn db_out(
    params: String,
    mut clock: Clock,
    // Native
    native_balances: pb::Events,
    // ERC-20
    erc20_balances: pb::Events,
) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();
    clock = update_genesis_clock(clock);

    // Handle support both EVM & TVM address encoding
    let encoding = common::handle_encoding_param(&params);

    // -- ERC20 Balances --
    erc20_balances::process_events(&encoding, &mut tables, &clock, &erc20_balances);

    // -- Native Balances --
    native_balances::process_events(&encoding, &mut tables, &clock, &native_balances);

    // ONLY include blocks if events are present
    if !tables.tables.is_empty() {
        set_clock(&clock, tables.create_row("blocks", [("block_num", clock.number.to_string())]));
    }
    substreams::log::info!("Total rows {}", tables.all_row_count());
    Ok(tables.to_database_changes())
}

pub fn set_clock(clock: &Clock, row: &mut substreams_database_change::tables::Row) {
    row.set("block_num", clock.number);
    row.set("block_hash", format!("0x{}", clock.id));
    if let Some(timestamp) = &clock.timestamp {
        row.set("timestamp", timestamp.seconds);
    }
}

pub fn update_genesis_clock(mut clock: Clock) -> Clock {
    // only applies to the first block of the stream
    if clock.number != 0 {
        return clock;
    }
    // ETH Mainnet
    if clock.id == "d4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3" {
        clock.timestamp = Some(Timestamp { seconds: 1438269973, nanos: 0 });
        return clock;
    // Arbitrum One
    } else if clock.id == "7ee576b35482195fc49205cec9af72ce14f003b9ae69f6ba0faef4514be8b442" {
        clock.timestamp = Some(Timestamp { seconds: 1622240000, nanos: 0 });
        return clock;
    // Arbitrum Nova
    } else if clock.id == "2ad24e03026118f9b3a48626f0636e38c93660e90a6812e853a99aa8c5371561" {
        clock.timestamp = Some(Timestamp { seconds: 1656120000, nanos: 0 });
        return clock;
    // Boba
    } else if clock.id == "dcd9e6a8f9973eaa62da2874959cb152faeb4fd6929177bd6335a1a16074ef9c" {
        clock.timestamp = Some(Timestamp {
            seconds: 1635393439, // Block 1
            nanos: 0,
        });
        return clock;
    }
    clock
}
