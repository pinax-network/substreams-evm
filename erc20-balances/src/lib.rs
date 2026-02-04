mod calls;

use std::collections::HashSet;

use calls::batch_balance_of;
use proto::pb::erc20::transfers::v1 as transfers_pb;
use proto::pb::evm::balances::v1 as balances_pb;

#[substreams::handlers::map]
fn map_events(params: String, transfers: transfers_pb::Events) -> Result<balances_pb::Events, substreams::errors::Error> {
    let mut events = balances_pb::Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");

    // Collect unique tokens by owners
    // Include addresses from:
    // - Transfer events: from, to
    // - WETH Deposit events: dst
    // - WETH Withdrawal events: src
    // - Approval events: owner, spender
    // - USDC Mint events: minter, to
    // - USDC Burn events: burner
    // - USDT Issue/Redeem events: owner (from call.caller)
    // - stETH SharesBurnt events: account
    // - stETH TransferShares events: from, to
    // - stETH ExternalSharesBurnt events: owner (from call.caller)
    // - transaction.from for all transactions
    // - log.address for all logs (token contract itself)
    let contracts_by_address = transfers
        .transactions
        .iter()
        .flat_map(|tx| {
            tx.logs.iter().flat_map(|log| {
                let mut addresses: Vec<(&common::Address, &common::Address)> = vec![];
                // Always track transaction.from and log.address (token contract)
                addresses.push((&log.address, &tx.from));
                addresses.push((&log.address, &log.address));

                match &log.log {
                    Some(transfers_pb::log::Log::Transfer(transfer)) => {
                        addresses.push((&log.address, &transfer.from));
                        addresses.push((&log.address, &transfer.to));
                    }
                    Some(transfers_pb::log::Log::Approval(approval)) => {
                        addresses.push((&log.address, &approval.owner));
                        addresses.push((&log.address, &approval.spender));
                    }
                    Some(transfers_pb::log::Log::Deposit(deposit)) => {
                        addresses.push((&log.address, &deposit.dst));
                    }
                    Some(transfers_pb::log::Log::Withdrawal(withdrawal)) => {
                        addresses.push((&log.address, &withdrawal.src));
                    }
                    // USDC events
                    Some(transfers_pb::log::Log::Mint(mint)) => {
                        addresses.push((&log.address, &mint.minter));
                        addresses.push((&log.address, &mint.to));
                    }
                    Some(transfers_pb::log::Log::Burn(burn)) => {
                        addresses.push((&log.address, &burn.burner));
                    }
                    // USDT Issue/Redeem events - owner is from call.caller
                    Some(transfers_pb::log::Log::Issue(issue)) => {
                        addresses.push((&log.address, &issue.owner));
                    }
                    Some(transfers_pb::log::Log::Redeem(redeem)) => {
                        addresses.push((&log.address, &redeem.owner));
                    }
                    // stETH events
                    Some(transfers_pb::log::Log::TokenRebased(_)) => {
                        // TokenRebased doesn't have individual account addresses
                    }
                    Some(transfers_pb::log::Log::SharesBurnt(shares_burnt)) => {
                        addresses.push((&log.address, &shares_burnt.account));
                    }
                    Some(transfers_pb::log::Log::TransferShares(transfer_shares)) => {
                        addresses.push((&log.address, &transfer_shares.from));
                        addresses.push((&log.address, &transfer_shares.to));
                    }
                    Some(transfers_pb::log::Log::ExternalSharesBurnt(external_shares_burnt)) => {
                        // ExternalSharesBurnt uses call.caller for the owner
                        if !external_shares_burnt.owner.is_empty() {
                            addresses.push((&log.address, &external_shares_burnt.owner));
                        }
                    }
                    None => {}
                }
                addresses
            })
        })
        .collect::<HashSet<(&common::Address, &common::Address)>>()
        .into_iter()
        .collect::<Vec<(&common::Address, &common::Address)>>();

    // Fetch RPC calls for Balance Of
    let amounts = batch_balance_of(&contracts_by_address, chunk_size);

    for (contract, address) in &contracts_by_address {
        if let Some(amount) = amounts.get(&(contract, address)) {
            events.balances.push(balances_pb::Balance {
                contract: Some(contract.to_vec()),
                address: address.to_vec(),
                amount: amount.to_string(),
            });
        };
    }
    Ok(events)
}
