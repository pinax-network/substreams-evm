use common::create::{CreateLog, CreateTransaction};
use proto::pb::erc20::tokens::v1 as pb;
use substreams_abis::tokens::erc20::sai::events as sai_events;
use substreams_abis::tokens::erc20::steth::events as steth_events;
use substreams_abis::tokens::erc20::usdc::fiattoken_v2_2::events as usdc_events;
use substreams_abis::tokens::erc20::usdt::swap_asset::events as usdt_swap_events;
use substreams_abis::tokens::erc20::usdt::tethertoken_v0_4_18::events as usdt_events;
use substreams_abis::tokens::erc20::usdt::tethertoken_v0_4_18::functions as usdt_functions;
use substreams_abis::tokens::erc20::usdt::tethertoken_v0_8_4::events as usdt_v084_events;
use substreams_abis::tokens::erc20::wbtc::events as wbtc_events;
use substreams_abis::tokens::erc20::weth::events as weth_events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();

    // WETH counters
    let mut total_weth_deposits = 0;
    let mut total_weth_withdrawals = 0;

    // Shared counters (Pause/Unpause/OwnershipTransferred across USDC, USDT, WBTC)
    let mut total_pauses = 0;
    let mut total_unpauses = 0;
    let mut total_ownership_transferred = 0;

    // USDC counters
    let mut total_usdc_mints = 0;
    let mut total_usdc_burns = 0;
    let mut total_usdc_blacklisted = 0;
    let mut total_usdc_blacklister_changed = 0;
    let mut total_usdc_master_minter_changed = 0;
    let mut total_usdc_minter_configured = 0;
    let mut total_usdc_minter_removed = 0;
    let mut total_usdc_pauser_changed = 0;
    let mut total_usdc_rescuer_changed = 0;
    let mut total_usdc_un_blacklisted = 0;
    let mut total_usdc_authorization_canceled = 0;
    let mut total_usdc_authorization_used = 0;

    // USDT counters
    let mut total_usdt_issues = 0;
    let mut total_usdt_redeems = 0;
    let mut total_usdt_deprecates = 0;
    let mut total_usdt_params = 0;
    let mut total_usdt_destroyed_black_funds = 0;
    let mut total_usdt_added_black_list = 0;
    let mut total_usdt_removed_black_list = 0;

    // USDT v0.8.4 counters
    let mut total_usdt_block_placed = 0;
    let mut total_usdt_block_released = 0;
    let mut total_usdt_mints = 0;
    let mut total_usdt_destroyed_blocked_funds = 0;
    let mut total_usdt_new_privileged_contract = 0;
    let mut total_usdt_removed_privileged_contract = 0;

    // USDT swap_asset counters
    let mut total_usdt_log_swapin = 0;
    let mut total_usdt_log_swapout = 0;
    let mut total_usdt_log_change_dcrm_owner = 0;

    // WBTC counters
    let mut total_wbtc_mints = 0;
    let mut total_wbtc_burns = 0;
    let mut total_wbtc_mint_finished = 0;
    let mut total_wbtc_ownership_renounced = 0;

    // SAI counters
    let mut total_sai_mints = 0;
    let mut total_sai_burns = 0;
    let mut total_sai_log_set_authority = 0;
    let mut total_sai_log_set_owner = 0;

    // stETH counters
    let mut total_steth_submitted = 0;
    let mut total_steth_unbuffered = 0;
    let mut total_steth_token_rebased = 0;
    let mut total_steth_transfer_shares = 0;
    let mut total_steth_shares_burnt = 0;
    let mut total_steth_external_shares_minted = 0;
    let mut total_steth_external_shares_burnt = 0;
    let mut total_steth_external_ether_transferred = 0;
    let mut total_steth_external_bad_debt = 0;
    let mut total_steth_max_external_ratio_set = 0;
    let mut total_steth_cl_validators_updated = 0;
    let mut total_steth_deposited_validators_changed = 0;
    let mut total_steth_eth_distributed = 0;
    let mut total_steth_internal_share_rate_updated = 0;
    let mut total_steth_staking_paused = 0;
    let mut total_steth_staking_resumed = 0;
    let mut total_steth_staking_limit_set = 0;
    let mut total_steth_staking_limit_removed = 0;
    let mut total_steth_el_rewards_received = 0;
    let mut total_steth_withdrawals_received = 0;
    let mut total_steth_lido_locator_set = 0;

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
            // WETH Events
            // ============================================

            // Deposit
            if let Some(event) = weth_events::Deposit::match_and_decode(log) {
                total_weth_deposits += 1;
                let event = pb::log::Log::WethDeposit(pb::WethDeposit {
                    dst: event.dst.to_vec(),
                    wad: event.wad.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Withdrawal
            if let Some(event) = weth_events::Withdrawal::match_and_decode(log) {
                total_weth_withdrawals += 1;
                let event = pb::log::Log::WethWithdrawal(pb::WethWithdrawal {
                    src: event.src.to_vec(),
                    wad: event.wad.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // USDC Events
            // ============================================

            // Mint
            if let Some(event) = usdc_events::Mint::match_and_decode(log) {
                total_usdc_mints += 1;
                let event = pb::log::Log::UsdcMint(pb::UsdcMint {
                    minter: event.minter.to_vec(),
                    to: event.to.to_vec(),
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Burn
            if let Some(event) = usdc_events::Burn::match_and_decode(log) {
                total_usdc_burns += 1;
                let event = pb::log::Log::UsdcBurn(pb::UsdcBurn {
                    burner: event.burner.to_vec(),
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Blacklisted
            if let Some(event) = usdc_events::Blacklisted::match_and_decode(log) {
                total_usdc_blacklisted += 1;
                let event = pb::log::Log::UsdcBlacklisted(pb::UsdcBlacklisted {
                    account: event.account.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // BlacklisterChanged
            if let Some(event) = usdc_events::BlacklisterChanged::match_and_decode(log) {
                total_usdc_blacklister_changed += 1;
                let event = pb::log::Log::UsdcBlacklisterChanged(pb::UsdcBlacklisterChanged {
                    new_blacklister: event.new_blacklister.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MasterMinterChanged
            if let Some(event) = usdc_events::MasterMinterChanged::match_and_decode(log) {
                total_usdc_master_minter_changed += 1;
                let event = pb::log::Log::UsdcMasterMinterChanged(pb::UsdcMasterMinterChanged {
                    new_master_minter: event.new_master_minter.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MinterConfigured
            if let Some(event) = usdc_events::MinterConfigured::match_and_decode(log) {
                total_usdc_minter_configured += 1;
                let event = pb::log::Log::UsdcMinterConfigured(pb::UsdcMinterConfigured {
                    minter: event.minter.to_vec(),
                    minter_allowed_amount: event.minter_allowed_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MinterRemoved
            if let Some(event) = usdc_events::MinterRemoved::match_and_decode(log) {
                total_usdc_minter_removed += 1;
                let event = pb::log::Log::UsdcMinterRemoved(pb::UsdcMinterRemoved {
                    old_minter: event.old_minter.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // OwnershipTransferred (shared: USDC, WBTC)
            if let Some(event) = usdc_events::OwnershipTransferred::match_and_decode(log) {
                total_ownership_transferred += 1;
                let event = pb::log::Log::OwnershipTransferred(pb::OwnershipTransferred {
                    previous_owner: event.previous_owner.to_vec(),
                    new_owner: event.new_owner.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Pause (shared: USDC, USDT, WBTC)
            if usdc_events::Pause::match_and_decode(log).is_some() {
                total_pauses += 1;
                let event = pb::log::Log::Pause(pb::Pause {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PauserChanged
            if let Some(event) = usdc_events::PauserChanged::match_and_decode(log) {
                total_usdc_pauser_changed += 1;
                let event = pb::log::Log::UsdcPauserChanged(pb::UsdcPauserChanged {
                    new_address: event.new_address.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RescuerChanged
            if let Some(event) = usdc_events::RescuerChanged::match_and_decode(log) {
                total_usdc_rescuer_changed += 1;
                let event = pb::log::Log::UsdcRescuerChanged(pb::UsdcRescuerChanged {
                    new_rescuer: event.new_rescuer.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // UnBlacklisted
            if let Some(event) = usdc_events::UnBlacklisted::match_and_decode(log) {
                total_usdc_un_blacklisted += 1;
                let event = pb::log::Log::UsdcUnBlacklisted(pb::UsdcUnBlacklisted {
                    account: event.account.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Unpause (shared: USDC, USDT, WBTC)
            if usdc_events::Unpause::match_and_decode(log).is_some() {
                total_unpauses += 1;
                let event = pb::log::Log::Unpause(pb::Unpause {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // AuthorizationCanceled
            if let Some(event) = usdc_events::AuthorizationCanceled::match_and_decode(log) {
                total_usdc_authorization_canceled += 1;
                let event = pb::log::Log::UsdcAuthorizationCanceled(pb::UsdcAuthorizationCanceled {
                    authorizer: event.authorizer.to_vec(),
                    nonce: event.nonce.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // AuthorizationUsed
            if let Some(event) = usdc_events::AuthorizationUsed::match_and_decode(log) {
                total_usdc_authorization_used += 1;
                let event = pb::log::Log::UsdcAuthorizationUsed(pb::UsdcAuthorizationUsed {
                    authorizer: event.authorizer.to_vec(),
                    nonce: event.nonce.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // USDT Events
            // ============================================

            // Issue
            if let Some(event) = usdt_events::Issue::match_and_decode(log) {
                if let Some(owner) = (usdt_functions::Owner {}).call(log.address.to_vec()) {
                    total_usdt_issues += 1;
                    let event = pb::log::Log::UsdtIssue(pb::UsdtIssue {
                        amount: event.amount.to_string(),
                        owner,
                    });
                    transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                }
            }

            // Redeem
            if let Some(event) = usdt_events::Redeem::match_and_decode(log) {
                if let Some(owner) = (usdt_functions::Owner {}).call(log.address.to_vec()) {
                    total_usdt_redeems += 1;
                    let event = pb::log::Log::UsdtRedeem(pb::UsdtRedeem {
                        amount: event.amount.to_string(),
                        owner,
                    });
                    transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                }
            }

            // Deprecate
            if let Some(event) = usdt_events::Deprecate::match_and_decode(log) {
                total_usdt_deprecates += 1;
                let event = pb::log::Log::UsdtDeprecate(pb::UsdtDeprecate {
                    new_address: event.new_address.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Params
            if let Some(event) = usdt_events::Params::match_and_decode(log) {
                total_usdt_params += 1;
                let event = pb::log::Log::UsdtParams(pb::UsdtParams {
                    fee_basis_points: event.fee_basis_points.to_string(),
                    max_fee: event.max_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // DestroyedBlackFunds
            if let Some(event) = usdt_events::DestroyedBlackFunds::match_and_decode(log) {
                total_usdt_destroyed_black_funds += 1;
                let event = pb::log::Log::UsdtDestroyedBlackFunds(pb::UsdtDestroyedBlackFunds {
                    black_listed_user: event.black_listed_user.to_vec(),
                    balance: event.balance.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // AddedBlackList
            if let Some(event) = usdt_events::AddedBlackList::match_and_decode(log) {
                total_usdt_added_black_list += 1;
                let event = pb::log::Log::UsdtAddedBlackList(pb::UsdtAddedBlackList { user: event.user.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RemovedBlackList
            if let Some(event) = usdt_events::RemovedBlackList::match_and_decode(log) {
                total_usdt_removed_black_list += 1;
                let event = pb::log::Log::UsdtRemovedBlackList(pb::UsdtRemovedBlackList { user: event.user.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // USDT Events (tethertoken_v0_8_4)
            // ============================================

            // BlockPlaced
            if let Some(event) = usdt_v084_events::BlockPlaced::match_and_decode(log) {
                total_usdt_block_placed += 1;
                let event = pb::log::Log::UsdtBlockPlaced(pb::UsdtBlockPlaced { user: event.user.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // BlockReleased
            if let Some(event) = usdt_v084_events::BlockReleased::match_and_decode(log) {
                total_usdt_block_released += 1;
                let event = pb::log::Log::UsdtBlockReleased(pb::UsdtBlockReleased { user: event.user.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Mint (v0.8.4 — different from Issue in v0.4.18)
            if let Some(event) = usdt_v084_events::Mint::match_and_decode(log) {
                total_usdt_mints += 1;
                let event = pb::log::Log::UsdtMint(pb::UsdtMint {
                    destination: event.destination.to_vec(),
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // DestroyedBlockedFunds (v0.8.4 — different from DestroyedBlackFunds in v0.4.18)
            if let Some(event) = usdt_v084_events::DestroyedBlockedFunds::match_and_decode(log) {
                total_usdt_destroyed_blocked_funds += 1;
                let event = pb::log::Log::UsdtDestroyedBlockedFunds(pb::UsdtDestroyedBlockedFunds {
                    blocked_user: event.blocked_user.to_vec(),
                    balance: event.balance.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // NewPrivilegedContract
            if let Some(event) = usdt_v084_events::NewPrivilegedContract::match_and_decode(log) {
                total_usdt_new_privileged_contract += 1;
                let event = pb::log::Log::UsdtNewPrivilegedContract(pb::UsdtNewPrivilegedContract {
                    contract: event.contract.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RemovedPrivilegedContract
            if let Some(event) = usdt_v084_events::RemovedPrivilegedContract::match_and_decode(log) {
                total_usdt_removed_privileged_contract += 1;
                let event = pb::log::Log::UsdtRemovedPrivilegedContract(pb::UsdtRemovedPrivilegedContract {
                    contract: event.contract.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // USDT Events (swap_asset)
            // ============================================

            // LogSwapin
            if let Some(event) = usdt_swap_events::LogSwapin::match_and_decode(log) {
                total_usdt_log_swapin += 1;
                let event = pb::log::Log::UsdtLogSwapin(pb::UsdtLogSwapin {
                    txhash: event.txhash.to_vec(),
                    account: event.account.to_vec(),
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LogSwapout
            if let Some(event) = usdt_swap_events::LogSwapout::match_and_decode(log) {
                total_usdt_log_swapout += 1;
                let event = pb::log::Log::UsdtLogSwapout(pb::UsdtLogSwapout {
                    account: event.account.to_vec(),
                    bindaddr: event.bindaddr.to_vec(),
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LogChangeDCRMOwner
            if let Some(event) = usdt_swap_events::LogChangeDcrmOwner::match_and_decode(log) {
                total_usdt_log_change_dcrm_owner += 1;
                let event = pb::log::Log::UsdtLogChangeDcrmOwner(pb::UsdtLogChangeDcrmOwner {
                    old_owner: event.old_owner.to_vec(),
                    new_owner: event.new_owner.to_vec(),
                    effective_height: event.effective_height.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // WBTC Events
            // ============================================

            // Mint
            if let Some(event) = wbtc_events::Mint::match_and_decode(log) {
                total_wbtc_mints += 1;
                let event = pb::log::Log::WbtcMint(pb::WbtcMint {
                    to: event.to.to_vec(),
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Burn
            if let Some(event) = wbtc_events::Burn::match_and_decode(log) {
                total_wbtc_burns += 1;
                let event = pb::log::Log::WbtcBurn(pb::WbtcBurn {
                    burner: event.burner.to_vec(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MintFinished
            if wbtc_events::MintFinished::match_and_decode(log).is_some() {
                total_wbtc_mint_finished += 1;
                let event = pb::log::Log::WbtcMintFinished(pb::WbtcMintFinished {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // OwnershipRenounced
            if let Some(event) = wbtc_events::OwnershipRenounced::match_and_decode(log) {
                total_wbtc_ownership_renounced += 1;
                let event = pb::log::Log::WbtcOwnershipRenounced(pb::WbtcOwnershipRenounced {
                    previous_owner: event.previous_owner.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // SAI Events
            // ============================================

            // Mint
            if let Some(event) = sai_events::Mint::match_and_decode(log) {
                total_sai_mints += 1;
                let event = pb::log::Log::SaiMint(pb::SaiMint {
                    guy: event.guy.to_vec(),
                    wad: event.wad.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Burn
            if let Some(event) = sai_events::Burn::match_and_decode(log) {
                total_sai_burns += 1;
                let event = pb::log::Log::SaiBurn(pb::SaiBurn {
                    guy: event.guy.to_vec(),
                    wad: event.wad.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LogSetAuthority
            if let Some(event) = sai_events::LogSetAuthority::match_and_decode(log) {
                total_sai_log_set_authority += 1;
                let event = pb::log::Log::SaiLogSetAuthority(pb::SaiLogSetAuthority {
                    authority: event.authority.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LogSetOwner
            if let Some(event) = sai_events::LogSetOwner::match_and_decode(log) {
                total_sai_log_set_owner += 1;
                let event = pb::log::Log::SaiLogSetOwner(pb::SaiLogSetOwner { owner: event.owner.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ============================================
            // stETH Events
            // ============================================

            // Submitted
            if let Some(event) = steth_events::Submitted::match_and_decode(log) {
                total_steth_submitted += 1;
                let event = pb::log::Log::StethSubmitted(pb::StethSubmitted {
                    sender: event.sender.to_vec(),
                    amount: event.amount.to_string(),
                    referral: event.referral.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Unbuffered
            if let Some(event) = steth_events::Unbuffered::match_and_decode(log) {
                total_steth_unbuffered += 1;
                let event = pb::log::Log::StethUnbuffered(pb::StethUnbuffered {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // TokenRebased
            if let Some(event) = steth_events::TokenRebased::match_and_decode(log) {
                total_steth_token_rebased += 1;
                let event = pb::log::Log::StethTokenRebased(pb::StethTokenRebased {
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

            // TransferShares
            if let Some(event) = steth_events::TransferShares::match_and_decode(log) {
                total_steth_transfer_shares += 1;
                let event = pb::log::Log::StethTransferShares(pb::StethTransferShares {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    shares_value: event.shares_value.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // SharesBurnt
            if let Some(event) = steth_events::SharesBurnt::match_and_decode(log) {
                total_steth_shares_burnt += 1;
                let event = pb::log::Log::StethSharesBurnt(pb::StethSharesBurnt {
                    account: event.account.to_vec(),
                    pre_rebase_token_amount: event.pre_rebase_token_amount.to_string(),
                    post_rebase_token_amount: event.post_rebase_token_amount.to_string(),
                    shares_amount: event.shares_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ExternalSharesMinted
            if let Some(event) = steth_events::ExternalSharesMinted::match_and_decode(log) {
                total_steth_external_shares_minted += 1;
                let event = pb::log::Log::StethExternalSharesMinted(pb::StethExternalSharesMinted {
                    recipient: event.receiver.to_vec(),
                    amount_of_shares: event.amount_of_shares.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ExternalSharesBurnt (requires call metadata for owner)
            if let Some(call) = call {
                if let Some(event) = steth_events::ExternalSharesBurnt::match_and_decode(log) {
                    total_steth_external_shares_burnt += 1;
                    let event = pb::log::Log::StethExternalSharesBurnt(pb::StethExternalSharesBurnt {
                        amount_of_shares: event.amount_of_shares.to_string(),
                        owner: call.caller.to_vec(),
                    });
                    transaction.logs.push(pb::Log::create_log_with_call(log, event, Some(call)));
                }
            }

            // ExternalEtherTransferredToBuffer
            if let Some(event) = steth_events::ExternalEtherTransferredToBuffer::match_and_decode(log) {
                total_steth_external_ether_transferred += 1;
                let event = pb::log::Log::StethExternalEtherTransferredToBuffer(pb::StethExternalEtherTransferredToBuffer {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ExternalBadDebtInternalized
            if let Some(event) = steth_events::ExternalBadDebtInternalized::match_and_decode(log) {
                total_steth_external_bad_debt += 1;
                let event = pb::log::Log::StethExternalBadDebtInternalized(pb::StethExternalBadDebtInternalized {
                    amount_of_shares: event.amount_of_shares.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MaxExternalRatioBPSet
            if let Some(event) = steth_events::MaxExternalRatioBpSet::match_and_decode(log) {
                total_steth_max_external_ratio_set += 1;
                let event = pb::log::Log::StethMaxExternalRatioBpSet(pb::StethMaxExternalRatioBpSet {
                    max_external_ratio_bp: event.max_external_ratio_bp.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // CLValidatorsUpdated
            if let Some(event) = steth_events::ClValidatorsUpdated::match_and_decode(log) {
                total_steth_cl_validators_updated += 1;
                let event = pb::log::Log::StethClValidatorsUpdated(pb::StethClValidatorsUpdated {
                    report_timestamp: event.report_timestamp.to_string(),
                    pre_cl_validators: event.pre_cl_validators.to_string(),
                    post_cl_validators: event.post_cl_validators.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // DepositedValidatorsChanged
            if let Some(event) = steth_events::DepositedValidatorsChanged::match_and_decode(log) {
                total_steth_deposited_validators_changed += 1;
                let event = pb::log::Log::StethDepositedValidatorsChanged(pb::StethDepositedValidatorsChanged {
                    deposited_validators: event.deposited_validators.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ETHDistributed
            if let Some(event) = steth_events::EthDistributed::match_and_decode(log) {
                total_steth_eth_distributed += 1;
                let event = pb::log::Log::StethEthDistributed(pb::StethEthDistributed {
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
            if let Some(event) = steth_events::InternalShareRateUpdated::match_and_decode(log) {
                total_steth_internal_share_rate_updated += 1;
                let event = pb::log::Log::StethInternalShareRateUpdated(pb::StethInternalShareRateUpdated {
                    report_timestamp: event.report_timestamp.to_string(),
                    post_internal_shares: event.post_internal_shares.to_string(),
                    post_internal_ether: event.post_internal_ether.to_string(),
                    shares_minted_as_fees: event.shares_minted_as_fees.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StakingPaused
            if steth_events::StakingPaused::match_and_decode(log).is_some() {
                total_steth_staking_paused += 1;
                let event = pb::log::Log::StethStakingPaused(pb::StethStakingPaused {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StakingResumed
            if steth_events::StakingResumed::match_and_decode(log).is_some() {
                total_steth_staking_resumed += 1;
                let event = pb::log::Log::StethStakingResumed(pb::StethStakingResumed {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StakingLimitSet
            if let Some(event) = steth_events::StakingLimitSet::match_and_decode(log) {
                total_steth_staking_limit_set += 1;
                let event = pb::log::Log::StethStakingLimitSet(pb::StethStakingLimitSet {
                    max_stake_limit: event.max_stake_limit.to_string(),
                    stake_limit_increase_per_block: event.stake_limit_increase_per_block.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StakingLimitRemoved
            if steth_events::StakingLimitRemoved::match_and_decode(log).is_some() {
                total_steth_staking_limit_removed += 1;
                let event = pb::log::Log::StethStakingLimitRemoved(pb::StethStakingLimitRemoved {});
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // ELRewardsReceived
            if let Some(event) = steth_events::ElRewardsReceived::match_and_decode(log) {
                total_steth_el_rewards_received += 1;
                let event = pb::log::Log::StethElRewardsReceived(pb::StethElRewardsReceived {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // WithdrawalsReceived
            if let Some(event) = steth_events::WithdrawalsReceived::match_and_decode(log) {
                total_steth_withdrawals_received += 1;
                let event = pb::log::Log::StethWithdrawalsReceived(pb::StethWithdrawalsReceived {
                    amount: event.amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LidoLocatorSet
            if let Some(event) = steth_events::LidoLocatorSet::match_and_decode(log) {
                total_steth_lido_locator_set += 1;
                let event = pb::log::Log::StethLidoLocatorSet(pb::StethLidoLocatorSet {
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

    substreams::log::info!("--- Shared Events ---");
    substreams::log::info!("  Pause: {}", total_pauses);
    substreams::log::info!("  Unpause: {}", total_unpauses);
    substreams::log::info!("  OwnershipTransferred: {}\n", total_ownership_transferred);

    substreams::log::info!("--- WETH Events ---");
    substreams::log::info!("  Deposit: {}", total_weth_deposits);
    substreams::log::info!("  Withdrawal: {}\n", total_weth_withdrawals);

    substreams::log::info!("--- USDC Events ---");
    substreams::log::info!("  Mint: {}", total_usdc_mints);
    substreams::log::info!("  Burn: {}", total_usdc_burns);
    substreams::log::info!("  Blacklisted: {}", total_usdc_blacklisted);
    substreams::log::info!("  BlacklisterChanged: {}", total_usdc_blacklister_changed);
    substreams::log::info!("  MasterMinterChanged: {}", total_usdc_master_minter_changed);
    substreams::log::info!("  MinterConfigured: {}", total_usdc_minter_configured);
    substreams::log::info!("  MinterRemoved: {}", total_usdc_minter_removed);
    substreams::log::info!("  PauserChanged: {}", total_usdc_pauser_changed);
    substreams::log::info!("  RescuerChanged: {}", total_usdc_rescuer_changed);
    substreams::log::info!("  UnBlacklisted: {}", total_usdc_un_blacklisted);
    substreams::log::info!("  AuthorizationCanceled: {}", total_usdc_authorization_canceled);
    substreams::log::info!("  AuthorizationUsed: {}\n", total_usdc_authorization_used);

    substreams::log::info!("--- USDT Events ---");
    substreams::log::info!("  Issue: {}", total_usdt_issues);
    substreams::log::info!("  Redeem: {}", total_usdt_redeems);
    substreams::log::info!("  Deprecate: {}", total_usdt_deprecates);
    substreams::log::info!("  Params: {}", total_usdt_params);
    substreams::log::info!("  DestroyedBlackFunds: {}", total_usdt_destroyed_black_funds);
    substreams::log::info!("  AddedBlackList: {}", total_usdt_added_black_list);
    substreams::log::info!("  RemovedBlackList: {}\n", total_usdt_removed_black_list);

    substreams::log::info!("--- USDT v0.8.4 Events ---");
    substreams::log::info!("  BlockPlaced: {}", total_usdt_block_placed);
    substreams::log::info!("  BlockReleased: {}", total_usdt_block_released);
    substreams::log::info!("  Mint: {}", total_usdt_mints);
    substreams::log::info!("  DestroyedBlockedFunds: {}", total_usdt_destroyed_blocked_funds);
    substreams::log::info!("  NewPrivilegedContract: {}", total_usdt_new_privileged_contract);
    substreams::log::info!("  RemovedPrivilegedContract: {}\n", total_usdt_removed_privileged_contract);

    substreams::log::info!("--- USDT swap_asset Events ---");
    substreams::log::info!("  LogSwapin: {}", total_usdt_log_swapin);
    substreams::log::info!("  LogSwapout: {}", total_usdt_log_swapout);
    substreams::log::info!("  LogChangeDCRMOwner: {}\n", total_usdt_log_change_dcrm_owner);

    substreams::log::info!("--- WBTC Events ---");
    substreams::log::info!("  Mint: {}", total_wbtc_mints);
    substreams::log::info!("  Burn: {}", total_wbtc_burns);
    substreams::log::info!("  MintFinished: {}", total_wbtc_mint_finished);
    substreams::log::info!("  OwnershipRenounced: {}\n", total_wbtc_ownership_renounced);

    substreams::log::info!("--- SAI Events ---");
    substreams::log::info!("  Mint: {}", total_sai_mints);
    substreams::log::info!("  Burn: {}", total_sai_burns);
    substreams::log::info!("  LogSetAuthority: {}", total_sai_log_set_authority);
    substreams::log::info!("  LogSetOwner: {}\n", total_sai_log_set_owner);

    substreams::log::info!("--- stETH Events ---");
    substreams::log::info!("  Submitted: {}", total_steth_submitted);
    substreams::log::info!("  Unbuffered: {}", total_steth_unbuffered);
    substreams::log::info!("  TokenRebased: {}", total_steth_token_rebased);
    substreams::log::info!("  TransferShares: {}", total_steth_transfer_shares);
    substreams::log::info!("  SharesBurnt: {}", total_steth_shares_burnt);
    substreams::log::info!("  ExternalSharesMinted: {}", total_steth_external_shares_minted);
    substreams::log::info!("  ExternalSharesBurnt: {}", total_steth_external_shares_burnt);
    substreams::log::info!("  ExternalEtherTransferredToBuffer: {}", total_steth_external_ether_transferred);
    substreams::log::info!("  ExternalBadDebtInternalized: {}", total_steth_external_bad_debt);
    substreams::log::info!("  MaxExternalRatioBPSet: {}", total_steth_max_external_ratio_set);
    substreams::log::info!("  CLValidatorsUpdated: {}", total_steth_cl_validators_updated);
    substreams::log::info!("  DepositedValidatorsChanged: {}", total_steth_deposited_validators_changed);
    substreams::log::info!("  ETHDistributed: {}", total_steth_eth_distributed);
    substreams::log::info!("  InternalShareRateUpdated: {}", total_steth_internal_share_rate_updated);
    substreams::log::info!("  StakingPaused: {}", total_steth_staking_paused);
    substreams::log::info!("  StakingResumed: {}", total_steth_staking_resumed);
    substreams::log::info!("  StakingLimitSet: {}", total_steth_staking_limit_set);
    substreams::log::info!("  StakingLimitRemoved: {}", total_steth_staking_limit_removed);
    substreams::log::info!("  ELRewardsReceived: {}", total_steth_el_rewards_received);
    substreams::log::info!("  WithdrawalsReceived: {}", total_steth_withdrawals_received);
    substreams::log::info!("  LidoLocatorSet: {}", total_steth_lido_locator_set);

    Ok(events)
}
