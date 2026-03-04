use common::clickhouse::{common_key, set_log};
use common::{bytes_to_string, Encoding};
use proto::pb::erc1155::v1 as erc1155;
use substreams::pb::substreams::Clock;

use crate::enums::{TokenStandard, TransferType};

pub fn process_erc1155(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: erc1155::Events, encoding: &Encoding) {
    let mut index = 0;

    for trx in events.transactions {
        let tx_hash = trx.hash.clone();

        for log in trx.logs {
            let contract = log.address.clone();
            let ordinal = log.ordinal;

            match log.log {
                Some(erc1155::log::Log::TransferSingle(event)) => {
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

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, None, encoding, row);
                    index += 1;
                }
                Some(erc1155::log::Log::TransferBatch(event)) => {
                    if event.ids.len() != event.values.len() {
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

                        set_log(clock, index, tx_hash.clone(), contract.clone(), ordinal, None, encoding, row);
                        index += 1;
                    });
                }
                Some(erc1155::log::Log::ApprovalForAll(event)) => {
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("erc1155_approvals_for_all", key)
                        .set("owner", bytes_to_string(&event.account, encoding))
                        .set("operator", bytes_to_string(&event.operator, encoding))
                        .set("approved", &event.approved.to_string())
                        .set("token_standard", TokenStandard::ERC1155.to_string());

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, None, encoding, row);
                    index += 1;
                }
                Some(erc1155::log::Log::Uri(_)) => {}
                None => {}
            }
        }
    }
}
