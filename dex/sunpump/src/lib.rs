mod store;
use common::create::{CreateLog, CreateTransaction};
use proto::pb::sunpump::v1 as pb;
use substreams_abis::dex::sunpump::legacy::launchpad::events::TokenCreate as TokenCreateLegacy;
use substreams_abis::dex::sunpump::v1::launchpadproxy::events;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_launch_pending = 0;
    let mut total_launcher_changed = 0;
    let mut total_min_tx_fee_set = 0;
    let mut total_mint_fee_set = 0;
    let mut total_operator_changed = 0;
    let mut total_owner_changed = 0;
    let mut total_pending_owner_set = 0;
    let mut total_purchase_fee_set = 0;
    let mut total_sale_fee_set = 0;
    let mut total_token_create = 0;
    let mut total_token_launched = 0;
    let mut total_token_purchased = 0;
    let mut total_token_sold = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
                trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
            } else {
                trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
            };
            for (log, call) in logs_with_calls {

            // TokenCreate event
            if let Some(event) = events::TokenCreate::match_and_decode(log) {
                total_token_create += 1;
                let event = pb::log::Log::TokenCreate(pb::TokenCreate {
                    token_address: event.token_address.to_vec(),
                    token_index: event.token_index.to_string(),
                    creator: event.creator.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // Legacy - TokenCreate event
            if let Some(event) = TokenCreateLegacy::match_and_decode(log) {
                total_token_create += 1;
                let event = pb::log::Log::TokenCreateLegacy(pb::TokenCreateLegacy {
                    token_address: event.token_address.to_vec(),
                    creator: event.creator.to_vec(),
                    nft_threshold: event.nft_threshold.to_u64(),
                    nft_max_supply: event.nft_max_supply.to_u64(),
                    name: event.name,
                    symbol: event.symbol,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LaunchPending event
            if let Some(event) = events::LaunchPending::match_and_decode(log) {
                total_launch_pending += 1;
                let event = pb::log::Log::LaunchPending(pb::LaunchPending { token: event.token.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LauncherChanged event
            if let Some(event) = events::LauncherChanged::match_and_decode(log) {
                total_launcher_changed += 1;
                let event = pb::log::Log::LauncherChanged(pb::LauncherChanged {
                    old_launcher: event.old_launcher.to_vec(),
                    new_launcher: event.new_launcher.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MinTxFeeSet event
            if let Some(event) = events::MinTxFeeSet::match_and_decode(log) {
                total_min_tx_fee_set += 1;
                let event = pb::log::Log::MinTxFeeSet(pb::MinTxFeeSet {
                    old_fee: event.old_fee.to_string(),
                    new_fee: event.new_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // MintFeeSet event
            if let Some(event) = events::MintFeeSet::match_and_decode(log) {
                total_mint_fee_set += 1;
                let event = pb::log::Log::MintFeeSet(pb::MintFeeSet {
                    old_fee: event.old_fee.to_string(),
                    new_fee: event.new_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // OperatorChanged event
            if let Some(event) = events::OperatorChanged::match_and_decode(log) {
                total_operator_changed += 1;
                let event = pb::log::Log::OperatorChanged(pb::OperatorChanged {
                    old_operator: event.old_operator.to_vec(),
                    new_operator: event.new_operator.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // OwnerChanged event
            if let Some(event) = events::OwnerChanged::match_and_decode(log) {
                total_owner_changed += 1;
                let event = pb::log::Log::OwnerChanged(pb::OwnerChanged {
                    old_owner: event.old_owner.to_vec(),
                    new_owner: event.new_owner.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PendingOwnerSet event
            if let Some(event) = events::PendingOwnerSet::match_and_decode(log) {
                total_pending_owner_set += 1;
                let event = pb::log::Log::PendingOwnerSet(pb::PendingOwnerSet {
                    old_pending_owner: event.old_pending_owner.to_vec(),
                    new_pending_owner: event.new_pending_owner.to_vec(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PurchaseFeeSet event
            if let Some(event) = events::PurchaseFeeSet::match_and_decode(log) {
                total_purchase_fee_set += 1;
                let event = pb::log::Log::PurchaseFeeSet(pb::PurchaseFeeSet {
                    old_fee: event.old_fee.to_string(),
                    new_fee: event.new_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // SaleFeeSet event
            if let Some(event) = events::SaleFeeSet::match_and_decode(log) {
                total_sale_fee_set += 1;
                let event = pb::log::Log::SaleFeeSet(pb::SaleFeeSet {
                    old_fee: event.old_fee.to_string(),
                    new_fee: event.new_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // TokenLaunched event
            if let Some(event) = events::TokenLaunched::match_and_decode(log) {
                total_token_launched += 1;
                let event = pb::log::Log::TokenLaunched(pb::TokenLaunched { token: event.token.to_vec() });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // TokenPurchased event
            if let Some(event) = events::TokenPurchased::match_and_decode(log) {
                total_token_purchased += 1;
                let event = pb::log::Log::TokenPurchased(pb::TokenPurchased {
                    token: event.token.to_vec(),
                    buyer: event.buyer.to_vec(),
                    trx_amount: event.trx_amount.to_string(),
                    fee: event.fee.to_string(),
                    token_amount: event.token_amount.to_string(),
                    token_reserve: event.token_reserve.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // TokenSold event
            if let Some(event) = events::TokenSold::match_and_decode(log) {
                total_token_sold += 1;
                let event = pb::log::Log::TokenSold(pb::TokenSold {
                    token: event.token.to_vec(),
                    seller: event.seller.to_vec(),
                    trx_amount: event.trx_amount.to_string(),
                    fee: event.fee.to_string(),
                    token_amount: event.token_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total LaunchPending events: {}", total_launch_pending);
    substreams::log::info!("Total LauncherChanged events: {}", total_launcher_changed);
    substreams::log::info!("Total MinTxFeeSet events: {}", total_min_tx_fee_set);
    substreams::log::info!("Total MintFeeSet events: {}", total_mint_fee_set);
    substreams::log::info!("Total OperatorChanged events: {}", total_operator_changed);
    substreams::log::info!("Total OwnerChanged events: {}", total_owner_changed);
    substreams::log::info!("Total PendingOwnerSet events: {}", total_pending_owner_set);
    substreams::log::info!("Total PurchaseFeeSet events: {}", total_purchase_fee_set);
    substreams::log::info!("Total SaleFeeSet events: {}", total_sale_fee_set);
    substreams::log::info!("Total TokenCreate events: {}", total_token_create);
    substreams::log::info!("Total TokenLaunched events: {}", total_token_launched);
    substreams::log::info!("Total TokenPurchased events: {}", total_token_purchased);
    substreams::log::info!("Total TokenSold events: {}", total_token_sold);
    Ok(events)
}
