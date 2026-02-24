use common::clickhouse::{common_key, set_clock, set_log};
use common::{bytes_to_string, Encoding};
use proto::pb::evm::erc1155::v1 as erc1155;
use substreams::pb::substreams::Clock;

use crate::enums::{TokenStandard, TransferType};

pub fn process_erc1155(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: erc1155::Events, encoding: &Encoding) {
    let mut index = 0;

    for event in events.transfers_single {
        let key = common_key(clock, index);
        let row = tables
            .create_row("erc1155_transfers", key)
            .set("operator", bytes_to_string(&event.operator, encoding))
            .set("token_id", &event.id)
            .set("from", bytes_to_string(&event.from, encoding))
            .set("to", bytes_to_string(&event.to, encoding))
            .set("amount", &event.value)
            .set("transfer_type", TransferType::Single.to_string())
            .set("token_standard", TokenStandard::ERC1155.to_string());

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, encoding, row);
        index += 1;
    }

    for event in events.transfers_batch {
        if event.ids.len() != event.values.len() {
            substreams::log::info!(
                "Invalid ERC1155 TransferBatch event: mismatch between ids length ({}) and values length ({}) in trx {}",
                event.ids.len(),
                event.values.len(),
                common::bytes_to_hex(&event.tx_hash)
            );
            continue;
        }

        event.ids.iter().zip(event.values.iter()).for_each(|(id, value)| {
            let key = common_key(clock, index);
            let row = tables
                .create_row("erc1155_transfers", key)
                .set("operator", bytes_to_string(&event.operator, encoding))
                .set("from", bytes_to_string(&event.from, encoding))
                .set("to", bytes_to_string(&event.to, encoding))
                .set("token_id", id)
                .set("amount", value)
                .set("transfer_type", TransferType::Batch.to_string())
                .set("token_standard", TokenStandard::ERC1155.to_string());

            set_log(
                clock,
                index,
                event.tx_hash.clone(),
                event.contract.clone(),
                event.ordinal,
                event.caller.clone(),
                encoding,
                row,
            );
            index += 1;
        });
    }

    for event in events.approvals_for_all {
        let key = common_key(clock, index);
        let row = tables
            .create_row("erc1155_approvals_for_all", key)
            .set("owner", bytes_to_string(&event.account, encoding))
            .set("operator", bytes_to_string(&event.operator, encoding))
            .set("approved", &event.approved.to_string())
            .set("token_standard", TokenStandard::ERC1155.to_string());

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, encoding, row);
        index += 1;
    }

    for event in events.uris {
        let key = [("contract", bytes_to_string(&event.contract, encoding)), ("token_id", event.id.to_string())];
        if event.value.is_empty() {
            continue;
        }

        let row = tables
            .create_row("erc1155_metadata_by_token", key)
            .set("contract", bytes_to_string(&event.contract, encoding))
            .set("token_id", &event.id)
            .set("uri", event.value);

        set_clock(clock, row);
        index += 1;
    }
}
