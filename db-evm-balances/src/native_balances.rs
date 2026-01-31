use common::bytes_to_hex;
use proto::pb::evm::balances::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_events(tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for balance in events.balances.iter() {
        let address = bytes_to_hex(&balance.address);
        let row = tables
            .upsert_row("native_balances", &address)
            .set("address", &address)
            .set("balance", &balance.balance);

        set_clock(clock, row);
    }
}
