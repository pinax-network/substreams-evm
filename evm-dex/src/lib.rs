mod normalized;

use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams_database_change::pb::database::DatabaseChanges;

#[substreams::handlers::map]
pub fn db_out(
    params: String,
    clock: Clock,
    events_swaps: proto::pb::dex::swaps::v1::Events,
) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    let encoding = common::handle_encoding_param(&params);
    normalized::process_events(&encoding, &mut tables, &clock, &events_swaps);

    if !tables.tables.is_empty() {
        common::clickhouse::set_clock(&clock, tables.create_row("blocks", [("block_num", clock.number.to_string())]));
    }

    substreams::log::info!("Total rows {}", tables.all_row_count());
    Ok(tables.to_database_changes())
}
