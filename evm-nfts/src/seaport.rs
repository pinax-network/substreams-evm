use common::clickhouse::{common_key, set_log};
use common::{bytes_to_string, Encoding};
use proto::pb::evm::seaport;
use substreams::pb::substreams::Clock;

use crate::to_json::{considerations_to_json, offers_to_json};

pub fn process_seaport(tables: &mut substreams_database_change::tables::Tables, clock: &Clock, events: seaport::v1::Events, encoding: &Encoding) {
    let mut index = 0;

    for event in events.order_fulfilled {
        let key = common_key(clock, index);
        let row = tables
            .create_row("seaport_order_fulfilled", key)
            .set("order_hash", common::bytes_to_hex(&event.order_hash))
            .set("offerer", bytes_to_string(&event.offerer, encoding))
            .set("zone", bytes_to_string(&event.zone, encoding))
            .set("recipient", bytes_to_string(&event.recipient, encoding))
            .set("offer_raw", offers_to_json(event.offer, encoding).to_string())
            .set("consideration_raw", considerations_to_json(event.consideration, encoding).to_string());

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, encoding, row);
        index += 1;
    }

    for event in events.orders_matched {
        let key = common_key(clock, index);
        let order_hashes_raw = event.order_hashes.iter().map(|h| common::bytes_to_hex(h)).collect::<Vec<String>>().join(",");
        let row = tables.create_row("seaport_orders_matched", key).set("order_hashes_raw", order_hashes_raw);

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, encoding, row);
        index += 1;
    }

    for event in events.order_cancelled {
        let key = common_key(clock, index);
        let row = tables
            .create_row("seaport_order_cancelled", key)
            .set("order_hash", common::bytes_to_hex(&event.order_hash))
            .set("offerer", bytes_to_string(&event.offerer, encoding))
            .set("zone", bytes_to_string(&event.zone, encoding));

        set_log(clock, index, event.tx_hash, event.contract, event.ordinal, event.caller, encoding, row);
        index += 1;
    }
}
