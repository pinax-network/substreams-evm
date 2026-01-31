use common::bytes_to_hex;
use proto::pb::evm::balances::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_events(tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for balance in events.balances.iter() {
        if let Some(contract_bytes) = &balance.contract {
            let contract = bytes_to_hex(contract_bytes);
            let address = bytes_to_hex(&balance.address);
            let key = [("contract", contract.to_string()), ("address", address.to_string())];
            let row = tables
                .upsert_row("erc20_balances", key)
                .set("contract", &contract)
                .set("address", &address)
                .set("balance", &balance.balance);

            set_clock(clock, row);
        }
    }
}
