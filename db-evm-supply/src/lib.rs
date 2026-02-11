mod supply;

use proto::pb::erc20::supply::v1 as pb;
use substreams::{errors::Error, pb::substreams::Clock};
use substreams_database_change::{pb::sf::substreams::sink::database::v1::DatabaseChanges, tables::Row};

#[substreams::handlers::map]
pub fn db_out(params: String, mut clock: Clock, supply_events: pb::Events) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    // Handle support both EVM & TVM address encoding
    let encoding = common::handle_encoding_param(&params);

    // -- Supply --
    supply::process_events(&encoding, &mut tables, &clock, &supply_events);

    // ONLY include blocks if events are present
    if tables.all_row_count() > 0 {
        set_clock(&clock, tables.upsert_row("blocks", [("block_num", clock.number.to_string())]));
    }

    substreams::log::info!("Total rows {}", tables.all_row_count());
    Ok(tables.to_database_changes())
}

// Helper function to set clock data in a row
pub fn set_clock(clock: &Clock, row: &mut Row) {
    row.set("block_num", clock.number.to_string())
        .set("block_hash", format!("0x{}", clock.id))
        .set("timestamp", clock.timestamp.as_ref().expect("missing timestamp").seconds.to_string());
}
