use common::{bytes_to_string, Encoding};
use proto::pb::evm::supply::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for supply in events.supplies.iter() {
        let contract = bytes_to_string(&supply.contract, encoding);
        let row = tables
            .create_row("supply", &contract)
            .set("contract", &contract)
            .set("total_supply", &supply.total_supply);

        if let Some(max_supply) = &supply.max_supply {
            row.set("max_supply", max_supply);
        }

        set_clock(clock, row);
    }
}
