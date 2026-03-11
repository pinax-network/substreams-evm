use common::clickhouse::{common_key, set_clock, set_template_log, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::erc1155::v1 as erc1155;
use substreams::pb::substreams::Clock;

use crate::enums::{TokenStandard, TransferType};

pub fn process_erc1155(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: &erc1155::Events, encoding: &Encoding) {
    let mut row_index = 0;

    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match log.log {
                Some(ref event) if matches!(event, erc1155::log::Log::TransferSingle(_)) => {
                    let erc1155::log::Log::TransferSingle(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("erc1155_transfers", key)
                        .set("operator", bytes_to_string(&event.operator, encoding))
                        .set("token_id", &event.id)
                        .set("from", bytes_to_string(&event.from, encoding))
                        .set("to", bytes_to_string(&event.to, encoding))
                        .set("amount", &event.value)
                        .set("transfer_type", TransferType::Single.to_string())
                        .set("token_standard", TokenStandard::ERC1155.to_string());

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, erc1155::log::Log::TransferBatch(_)) => {
                    let erc1155::log::Log::TransferBatch(event) = event else { unreachable!() };
                    if event.ids.len() != event.values.len() {
                        continue;
                    }

                    event.ids.iter().zip(event.values.iter()).for_each(|(id, value)| {
                        let key = common_key(clock, row_index);
                        let row = tables
                            .create_row("erc1155_transfers", key)
                            .set("operator", bytes_to_string(&event.operator, encoding))
                            .set("from", bytes_to_string(&event.from, encoding))
                            .set("to", bytes_to_string(&event.to, encoding))
                            .set("token_id", id)
                            .set("amount", value)
                            .set("transfer_type", TransferType::Batch.to_string())
                            .set("token_standard", TokenStandard::ERC1155.to_string());

                        set_clock(clock, row);
                        set_template_tx(encoding, tx, tx_index, row);
                        set_template_log(encoding, log, log_index, row);
                        row_index += 1;
                    });
                }
                Some(ref event) if matches!(event, erc1155::log::Log::ApprovalForAll(_)) => {
                    let erc1155::log::Log::ApprovalForAll(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("erc1155_approvals_for_all", key)
                        .set("owner", bytes_to_string(&event.account, encoding))
                        .set("operator", bytes_to_string(&event.operator, encoding))
                        .set("approved", &event.approved.to_string())
                        .set("token_standard", TokenStandard::ERC1155.to_string());

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, erc1155::log::Log::Uri(_)) => {}
                Some(_) => {}
                None => {}
            }
        }
    }
}
