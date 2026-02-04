use common::create::{CreateLog, CreateTransaction};
use proto::pb::steth::v1 as pb;
use substreams_abis::evm::tokens::steth::events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();

    // Counters for logging
    let mut total_submitted = 0;
    let mut total_unbuffered = 0;
    let mut total_token_rebased = 0;
    let mut total_transfer_shares = 0;
    let mut total_shares_burnt = 0;
    let mut total_external_shares_minted = 0;
    let mut total_external_shares_burnt = 0;
    let mut total_external_ether_transferred = 0;
    let mut total_external_bad_debt = 0;
    let mut total_max_external_ratio_set = 0;
    let mut total_cl_validators_updated = 0;
    let mut total_deposited_validators_changed = 0;
    let mut total_eth_distributed = 0;
    let mut total_internal_share_rate_updated = 0;
    let mut total_staking_paused = 0;
    let mut total_staking_resumed = 0;
    let mut total_staking_limit_set = 0;
    let mut total_staking_limit_removed = 0;
    let mut total_el_rewards_received = 0;
    let mut total_withdrawals_received = 0;
    let mut total_lido_locator_set = 0;

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
            // ============================================
            // Staking Events
            // ============================================

            // Submitted - Records a deposit made by a user
            if let Some(event) = events::Submitted::match_and_decode(log) {
                total_submitted += 1;
                let event = pb::log::Log::Submitted(pb::Submitted {
                    sender: event.sender.to_vec(),
                    amount: event.amount.to_string(),
                    referral: event.referral.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Unbuffered - The amount of ether was sent to the deposit_contract
            if let Some(event) = events::Unbuffered::match_and_decode(log) {
                total_unbuffered += 1;
                let event = pb::log::Log::Unbuffered(pb::Unbuffered {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // Rebase Events
            // ============================================

            // TokenRebased - Emitted when the token is rebased
            if let Some(event) = events::TokenRebased::match_and_decode(log) {
                total_token_rebased += 1;
                let event = pb::log::Log::TokenRebased(pb::TokenRebased {
                    report_timestamp: event.report_timestamp.to_string(),
                    time_elapsed: event.time_elapsed.to_string(),
                    pre_total_shares: event.pre_total_shares.to_string(),
                    pre_total_ether: event.pre_total_ether.to_string(),
                    post_total_shares: event.post_total_shares.to_string(),
                    post_total_ether: event.post_total_ether.to_string(),
                    shares_minted_as_fees: event.shares_minted_as_fees.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // TransferShares - Emitted when shares are transferred
            if let Some(event) = events::TransferShares::match_and_decode(log) {
                total_transfer_shares += 1;
                let event = pb::log::Log::TransferShares(pb::TransferShares {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    shares_value: event.shares_value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // SharesBurnt - Emitted when shares are burned
            if let Some(event) = events::SharesBurnt::match_and_decode(log) {
                total_shares_burnt += 1;
                let event = pb::log::Log::SharesBurnt(pb::SharesBurnt {
                    account: event.account.to_vec(),
                    pre_rebase_token_amount: event.pre_rebase_token_amount.to_string(),
                    post_rebase_token_amount: event.post_rebase_token_amount.to_string(),
                    shares_amount: event.shares_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // External Shares Events (Lido V3 stVaults)
            // ============================================

            // ExternalSharesMinted
            if let Some(event) = events::ExternalSharesMinted::match_and_decode(log) {
                total_external_shares_minted += 1;
                let event = pb::log::Log::ExternalSharesMinted(pb::ExternalSharesMinted {
                    recipient: event.receiver.to_vec(),
                    amount_of_shares: event.amount_of_shares.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ExternalSharesBurnt
            if let Some(event) = events::ExternalSharesBurnt::match_and_decode(log) {
                total_external_shares_burnt += 1;
                let event = pb::log::Log::ExternalSharesBurnt(pb::ExternalSharesBurnt {
                    amount_of_shares: event.amount_of_shares.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ExternalEtherTransferredToBuffer
            if let Some(event) = events::ExternalEtherTransferredToBuffer::match_and_decode(log) {
                total_external_ether_transferred += 1;
                let event = pb::log::Log::ExternalEtherTransferredToBuffer(pb::ExternalEtherTransferredToBuffer {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ExternalBadDebtInternalized
            if let Some(event) = events::ExternalBadDebtInternalized::match_and_decode(log) {
                total_external_bad_debt += 1;
                let event = pb::log::Log::ExternalBadDebtInternalized(pb::ExternalBadDebtInternalized {
                    amount_of_shares: event.amount_of_shares.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MaxExternalRatioBPSet
            if let Some(event) = events::MaxExternalRatioBpSet::match_and_decode(log) {
                total_max_external_ratio_set += 1;
                let event = pb::log::Log::MaxExternalRatioBpSet(pb::MaxExternalRatioBpSet {
                    max_external_ratio_bp: event.max_external_ratio_bp.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // Validator/CL Events
            // ============================================

            // CLValidatorsUpdated
            if let Some(event) = events::ClValidatorsUpdated::match_and_decode(log) {
                total_cl_validators_updated += 1;
                let event = pb::log::Log::ClValidatorsUpdated(pb::ClValidatorsUpdated {
                    report_timestamp: event.report_timestamp.to_string(),
                    pre_cl_validators: event.pre_cl_validators.to_string(),
                    post_cl_validators: event.post_cl_validators.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // DepositedValidatorsChanged
            if let Some(event) = events::DepositedValidatorsChanged::match_and_decode(log) {
                total_deposited_validators_changed += 1;
                let event = pb::log::Log::DepositedValidatorsChanged(pb::DepositedValidatorsChanged {
                    deposited_validators: event.deposited_validators.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ETHDistributed
            if let Some(event) = events::EthDistributed::match_and_decode(log) {
                total_eth_distributed += 1;
                let event = pb::log::Log::EthDistributed(pb::EthDistributed {
                    report_timestamp: event.report_timestamp.to_string(),
                    pre_cl_balance: event.pre_cl_balance.to_string(),
                    post_cl_balance: event.post_cl_balance.to_string(),
                    withdrawals_withdrawn: event.withdrawals_withdrawn.to_string(),
                    execution_layer_rewards_withdrawn: event.execution_layer_rewards_withdrawn.to_string(),
                    post_buffered_ether: event.post_buffered_ether.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // InternalShareRateUpdated
            if let Some(event) = events::InternalShareRateUpdated::match_and_decode(log) {
                total_internal_share_rate_updated += 1;
                let event = pb::log::Log::InternalShareRateUpdated(pb::InternalShareRateUpdated {
                    report_timestamp: event.report_timestamp.to_string(),
                    post_internal_shares: event.post_internal_shares.to_string(),
                    post_internal_ether: event.post_internal_ether.to_string(),
                    shares_minted_as_fees: event.shares_minted_as_fees.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // Protocol Control Events
            // ============================================

            // StakingPaused
            if events::StakingPaused::match_and_decode(log).is_some() {
                total_staking_paused += 1;
                let event = pb::log::Log::StakingPaused(pb::StakingPaused {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StakingResumed
            if events::StakingResumed::match_and_decode(log).is_some() {
                total_staking_resumed += 1;
                let event = pb::log::Log::StakingResumed(pb::StakingResumed {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StakingLimitSet
            if let Some(event) = events::StakingLimitSet::match_and_decode(log) {
                total_staking_limit_set += 1;
                let event = pb::log::Log::StakingLimitSet(pb::StakingLimitSet {
                    max_stake_limit: event.max_stake_limit.to_string(),
                    stake_limit_increase_per_block: event.stake_limit_increase_per_block.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StakingLimitRemoved
            if events::StakingLimitRemoved::match_and_decode(log).is_some() {
                total_staking_limit_removed += 1;
                let event = pb::log::Log::StakingLimitRemoved(pb::StakingLimitRemoved {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // EL/Withdrawal Events
            // ============================================

            // ELRewardsReceived
            if let Some(event) = events::ElRewardsReceived::match_and_decode(log) {
                total_el_rewards_received += 1;
                let event = pb::log::Log::ElRewardsReceived(pb::ElRewardsReceived {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // WithdrawalsReceived
            if let Some(event) = events::WithdrawalsReceived::match_and_decode(log) {
                total_withdrawals_received += 1;
                let event = pb::log::Log::WithdrawalsReceived(pb::WithdrawalsReceived {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // Locator Event
            // ============================================

            // LidoLocatorSet
            if let Some(event) = events::LidoLocatorSet::match_and_decode(log) {
                total_lido_locator_set += 1;
                let event = pb::log::Log::LidoLocatorSet(pb::LidoLocatorSet {
                    lido_locator: event.lido_locator.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }
        }

        // Only include transactions with logs
        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    // Log statistics
    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}\n", events.transactions.len());
    substreams::log::info!("--- Staking Events ---");
    substreams::log::info!("  Submitted: {}", total_submitted);
    substreams::log::info!("  Unbuffered: {}\n", total_unbuffered);
    substreams::log::info!("--- Rebase Events ---");
    substreams::log::info!("  TokenRebased: {}", total_token_rebased);
    substreams::log::info!("  TransferShares: {}", total_transfer_shares);
    substreams::log::info!("  SharesBurnt: {}\n", total_shares_burnt);
    substreams::log::info!("--- External Shares Events ---");
    substreams::log::info!("  ExternalSharesMinted: {}", total_external_shares_minted);
    substreams::log::info!("  ExternalSharesBurnt: {}", total_external_shares_burnt);
    substreams::log::info!("  ExternalEtherTransferredToBuffer: {}", total_external_ether_transferred);
    substreams::log::info!("  ExternalBadDebtInternalized: {}", total_external_bad_debt);
    substreams::log::info!("  MaxExternalRatioBPSet: {}\n", total_max_external_ratio_set);
    substreams::log::info!("--- Validator/CL Events ---");
    substreams::log::info!("  CLValidatorsUpdated: {}", total_cl_validators_updated);
    substreams::log::info!("  DepositedValidatorsChanged: {}", total_deposited_validators_changed);
    substreams::log::info!("  ETHDistributed: {}", total_eth_distributed);
    substreams::log::info!("  InternalShareRateUpdated: {}\n", total_internal_share_rate_updated);
    substreams::log::info!("--- Protocol Control Events ---");
    substreams::log::info!("  StakingPaused: {}", total_staking_paused);
    substreams::log::info!("  StakingResumed: {}", total_staking_resumed);
    substreams::log::info!("  StakingLimitSet: {}", total_staking_limit_set);
    substreams::log::info!("  StakingLimitRemoved: {}\n", total_staking_limit_removed);
    substreams::log::info!("--- EL/Withdrawal Events ---");
    substreams::log::info!("  ELRewardsReceived: {}", total_el_rewards_received);
    substreams::log::info!("  WithdrawalsReceived: {}\n", total_withdrawals_received);
    substreams::log::info!("--- Locator Event ---");
    substreams::log::info!("  LidoLocatorSet: {}", total_lido_locator_set);

    Ok(events)
}
