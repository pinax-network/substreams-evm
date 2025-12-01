mod calls;

use std::collections::HashSet;

use calls::batch_balance_of;
use proto::pb::evm as pb;

#[substreams::handlers::map]
fn map_events(params: String, transfers: pb::transfers::v1::Events) -> Result<pb::balances::v1::Events, substreams::errors::Error> {
    let mut events = pb::balances::v1::Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");

    // Collect unique tokens by owners
    let contracts_by_owner = transfers
        .transactions
        .iter()
        .flat_map(|tx| tx.logs.iter())
        .filter_map(|log| {
            if let Some(pb::transfers::v1::log::Log::Transfer(transfer)) = &log.log {
                Some((&log.address, &transfer.from))
            } else {
                None
            }
        })
        .collect::<HashSet<(&common::Address, &common::Address)>>()
        .into_iter()
        .collect::<Vec<(&common::Address, &common::Address)>>();

    // Fetch RPC calls for Balance Of
    let balance_ofs = batch_balance_of(&contracts_by_owner, chunk_size);

    for (contract, owner) in &contracts_by_owner {
        if let Some(balance) = balance_ofs.get(&(contract, owner)) {
            events.balances.push(pb::balances::v1::Balance {
                contract: Some(contract.to_vec()),
                account: owner.to_vec(),
                balance: balance.to_string(),
            });
        };
    }
    Ok(events)
}
