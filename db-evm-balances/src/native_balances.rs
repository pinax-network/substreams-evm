use common::{bytes_to_string, Encoding};
use proto::pb::evm::balances::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for balance in events.balances.iter() {
        let address = bytes_to_string(&balance.address, encoding);
        let row = tables
            .upsert_row("balances_native", &address)
            .set("address", &address)
            .set("amount", &balance.amount);

        set_clock(clock, row);
    }
}
