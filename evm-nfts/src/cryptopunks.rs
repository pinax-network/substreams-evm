use common::clickhouse::{common_key, set_log, CallMetadata};
use common::{bytes_to_string, Encoding};
use proto::pb::erc721::tokens::v1 as pb;
use substreams::pb::substreams::Clock;

fn call_type_name(value: i32) -> &'static str {
    match pb::CallType::try_from(value) {
        Ok(call_type) => call_type.as_str_name(),
        Err(_) => {
            substreams::log::debug!("unexpected cryptopunks call_type value: {}", value);
            pb::CallType::Unspecified.as_str_name()
        }
    }
}

pub fn process_cryptopunks(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: &pb::Events, encoding: &Encoding) {
    let mut index = 0;

    for tx in &events.transactions {
        let tx_hash = tx.hash.clone();

        for log in &tx.logs {
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
                Some(ref event) if matches!(event, pb::log::Log::Assign(_)) => {
                    let pb::log::Log::Assign(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("punk_assigns", key)
                        .set("to", bytes_to_string(&event.to, encoding))
                        .set("punk_index", &event.punk_index);

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkTransfer(_)) => {
                    let pb::log::Log::PunkTransfer(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("punk_transfers", key)
                        .set("from", bytes_to_string(&event.from, encoding))
                        .set("to", bytes_to_string(&event.to, encoding))
                        .set("punk_index", &event.punk_index);

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkBought(_)) => {
                    let pb::log::Log::PunkBought(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("punk_bought", key)
                        .set("from", bytes_to_string(&event.from_address, encoding))
                        .set("to", bytes_to_string(&event.to_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("value_is_null", &event.value.is_none().to_string())
                        .set("value", &event.value.clone().unwrap_or_default());

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkBidEntered(_)) => {
                    let pb::log::Log::PunkBidEntered(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("punk_bid_entered", key)
                        .set("from", bytes_to_string(&event.from_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("value", &event.value);

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkBidWithdrawn(_)) => {
                    let pb::log::Log::PunkBidWithdrawn(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("punk_bid_withdrawn", key)
                        .set("from", bytes_to_string(&event.from_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("value", &event.value);

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkNoLongerForSale(_)) => {
                    let pb::log::Log::PunkNoLongerForSale(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables.create_row("punk_no_longer_for_sale", key).set("punk_index", &event.punk_index);

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkOffered(_)) => {
                    let pb::log::Log::PunkOffered(event) = event else { unreachable!() };
                    let key = common_key(clock, index);
                    let row = tables
                        .create_row("punk_offered", key)
                        .set("to", bytes_to_string(&event.to_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("min_value", &event.min_value);

                    set_log(clock, index, tx_hash.clone(), contract, ordinal, block_index, call, encoding, row);
                    index += 1;
                }
                Some(_) => {}
                None => {}
            }
        }
    }
}
