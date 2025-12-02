use common::{bytes_to_string, Encoding};
use proto::pb::evm::balances::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for balance in events.balances.iter() {
        if let Some(contract) = &balance.contract {
            let contract = bytes_to_string(&contract, encoding);
            let account = bytes_to_string(&balance.account, encoding);
            let key = [("contract", contract.to_string()), ("account", account.to_string())];
            let row = tables.create_row("erc20_balances", key);

            set_clock(clock, row);

            row.set("contract", &contract);
            row.set("account", &account);
            row.set("balance", &balance.balance);
        }
    }
}
