use proto::pb::balancer::v1 as pb;
use substreams_abis::evm::balancer::weightedpool as balancer;
use substreams_ethereum::pb::eth::v2::{Block, Log};
use substreams_ethereum::Event;

fn create_log(log: &Log, event: pb::log::Log) -> pb::Log {
    pb::Log {
        address: log.address.to_vec(),
        ordinal: log.ordinal,
        topics: log.topics.iter().map(|t| t.to_vec()).collect(),
        data: log.data.to_vec(),
        log: Some(event),
    }
}

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_approvals = 0;
    let mut total_transfers = 0;
    let mut total_swap_fee_percentage_changed = 0;
    let mut total_paused_state_changed = 0;
    let mut total_recovery_mode_state_changed = 0;
    let mut total_protocol_fee_percentage_cache_updated = 0;

    for trx in block.transactions() {
        let gas_price = trx.clone().gas_price.unwrap_or_default().with_decimal(0).to_string();
        let value = trx.clone().value.unwrap_or_default().with_decimal(0);
        let to = if trx.to.is_empty() { None } else { Some(trx.to.to_vec()) };
        let mut transaction = pb::Transaction {
            from: trx.from.to_vec(),
            to,
            hash: trx.hash.to_vec(),
            nonce: trx.nonce,
            gas_price,
            gas_limit: trx.gas_limit,
            gas_used: trx.receipt().receipt.cumulative_gas_used,
            value: value.to_string(),
            logs: vec![],
        };

        for log_view in trx.receipt().logs() {
            let log = log_view.log;

            // Approval event
            if let Some(event) = balancer::events::Approval::match_and_decode(log) {
                total_approvals += 1;
                let event = pb::log::Log::Approval(pb::Approval {
                    owner: event.owner.to_vec(),
                    spender: event.spender.to_vec(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // Transfer event
            if let Some(event) = balancer::events::Transfer::match_and_decode(log) {
                total_transfers += 1;
                let event = pb::log::Log::Transfer(pb::Transfer {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // SwapFeePercentageChanged event
            if let Some(event) = balancer::events::SwapFeePercentageChanged::match_and_decode(log) {
                total_swap_fee_percentage_changed += 1;
                let event = pb::log::Log::SwapFeePercentageChanged(pb::SwapFeePercentageChanged {
                    swap_fee_percentage: event.swap_fee_percentage.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // PausedStateChanged event
            if let Some(event) = balancer::events::PausedStateChanged::match_and_decode(log) {
                total_paused_state_changed += 1;
                let event = pb::log::Log::PausedStateChanged(pb::PausedStateChanged { paused: event.paused });
                transaction.logs.push(create_log(log, event));
            }

            // RecoveryModeStateChanged event
            if let Some(event) = balancer::events::RecoveryModeStateChanged::match_and_decode(log) {
                total_recovery_mode_state_changed += 1;
                let event = pb::log::Log::RecoveryModeStateChanged(pb::RecoveryModeStateChanged { enabled: event.enabled });
                transaction.logs.push(create_log(log, event));
            }

            // ProtocolFeePercentageCacheUpdated event
            if let Some(event) = balancer::events::ProtocolFeePercentageCacheUpdated::match_and_decode(log) {
                total_protocol_fee_percentage_cache_updated += 1;
                let event = pb::log::Log::ProtocolFeePercentageCacheUpdated(pb::ProtocolFeePercentageCacheUpdated {
                    fee_type: event.fee_type.to_string(),
                    protocol_fee_percentage: event.protocol_fee_percentage.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total Approval events: {}", total_approvals);
    substreams::log::info!("Total Transfer events: {}", total_transfers);
    substreams::log::info!("Total SwapFeePercentageChanged events: {}", total_swap_fee_percentage_changed);
    substreams::log::info!("Total PausedStateChanged events: {}", total_paused_state_changed);
    substreams::log::info!("Total RecoveryModeStateChanged events: {}", total_recovery_mode_state_changed);
    substreams::log::info!(
        "Total ProtocolFeePercentageCacheUpdated events: {}",
        total_protocol_fee_percentage_cache_updated
    );
    Ok(events)
}
