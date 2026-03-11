use common::clickhouse::{common_key, set_clock, set_template_log, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::erc721::tokens::v1 as pb;
use substreams::pb::substreams::Clock;

pub fn process_cryptopunks(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: &pb::Events, encoding: &Encoding) {
    let mut row_index = 0;

    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match log.log {
                Some(ref event) if matches!(event, pb::log::Log::Assign(_)) => {
                    let pb::log::Log::Assign(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("punk_assigns", key)
                        .set("to", bytes_to_string(&event.to, encoding))
                        .set("punk_index", &event.punk_index);

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkTransfer(_)) => {
                    let pb::log::Log::PunkTransfer(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("punk_transfers", key)
                        .set("from", bytes_to_string(&event.from, encoding))
                        .set("to", bytes_to_string(&event.to, encoding))
                        .set("punk_index", &event.punk_index);

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkBought(_)) => {
                    let pb::log::Log::PunkBought(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("punk_bought", key)
                        .set("from", bytes_to_string(&event.from_address, encoding))
                        .set("to", bytes_to_string(&event.to_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("value_is_null", &event.value.is_none().to_string())
                        .set("value", &event.value.clone().unwrap_or_default());

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkBidEntered(_)) => {
                    let pb::log::Log::PunkBidEntered(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("punk_bid_entered", key)
                        .set("from", bytes_to_string(&event.from_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("value", &event.value);

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkBidWithdrawn(_)) => {
                    let pb::log::Log::PunkBidWithdrawn(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("punk_bid_withdrawn", key)
                        .set("from", bytes_to_string(&event.from_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("value", &event.value);

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkNoLongerForSale(_)) => {
                    let pb::log::Log::PunkNoLongerForSale(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables.create_row("punk_no_longer_for_sale", key).set("punk_index", &event.punk_index);

                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row_index += 1;
                }
                Some(ref event) if matches!(event, pb::log::Log::PunkOffered(_)) => {
                    let pb::log::Log::PunkOffered(event) = event else { unreachable!() };
                    let key = common_key(clock, row_index);
                    let row = tables
                        .create_row("punk_offered", key)
                        .set("to", bytes_to_string(&event.to_address, encoding))
                        .set("punk_index", &event.punk_index)
                        .set("min_value", &event.min_value);

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
