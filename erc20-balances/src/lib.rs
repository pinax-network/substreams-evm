mod calls;

use std::collections::HashSet;

use calls::batch_balance_of;
use proto::pb::erc20::transfers::v1 as transfers_pb;
use proto::pb::evm::balances::v1 as balances_pb;

#[substreams::handlers::map]
fn map_events(params: String, transfers: transfers_pb::Events) -> Result<balances_pb::Events, substreams::errors::Error> {
    let mut events = balances_pb::Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");

    // Collect unique tokens by owners (both sender and recipient)
    let contracts_by_address = transfers
        .transactions
        .iter()
        .flat_map(|tx| tx.logs.iter())
        .flat_map(|log| {
            if let Some(transfers_pb::log::Log::Transfer(transfer)) = &log.log {
                vec![(&log.address, &transfer.from), (&log.address, &transfer.to)]
            } else {
                vec![]
            }
        })
        .collect::<HashSet<(&common::Address, &common::Address)>>()
        .into_iter()
        .collect::<Vec<(&common::Address, &common::Address)>>();

    // Fetch RPC calls for Balance Of
    let balance_ofs = batch_balance_of(&contracts_by_address, chunk_size);

    for (contract, address) in &contracts_by_address {
        if let Some(balance) = balance_ofs.get(&(contract, address)) {
            events.balances.push(balances_pb::Balance {
                contract: Some(contract.to_vec()),
                address: address.to_vec(),
                balance: balance.to_string(),
            });
        };
    }
    Ok(events)
}
