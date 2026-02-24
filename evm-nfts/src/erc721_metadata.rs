use common::clickhouse::set_clock;
use common::{bytes_to_string, Encoding};
use proto::pb::evm::erc721;
use substreams::pb::substreams::Clock;

pub fn process_erc721_metadata(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: erc721::metadata::v1::Events, encoding: &Encoding) {
    for event in events.metadata_by_tokens {
        let key = [("contract", bytes_to_string(&event.contract, encoding)), ("token_id", event.token_id.to_string())];
        if event.uri.is_none() {
            continue;
        }

        let row = tables
            .create_row("erc721_metadata_by_token", key)
            .set("contract", bytes_to_string(&event.contract, encoding))
            .set("token_id", &event.token_id)
            .set("uri", event.uri());

        set_clock(clock, row);
    }

    for event in events.metadata_by_contracts.iter() {
        let key = [("contract", bytes_to_string(&event.contract, encoding))];
        if event.name.is_none() && event.symbol.is_none() && event.base_uri.is_none() {
            continue;
        }

        let row = tables
            .create_row("erc721_metadata_by_contract", key)
            .set("contract", bytes_to_string(&event.contract, encoding))
            .set("name", event.name())
            .set("symbol", event.symbol());

        set_clock(clock, row);
    }

    for event in events.metadata_by_contracts.iter() {
        let key = [("contract", bytes_to_string(&event.contract, encoding))];
        if event.total_supply.is_none() {
            continue;
        }

        let row = tables
            .create_row("erc721_total_supply", key)
            .set("contract", bytes_to_string(&event.contract, encoding))
            .set("total_supply", event.total_supply());

        set_clock(clock, row);
    }

    for event in events.metadata_by_contracts {
        let key = [("contract", bytes_to_string(&event.contract, encoding))];
        if event.base_uri().is_empty() {
            continue;
        }

        let row = tables
            .create_row("erc721_base_uri", key)
            .set("contract", bytes_to_string(&event.contract, encoding))
            .set("base_uri", event.base_uri());

        set_clock(clock, row);
    }
}
