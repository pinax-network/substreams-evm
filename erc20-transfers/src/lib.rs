use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc20::transfers::v1 as pb;
use substreams_abis::evm::token::erc20::events;
use substreams_abis::evm::tokens::weth::events as weth_events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_erc20_transfers = 0;
    let mut total_erc20_approvals = 0;
    let mut total_weth_deposits = 0;
    let mut total_weth_withdrawals = 0;

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
    Ok(events)
}
