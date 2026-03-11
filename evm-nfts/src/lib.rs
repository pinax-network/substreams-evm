mod cryptopunks;
mod enums;
mod erc1155;
mod erc721;
mod seaport;
mod to_json;

use common::clickhouse::set_clock;
use proto::pb::{erc1155 as erc1155_pb, erc721 as erc721_pb, evm::seaport as seaport_pb};
use substreams::pb::substreams::Clock;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;

#[substreams::handlers::map]
pub fn db_out(
    params: String,
    clock: Clock,
    erc721_events: erc721_pb::transfers::v1::Events,
    erc721_token_events: erc721_pb::tokens::v1::Events,
    erc1155_events: erc1155_pb::v1::Events,
    seaport_events: seaport_pb::v1::Events,
) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut tables = Tables::new();

    // Handle support both EVM & TVM address encoding
    let encoding = common::handle_encoding_param(&params);

    // Process packages
    erc721::process_erc721(&mut tables, &clock, &erc721_events, &encoding);
    cryptopunks::process_cryptopunks(&mut tables, &clock, &erc721_token_events, &encoding);
    erc1155::process_erc1155(&mut tables, &clock, &erc1155_events, &encoding);
    seaport::process_seaport(&mut tables, &clock, &seaport_events, &encoding);

    // ONLY include blocks if events are present
    if !tables.tables.is_empty() {
        set_clock(&clock, tables.create_row("blocks", [("block_num", clock.number.to_string())]));
    }

    Ok(tables.to_database_changes())
}
