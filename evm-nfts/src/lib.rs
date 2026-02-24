mod cryptopunks;
mod enums;
mod erc1155;
mod erc1155_metadata;
mod erc721;
mod erc721_metadata;
mod seaport;
mod to_json;

use proto::pb::evm;
use substreams::pb::substreams::Clock;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;

#[substreams::handlers::map]
pub fn db_out(
    params: String,
    clock: Clock,
    erc721_events: evm::erc721::v1::Events,
    erc721_metadata_events: evm::erc721::metadata::v1::Events,
    erc1155_events: evm::erc1155::v1::Events,
    erc1155_metadata_events: evm::erc1155::metadata::v1::Events,
    seaport_events: evm::seaport::v1::Events,
    erc721_cryptopunks_events: evm::erc721::v1::Events,
    cryptopunks_events: evm::cryptopunks::v1::Events,
) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = Tables::new();

    // Handle support both EVM & TVM address encoding
    let encoding = common::handle_encoding_param(&params);

    // Process packages
    erc721::process_erc721(&mut tables, &clock, erc721_events, &encoding);
    erc721::process_erc721(&mut tables, &clock, erc721_cryptopunks_events, &encoding);
    erc721_metadata::process_erc721_metadata(&mut tables, &clock, erc721_metadata_events, &encoding);
    erc1155::process_erc1155(&mut tables, &clock, erc1155_events, &encoding);
    erc1155_metadata::process_erc1155_metadata(&mut tables, &clock, erc1155_metadata_events, &encoding);
    seaport::process_seaport(&mut tables, &clock, seaport_events, &encoding);
    cryptopunks::process_cryptopunks(&mut tables, &clock, cryptopunks_events, &encoding);

    // ONLY include blocks if events are present
    if !tables.tables.is_empty() {
        let row = tables.create_row("blocks", [("block_num", clock.number.to_string().as_str())]);
        row.set("block_num", clock.number);
        row.set("block_hash", format!("0x{}", clock.id));
        if let Some(timestamp) = &clock.timestamp {
            row.set("timestamp", timestamp.seconds);
        }
    }

    Ok(tables.to_database_changes())
}
