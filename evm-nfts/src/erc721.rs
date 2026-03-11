use common::clickhouse::{common_key, set_clock, set_template_log, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::erc721::transfers::v1 as pb;
use substreams::pb::substreams::Clock;

use crate::enums::{TokenStandard, TransferType};

pub fn process_erc721(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: &pb::Events, encoding: &Encoding) {
    let mut row_index = 0;

    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match log.log {
                Some(ref event) if matches!(event, pb::log::Log::Transfer(_)) => {
                    let pb::log::Log::Transfer(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("erc721_transfers", key)
                        .set("token_id", &event.token_id)
                        .set("from", bytes_to_string(&event.from, encoding))
                        .set("to", bytes_to_string(&event.to, encoding))
                        .set("operator", "".to_string())
                        .set("amount", 1)
                        .set("transfer_type", TransferType::Single.to_string())
                        .set("token_standard", TokenStandard::ERC721.to_string());

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::Approval(_)) => {
                    let pb::log::Log::Approval(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("erc721_approvals", key)
                        .set("owner", bytes_to_string(&event.owner, encoding))
                        .set("approved", bytes_to_string(&event.approved, encoding))
                        .set("token_id", &event.token_id);

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::ApprovalForAll(_)) => {
                    let pb::log::Log::ApprovalForAll(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("erc721_approvals_for_all", key)
                        .set("owner", bytes_to_string(&event.owner, encoding))
                        .set("operator", bytes_to_string(&event.operator, encoding))
                        .set("approved", &event.approved.to_string())
                        .set("token_standard", TokenStandard::ERC721.to_string());

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(_) => {}
                None => {}
            }
        }
    }
}
