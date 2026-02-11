mod calls;

use std::collections::HashSet;

use calls::batch_total_supply;
use proto::pb::evm::balance_changes::v1 as balance_changes_pb;
use proto::pb::evm::supply::v1 as supply_pb;

#[substreams::handlers::map]
fn map_events(params: String, balance_changes: balance_changes_pb::BalanceChanges) -> Result<supply_pb::Events, substreams::errors::Error> {
    let mut events = supply_pb::Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");

    // Collect unique contracts from balance changes
    let contracts: HashSet<&common::Address> = balance_changes
        .balance_changes
        .iter()
        .filter_map(|bc| bc.contract.as_ref())
        .collect();

    let contracts_vec: Vec<&common::Address> = contracts.iter().copied().collect();

    // Fetch RPC calls for Total Supply
    let supplies = batch_total_supply(&contracts_vec, chunk_size);

    for contract in &contracts {
        if let Some(supply) = supplies.get(contract) {
            events.supplies.push(supply_pb::Supply {
                contract: contract.to_vec(),
                supply: supply.to_string(),
            });
        }
    }
    Ok(events)
}
