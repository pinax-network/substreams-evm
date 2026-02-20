mod calls;

use std::collections::HashSet;

use calls::batch_total_supply;
use proto::pb::erc20::supply::v1 as supply_pb;
use proto::pb::evm::balances::v1 as balances_pb;

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

    for contract in &contracts {
        if let Some(amount) = total_supplies.get(contract) {
            events.total_supplies.push(supply_pb::TotalSupply {
                contract: contract.to_vec(),
                amount: amount.to_string(),
            });
        }
    }

    Ok(events)
}
