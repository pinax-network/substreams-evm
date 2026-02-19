use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc20::transfers::v1 as pb;
use substreams_abis::standard::erc20::events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_erc20_transfers = 0;
    let mut total_erc20_approvals = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        // Use logs_with_calls() to capture logs from internal calls (e.g., multisig executions)
        // Fall back to receipt().logs() for chains without call traces (e.g., Avalanche with DetailLevel: BASE)
        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
            trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
        } else {
            trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
        };

        for (log, call) in logs_with_calls {
            // ERC-20 Transfer event
            if let Some(event) = events::Transfer::match_and_decode(log) {
                total_erc20_transfers += 1;
                let event = pb::log::Log::Transfer(pb::Transfer {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    amount: event.value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ERC-20 Approval event
            if let Some(event) = events::Approval::match_and_decode(log) {
                total_erc20_approvals += 1;
                let event = pb::log::Log::Approval(pb::Approval {
                    owner: event.owner.to_vec(),
                    spender: event.spender.to_vec(),
                    value: event.value.to_string(),
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
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total ERC20 Transfer events: {}", total_erc20_transfers);
    substreams::log::info!("Total ERC20 Approval events: {}", total_erc20_approvals);
    Ok(events)
}
