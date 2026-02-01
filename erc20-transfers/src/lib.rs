use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc20::transfers::v1 as pb;
use substreams_abis::evm::token::erc20::events;
use substreams_abis::evm::tokens::usdt::events as usdt_events;
use substreams_abis::evm::tokens::usdt::functions as usdt_functions;
use substreams_abis::evm::tokens::weth::events as weth_events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::rpc::RpcBatch;
use substreams_ethereum::Event;

/// Fetches the owner address of a USDT contract via RPC call.
/// Returns None if the RPC call fails or the response cannot be decoded.
fn get_usdt_owner(contract_address: &[u8]) -> Option<Vec<u8>> {
    RpcBatch::new()
        .add(usdt_functions::Owner {}, contract_address.to_vec())
        .execute()
        .ok()?
        .responses
        .into_iter()
        .next()
        .and_then(|r| {
            if r.failed {
                substreams::log::debug!("RPC call to get USDT owner failed");
                None
            } else {
                usdt_functions::Owner::output(&r.raw).ok()
            }
        })
}

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_erc20_transfers = 0;
    let mut total_erc20_approvals = 0;
    let mut total_weth_deposits = 0;
    let mut total_weth_withdrawals = 0;
    let mut total_usdt_issues = 0;
    let mut total_usdt_redeems = 0;
    let mut total_usdt_destroyed_black_funds = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        for log_view in trx.receipt().logs() {
            let log = log_view.log;
            // ERC-20 Transfer event
            if let Some(event) = events::Transfer::match_and_decode(log) {
                total_erc20_transfers += 1;
                let event = pb::log::Log::Transfer(pb::Transfer {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    amount: event.value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // ERC-20 Approval event
            if let Some(event) = events::Approval::match_and_decode(log) {
                total_erc20_approvals += 1;
                let event = pb::log::Log::Approval(pb::Approval {
                    owner: event.owner.to_vec(),
                    spender: event.spender.to_vec(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // WETH Deposit/Withdraw event
            if let Some(event) = weth_events::Deposit::match_and_decode(log) {
                total_weth_deposits += 1;
                let event = pb::log::Log::Deposit(pb::Deposit {
                    dst: event.dst.to_vec(),
                    wad: event.wad.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }
            if let Some(event) = weth_events::Withdrawal::match_and_decode(log) {
                total_weth_withdrawals += 1;
                let event = pb::log::Log::Withdrawal(pb::Withdrawal {
                    src: event.src.to_vec(),
                    wad: event.wad.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // USDT Issue event (mints to owner - requires RPC call to get owner address)
            if let Some(issue_event) = usdt_events::Issue::match_and_decode(log) {
                if let Some(owner_address) = get_usdt_owner(&log.address) {
                    total_usdt_issues += 1;
                    let event = pb::log::Log::Issue(pb::Issue {
                        owner: owner_address,
                        amount: issue_event.amount.to_string(),
                    });
                    transaction.logs.push(pb::Log::create_log(log, event));
                }
            }

            // USDT Redeem event (burns from owner - requires RPC call to get owner address)
            if let Some(redeem_event) = usdt_events::Redeem::match_and_decode(log) {
                if let Some(owner_address) = get_usdt_owner(&log.address) {
                    total_usdt_redeems += 1;
                    let event = pb::log::Log::Redeem(pb::Redeem {
                        owner: owner_address,
                        amount: redeem_event.amount.to_string(),
                    });
                    transaction.logs.push(pb::Log::create_log(log, event));
                }
            }

            // USDT DestroyedBlackFunds event (burns from blacklisted address)
            if let Some(event) = usdt_events::DestroyedBlackFunds::match_and_decode(log) {
                total_usdt_destroyed_black_funds += 1;
                let event = pb::log::Log::DestroyedBlackFunds(pb::DestroyedBlackFunds {
                    black_listed_user: event.black_listed_user.to_vec(),
                    balance: event.balance.to_string(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }
        }
        // Only include transactions with logs
        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }
    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total ERC20 Transfer events: {}", total_erc20_transfers);
    substreams::log::info!("Total ERC20 Approval events: {}", total_erc20_approvals);
    substreams::log::info!("Total WETH Deposit events: {}", total_weth_deposits);
    substreams::log::info!("Total WETH Withdrawal events: {}", total_weth_withdrawals);
    substreams::log::info!("Total USDT Issue events: {}", total_usdt_issues);
    substreams::log::info!("Total USDT Redeem events: {}", total_usdt_redeems);
    substreams::log::info!("Total USDT DestroyedBlackFunds events: {}", total_usdt_destroyed_black_funds);
    Ok(events)
}
