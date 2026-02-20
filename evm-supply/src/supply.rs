use common::{bytes_to_string, Encoding};
use proto::pb::erc20::supply::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for supply in events.total_supplies.iter() {
        let contract = bytes_to_string(&supply.contract, encoding);
        let row = tables
            .upsert_row("total_supply", &contract)
            .set("contract", &contract)
            .set("amount", &supply.amount);

        set_clock(clock, row);
    }
}
