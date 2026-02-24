use common::clickhouse::set_clock;
use common::{bytes_to_string, Encoding};
use proto::pb::evm::erc1155;
use substreams::pb::substreams::Clock;

pub fn process_erc1155_metadata(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: erc1155::metadata::v1::Events, encoding: &Encoding) {
    for event in events.metadata_by_tokens {
        let key = [("contract", bytes_to_string(&event.contract, encoding)), ("token_id", event.token_id.to_string())];
        let row = tables
            .create_row("erc1155_metadata_by_token", key)
            .set("contract", bytes_to_string(&event.contract, encoding))
            .set("token_id", &event.token_id)
            .set("uri", event.uri);

        set_clock(clock, row);
    }

    for event in events.metadata_by_contracts.iter() {
        let key = [("contract", bytes_to_string(&event.contract, encoding))];
        if event.name.is_none() && event.symbol.is_none() {
            continue;
        }

        let row = tables
            .create_row("erc1155_metadata_by_contract", key)
            .set("contract", bytes_to_string(&event.contract, encoding))
            .set("name", event.name())
            .set("symbol", event.symbol());

        set_clock(clock, row);
    }
}
