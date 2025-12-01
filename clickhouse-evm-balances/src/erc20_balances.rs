use common::{bytes_to_string, Encoding};
use proto::pb::evm::balances::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{clock_key, set_clock};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for balance in events.balances.iter() {
        if let Some(contract) = &balance.contract {
            let row = tables.create_row("erc20_balances", clock_key(clock));

            set_clock(clock, row);

            row.set("contract", bytes_to_string(&contract, encoding));
            row.set("account", bytes_to_string(&balance.account, encoding));
            row.set("balance", &balance.balance);
        }
    }
}
