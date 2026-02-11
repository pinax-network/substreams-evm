mod calls;

use std::collections::HashSet;

use calls::{batch_max_supply, batch_total_supply};
use proto::pb::evm::balances::v1 as balances_pb;
use proto::pb::evm::supply::v1 as supply_pb;

#[substreams::handlers::map]
fn map_events(params: String, erc20_balance_events: balances_pb::Events) -> Result<supply_pb::Events, substreams::errors::Error> {
    let mut events = supply_pb::Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");

    // Collect unique contract addresses from balance events
    let contracts: Vec<&common::Address> = erc20_balance_events
        .balances
        .iter()
        .filter_map(|balance| balance.contract.as_ref())
        .collect::<HashSet<&common::Address>>()
        .into_iter()
        .collect();

    if contracts.is_empty() {
        return Ok(events);
    }

    // Fetch totalSupply for all contracts (with fallback to alternative methods)
    let total_supplies = batch_total_supply(&contracts, chunk_size);

    // Fetch maxSupply for all contracts (with fallback to cap())
    let max_supplies = batch_max_supply(&contracts, chunk_size);

    for contract in &contracts {
        if let Some(total_supply) = total_supplies.get(contract) {
            events.supplies.push(supply_pb::Supply {
                contract: contract.to_vec(),
                total_supply: total_supply.to_string(),
                max_supply: max_supplies.get(contract).map(|v| v.to_string()),
            });
        }
    }

    Ok(events)
}
