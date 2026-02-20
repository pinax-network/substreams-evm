use common::{bytes_to_string, Encoding};
use proto::pb::dca_dot_fun::v1 as dca;
use substreams::pb::substreams::Clock;
use substreams::store::{StoreGet, StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    transactions::set_template_tx,
};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &dca::Events, store: &StoreGetProto<dca::StoreOrder>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(dca::log::Log::FillOrder(event)) => {
                    let key = log_key(clock, tx_index, log_index);
                    let row = tables.create_row("dca_dot_fun_fill_order", key);
                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row.set("order_id", &event.order_id);
                    row.set("caller", bytes_to_string(&event.caller, encoding));
                    row.set("recipient", bytes_to_string(&event.recipient, encoding));
                    row.set("fill_amount", &event.fill_amount);
                    row.set("amount_of_token_out", &event.amount_of_token_out);
                    row.set("protocol_fee", &event.protocol_fee);
                    row.set("token_in_price", &event.token_in_price);
                    row.set("token_out_price", &event.token_out_price);
                    row.set("scaling_factor", &event.scaling_factor);

                    // Enrich with token addresses from store
                    if let Some(order) = store.get_first(&event.order_id) {
                        row.set("token_in", bytes_to_string(&order.token_in, encoding));
                        row.set("token_out", bytes_to_string(&order.token_out, encoding));
                    }
                }
                Some(dca::log::Log::CreateOrder(event)) => {
                    let key = log_key(clock, tx_index, log_index);
                    let row = tables.create_row("dca_dot_fun_create_order", key);
                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row.set("order_id", &event.order_id);
                    row.set("creator", bytes_to_string(&event.creator, encoding));
                    row.set("recipient", bytes_to_string(&event.recipient, encoding));
                    row.set("token_in", bytes_to_string(&event.token_in, encoding));
                    row.set("token_out", bytes_to_string(&event.token_out, encoding));
                    row.set("spend_amount", &event.spend_amount);
                    row.set("repeats", &event.repeats);
                    row.set("slippage", &event.slippage);
                    row.set("freq_interval", &event.freq_interval);
                    row.set("scaling_interval", &event.scaling_interval);
                    row.set("protocol_fee", &event.protocol_fee);
                    row.set("vault", bytes_to_string(&event.vault, encoding));
                    row.set("stake_asset_in", event.stake_asset_in);
                    row.set("stake_asset_out", event.stake_asset_out);
                }
                Some(dca::log::Log::CancelOrder(event)) => {
                    let key = log_key(clock, tx_index, log_index);
                    let row = tables.create_row("dca_dot_fun_cancel_order", key);
                    set_clock(clock, row);
                    set_template_tx(encoding, tx, tx_index, row);
                    set_template_log(encoding, log, log_index, row);
                    row.set("order_id", &event.order_id);
                    row.set("vault", bytes_to_string(&event.vault, encoding));
                }
                _ => {}
            }
        }
    }
}
