mod store;
use common::create::{CreateLog, CreateTransaction};
use proto::pb::curvefi::v1 as pb;
use substreams_abis::dex::curvefi;
use substreams_ethereum::pb::eth::v2::{Block, CallType, TransactionTrace};
use substreams_ethereum::Event;

fn get_create_address(trx: &TransactionTrace) -> Option<Vec<u8>> {
    for call in trx.calls.iter() {
        if call.call_type == CallType::Create as i32 {
            return Some(call.address.to_vec());
        }
    }
    None
}

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_token_exchange = 0;
    let mut total_add_liquidity = 0;
    let mut total_remove_liquidity = 0;
    let mut total_remove_liquidity_one = 0;
    let mut total_remove_liquidity_imbalance = 0;
    let mut total_commit_new_admin = 0;
    let mut total_new_admin = 0;
    let mut total_commit_new_fee = 0;
    let mut total_new_fee = 0;
    let mut total_ramp_a = 0;
    let mut total_stop_ramp_a = 0;
    let mut total_plain_pool_deployed = 0;
    let mut total_meta_pool_deployed = 0;
    let mut total_base_pool_added = 0;
    let mut total_liquidity_gauge_deployed = 0;
    let _total_init = 0; // Reserved for future Init event tracking

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
                trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
            } else {
                trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
            };
            for (log, call) in logs_with_calls {

            // TokenExchange event
            if let Some(event) = curvefi::pool::events::TokenExchange::match_and_decode(log) {
                total_token_exchange += 1;
                // Decode event data (simplified - would need full ethabi decoding for production)
                let event = pb::log::Log::TokenExchange(pb::TokenExchange {
                    buyer: event.buyer,
                    sold_id: event.sold_id.to_string(),
                    tokens_sold: event.tokens_sold.to_string(),
                    bought_id: event.bought_id.to_string(),
                    tokens_bought: event.tokens_bought.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // AddLiquidity event
            if let Some(event) = curvefi::pool::events::AddLiquidity::match_and_decode(log) {
                total_add_liquidity += 1;
                let event = pb::log::Log::AddLiquidity(pb::AddLiquidity {
                    provider: event.provider,
                    token_amounts: event.token_amounts.iter().map(|amt| amt.to_string()).collect(),
                    fees: event.fees.iter().map(|fee| fee.to_string()).collect(),
                    invariant: event.invariant.to_string(),
                    token_supply: event.token_supply.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RemoveLiquidity event
            if let Some(event) = curvefi::pool::events::RemoveLiquidity::match_and_decode(log) {
                total_remove_liquidity += 1;
                let event = pb::log::Log::RemoveLiquidity(pb::RemoveLiquidity {
                    provider: event.provider,
                    token_amounts: event.token_amounts.iter().map(|amt| amt.to_string()).collect(),
                    fees: event.fees.iter().map(|fee| fee.to_string()).collect(),
                    token_supply: event.token_supply.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RemoveLiquidityOne event
            if let Some(event) = curvefi::pool::events::RemoveLiquidityOne::match_and_decode(log) {
                total_remove_liquidity_one += 1;
                let event = pb::log::Log::RemoveLiquidityOne(pb::RemoveLiquidityOne {
                    provider: event.provider,
                    token_amount: event.token_amount.to_string(),
                    coin_amount: event.coin_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RemoveLiquidityImbalance event
            if let Some(event) = curvefi::pool::events::RemoveLiquidityImbalance::match_and_decode(log) {
                total_remove_liquidity_imbalance += 1;
                let event = pb::log::Log::RemoveLiquidityImbalance(pb::RemoveLiquidityImbalance {
                    provider: event.provider,
                    token_amounts: event.token_amounts.iter().map(|amt| amt.to_string()).collect(),
                    fees: event.fees.iter().map(|fee| fee.to_string()).collect(),
                    invariant: event.invariant.to_string(),
                    token_supply: event.token_supply.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // CommitNewAdmin event
            if let Some(event) = curvefi::pool::events::CommitNewAdmin::match_and_decode(log) {
                total_commit_new_admin += 1;
                let event = pb::log::Log::CommitNewAdmin(pb::CommitNewAdmin {
                    deadline: event.deadline.to_string(),
                    admin: event.admin,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // NewAdmin event
            if let Some(event) = curvefi::pool::events::NewAdmin::match_and_decode(log) {
                total_new_admin += 1;
                let event = pb::log::Log::NewAdmin(pb::NewAdmin { admin: event.admin });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // CommitNewFee event
            if let Some(event) = curvefi::pool::events::CommitNewFee::match_and_decode(log) {
                total_commit_new_fee += 1;
                let event = pb::log::Log::CommitNewFee(pb::CommitNewFee {
                    deadline: event.deadline.to_string(),
                    fee: event.fee.to_string(),
                    admin_fee: event.admin_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // NewFee event
            if let Some(event) = curvefi::pool::events::NewFee::match_and_decode(log) {
                total_new_fee += 1;
                let event = pb::log::Log::NewFee(pb::NewFee {
                    fee: event.fee.to_string(),
                    admin_fee: event.admin_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // RampA event
            if let Some(event) = curvefi::pool::events::RampA::match_and_decode(log) {
                total_ramp_a += 1;
                let event = pb::log::Log::RampA(pb::RampA {
                    old_a: event.old_a.to_string(),
                    new_a: event.new_a.to_string(),
                    initial_time: event.initial_time.to_string(),
                    future_time: event.future_time.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // StopRampA event
            if let Some(event) = curvefi::pool::events::StopRampA::match_and_decode(log) {
                total_stop_ramp_a += 1;
                let event = pb::log::Log::StopRampA(pb::StopRampA {
                    a: event.a.to_string(),
                    t: event.t.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // PlainPoolDeploy
            if let Some(event) = curvefi::factory::events::PlainPoolDeployed::match_and_decode(log) {
                if let Some(address) = get_create_address(trx) {
                    total_plain_pool_deployed += 1;
                    let event = pb::log::Log::PlainPoolDeployed(pb::PlainPoolDeployed {
                        address,
                        a: event.a.to_string(),
                        coins: event.coins,
                        deployer: event.deployer,
                        fee: event.fee.to_string(),
                    });
                    transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                }
            }

            // MetaPoolDeployed
            if let Some(event) = curvefi::factory::events::MetaPoolDeployed::match_and_decode(log) {
                if let Some(address) = get_create_address(trx) {
                    total_meta_pool_deployed += 1;
                    let event = pb::log::Log::MetaPoolDeployed(pb::MetaPoolDeployed {
                        address,
                        a: event.a.to_string(),
                        base_pool: event.base_pool,
                        coin: event.coin,
                        deployer: event.deployer,
                        fee: event.fee.to_string(),
                    });
                    transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                }
            }

            // BasePoolAdded
            if let Some(event) = curvefi::factory::events::BasePoolAdded::match_and_decode(log) {
                total_base_pool_added += 1;
                let event = pb::log::Log::BasePoolAdded(pb::BasePoolAdded { base_pool: event.base_pool });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
            }

            // LiquidityGaugeDeployed
            if let Some(event) = curvefi::factory::events::LiquidityGaugeDeployed::match_and_decode(log) {
                total_liquidity_gauge_deployed += 1;
                let event = pb::log::Log::LiquidityGaugeDeployed(pb::LiquidityGaugeDeployed {
                    pool: event.pool,
                    gauge: event.gauge,
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
    substreams::log::info!("Total TokenExchange events: {}", total_token_exchange);
    substreams::log::info!("Total AddLiquidity events: {}", total_add_liquidity);
    substreams::log::info!("Total RemoveLiquidity events: {}", total_remove_liquidity);
    substreams::log::info!("Total RemoveLiquidityOne events: {}", total_remove_liquidity_one);
    substreams::log::info!("Total RemoveLiquidityImbalance events: {}", total_remove_liquidity_imbalance);
    substreams::log::info!("Total CommitNewAdmin events: {}", total_commit_new_admin);
    substreams::log::info!("Total NewAdmin events: {}", total_new_admin);
    substreams::log::info!("Total CommitNewFee events: {}", total_commit_new_fee);
    substreams::log::info!("Total NewFee events: {}", total_new_fee);
    substreams::log::info!("Total RampA events: {}", total_ramp_a);
    substreams::log::info!("Total StopRampA events: {}", total_stop_ramp_a);
    substreams::log::info!("Total PlainPoolDeployed events: {}", total_plain_pool_deployed);
    substreams::log::info!("Total MetaPoolDeployed events: {}", total_meta_pool_deployed);
    substreams::log::info!("Total BasePoolAdded events: {}", total_base_pool_added);
    substreams::log::info!("Total LiquidityGaugeDeployed events: {}", total_liquidity_gauge_deployed);
    // Note: Init event tracking reserved for future implementation
    Ok(events)
}
