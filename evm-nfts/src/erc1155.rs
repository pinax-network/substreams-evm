use common::clickhouse::{common_key, set_log, CallMetadata};
use common::{bytes_to_string, Encoding};
use proto::pb::erc1155::v1 as erc1155;
use substreams::pb::substreams::Clock;

use crate::enums::{TokenStandard, TransferType};

fn call_type_name(value: i32) -> &'static str {
    match erc1155::CallType::try_from(value) {
        Ok(call_type) => call_type.as_str_name(),
        Err(_) => {
            substreams::log::debug!("unexpected erc1155 call_type value: {}", value);
            erc1155::CallType::Unspecified.as_str_name()
        }
    }
}

pub fn process_erc1155(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: &erc1155::Events, encoding: &Encoding) {
    let mut index = 0;

    for trx in &events.transactions {
        let tx_hash = trx.hash.clone();

        for log in &trx.logs {
            let contract = log.address.clone();
            let ordinal = log.ordinal;
            let block_index = Some(log.block_index);
            let call = log.call.as_ref().map(|call| CallMetadata {
                caller: Some(call.caller.as_slice()),
                index: Some(call.index),
                begin_ordinal: Some(call.begin_ordinal),
                end_ordinal: Some(call.end_ordinal),
                address: Some(call.address.as_slice()),
                value: Some(call.value.as_str()),
                gas_consumed: Some(call.gas_consumed),
                gas_limit: Some(call.gas_limit),
                depth: Some(call.depth),
                parent_index: Some(call.parent_index),
                call_type: Some(call_type_name(call.call_type)),
            });

            match log.log {
                Some(ref event) if matches!(event, erc1155::log::Log::TransferSingle(_)) => {
                    let erc1155::log::Log::TransferSingle(event) = event else { unreachable!() };
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

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, erc1155::log::Log::TransferBatch(_)) => {
                    let erc1155::log::Log::TransferBatch(event) = event else { unreachable!() };
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

                        set_log(clock, index, tx_hash.clone(), contract.clone(), ordinal, block_index, call, encoding, row);
                        index += 1;
                    });
                }
                Some(ref event) if matches!(event, erc1155::log::Log::ApprovalForAll(_)) => {
                    let erc1155::log::Log::ApprovalForAll(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("erc1155_approvals_for_all", key)
                        .set("owner", bytes_to_string(&event.account, encoding))
                        .set("operator", bytes_to_string(&event.operator, encoding))
                        .set("approved", &event.approved.to_string())
                        .set("token_standard", TokenStandard::ERC1155.to_string());

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, erc1155::log::Log::Uri(_)) => {}
                Some(_) => {}
                None => {}
            }
        }
    }
}
