mod calls;

use std::collections::HashSet;

use calls::batch_balance_of;
use proto::pb::erc20::tokens::v1 as tokens_pb;
use proto::pb::erc20::transfers::v1 as transfers_pb;
use proto::pb::evm::balances::v1 as balances_pb;

#[substreams::handlers::map]
fn map_events(params: String, transfers: transfers_pb::Events, tokens: tokens_pb::Events) -> Result<balances_pb::Events, substreams::errors::Error> {
    let mut events = balances_pb::Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");

    // Collect unique tokens by owners
    // Include addresses from:
    // - Transfer events: from, to
    // - WETH Deposit events: dst
    // - WETH Withdrawal events: src
    // - Approval events: owner, spender
    // - transaction.from for all transactions
    // - log.address for all logs (token contract itself)
    //
    // From erc20-tokens:
    // - OwnershipTransferred events: previous_owner, new_owner (shared: USDC, USDT, WBTC)
    // - USDC Mint events: minter, to
    // - USDC Burn events: burner
    // - USDT Issue events: caller (owner)
    // - USDT Redeem events: caller (owner)
    // - USDT DestroyedBlackFunds events: black_listed_user
    // - USDT BlockPlaced events: user (v0.8.4)
    // - USDT BlockReleased events: user (v0.8.4)
    // - USDT Mint events: destination (v0.8.4)
    // - USDT DestroyedBlockedFunds events: blocked_user (v0.8.4)
    // - USDT LogSwapin events: account (swap_asset)
    // - USDT LogSwapout events: account (swap_asset)
    // - WBTC Mint events: to
    // - WBTC Burn events: burner
    // - SAI Mint events: guy
    // - SAI Burn events: guy
    // - stETH Submitted events: sender
    // - stETH TransferShares events: from, to
    // - stETH SharesBurnt events: account
    // - stETH ExternalSharesMinted events: recipient
    // - stETH ExternalSharesBurnt events: owner
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
                    None => {}
                }
                addresses
            })
        })
        .chain(
            // Add addresses from erc20-tokens events
            tokens.transactions.iter().flat_map(|tx| {
                tx.logs.iter().flat_map(|log| {
                    let mut addresses: Vec<(&common::Address, &common::Address)> = vec![];
                    // Always track transaction.from and log.address (token contract)
                    addresses.push((&log.address, &tx.from));
                    addresses.push((&log.address, &log.address));

                    match &log.log {
                        // WETH events
                        Some(tokens_pb::log::Log::WethDeposit(deposit)) => {
                            addresses.push((&log.address, &deposit.dst));
                        }
                        Some(tokens_pb::log::Log::WethWithdrawal(withdrawal)) => {
                            addresses.push((&log.address, &withdrawal.src));
                        }
                        // USDC events
                        Some(tokens_pb::log::Log::UsdcMint(mint)) => {
                            addresses.push((&log.address, &mint.minter));
                            addresses.push((&log.address, &mint.to));
                        }
                        Some(tokens_pb::log::Log::UsdcBurn(burn)) => {
                            addresses.push((&log.address, &burn.burner));
                        }
                        Some(tokens_pb::log::Log::UsdcBlacklisted(blacklisted)) => {
                            addresses.push((&log.address, &blacklisted.account));
                        }
                        Some(tokens_pb::log::Log::UsdcUnBlacklisted(un_blacklisted)) => {
                            addresses.push((&log.address, &un_blacklisted.account));
                        }
                        // USDT events
                        Some(tokens_pb::log::Log::UsdtDestroyedBlackFunds(destroyed)) => {
                            addresses.push((&log.address, &destroyed.black_listed_user));
                        }
                        Some(tokens_pb::log::Log::UsdtAddedBlackList(added)) => {
                            addresses.push((&log.address, &added.user));
                        }
                        Some(tokens_pb::log::Log::UsdtRemovedBlackList(removed)) => {
                            addresses.push((&log.address, &removed.user));
                        }
                        // USDT Issue/Redeem: modifies balances[owner], owner from call.caller
                        Some(tokens_pb::log::Log::UsdtIssue(issue)) => {
                            addresses.push((&log.address, &issue.owner));
                        }
                        Some(tokens_pb::log::Log::UsdtRedeem(redeem)) => {
                            addresses.push((&log.address, &redeem.owner));
                        }
                        // USDT v0.8.4 events
                        Some(tokens_pb::log::Log::UsdtBlockPlaced(block_placed)) => {
                            addresses.push((&log.address, &block_placed.user));
                        }
                        Some(tokens_pb::log::Log::UsdtBlockReleased(block_released)) => {
                            addresses.push((&log.address, &block_released.user));
                        }
                        Some(tokens_pb::log::Log::UsdtMint(mint)) => {
                            addresses.push((&log.address, &mint.destination));
                        }
                        Some(tokens_pb::log::Log::UsdtDestroyedBlockedFunds(destroyed)) => {
                            addresses.push((&log.address, &destroyed.blocked_user));
                        }
                        // USDT swap_asset events
                        Some(tokens_pb::log::Log::UsdtLogSwapin(swapin)) => {
                            addresses.push((&log.address, &swapin.account));
                        }
                        Some(tokens_pb::log::Log::UsdtLogSwapout(swapout)) => {
                            addresses.push((&log.address, &swapout.account));
                        }
                        // OwnershipTransferred (shared: USDC, USDT, WBTC)
                        Some(tokens_pb::log::Log::OwnershipTransferred(ownership)) => {
                            addresses.push((&log.address, &ownership.previous_owner));
                            addresses.push((&log.address, &ownership.new_owner));
                        }
                        // WBTC events
                        Some(tokens_pb::log::Log::WbtcMint(mint)) => {
                            addresses.push((&log.address, &mint.to));
                        }
                        Some(tokens_pb::log::Log::WbtcBurn(burn)) => {
                            addresses.push((&log.address, &burn.burner));
                        }
                        // SAI events
                        Some(tokens_pb::log::Log::SaiMint(mint)) => {
                            addresses.push((&log.address, &mint.guy));
                        }
                        Some(tokens_pb::log::Log::SaiBurn(burn)) => {
                            addresses.push((&log.address, &burn.guy));
                        }
                        // stETH events
                        Some(tokens_pb::log::Log::StethSubmitted(submitted)) => {
                            addresses.push((&log.address, &submitted.sender));
                        }
                        Some(tokens_pb::log::Log::StethTransferShares(transfer_shares)) => {
                            addresses.push((&log.address, &transfer_shares.from));
                            addresses.push((&log.address, &transfer_shares.to));
                        }
                        Some(tokens_pb::log::Log::StethSharesBurnt(shares_burnt)) => {
                            addresses.push((&log.address, &shares_burnt.account));
                        }
                        Some(tokens_pb::log::Log::StethExternalSharesMinted(external_shares_minted)) => {
                            addresses.push((&log.address, &external_shares_minted.recipient));
                        }
                        Some(tokens_pb::log::Log::StethExternalSharesBurnt(external_shares_burnt)) => {
                            addresses.push((&log.address, &external_shares_burnt.owner));
                        }
                        _ => {}
                    }
                    addresses
                })
            }),
        )
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
