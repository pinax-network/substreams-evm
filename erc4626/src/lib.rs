use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc4626::v1 as pb;
use substreams_abis::standard::erc4626::events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_deposits = 0;
    let mut total_withdraws = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        // Use logs_with_calls() to capture logs from internal calls (e.g. router/aggregator deposits)
        // Fall back to receipt().logs() for chains without call traces (DetailLevel: BASE)
        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
            trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
        } else {
            trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
        };

        for (log, call) in logs_with_calls {
            // ERC-4626 Deposit(sender, owner, assets, shares)
            if let Some(event) = events::Deposit::match_and_decode(log) {
                total_deposits += 1;
                let event = pb::log::Log::Deposit(pb::Deposit {
                    sender: event.sender.to_vec(),
                    owner: event.owner.to_vec(),
                    assets: event.assets.to_string(),
                    shares: event.shares.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ERC-4626 Withdraw(sender, receiver, owner, assets, shares)
            if let Some(event) = events::Withdraw::match_and_decode(log) {
                total_withdraws += 1;
                let event = pb::log::Log::Withdraw(pb::Withdraw {
                    sender: event.sender.to_vec(),
                    receiver: event.receiver.to_vec(),
                    owner: event.owner.to_vec(),
                    assets: event.assets.to_string(),
                    shares: event.shares.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }
        }

        // Only include transactions with logs
        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total ERC-4626 Deposit events: {}", total_deposits);
    substreams::log::info!("Total ERC-4626 Withdraw events: {}", total_withdraws);
    Ok(events)
}
