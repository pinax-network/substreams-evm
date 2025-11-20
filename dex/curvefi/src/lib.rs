pub mod store;
use proto::pb::dex::curvefi::v1 as pb;
use substreams_abis::evm::curvefi::pool as curvefi;
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

            // TokenExchange event
            if let Some(event) = curvefi::events::TokenExchange::match_and_decode(log) {
                total_token_exchange += 1;
                // Decode event data (simplified - would need full ethabi decoding for production)
                let event = pb::log::Log::TokenExchange(pb::TokenExchange {
                    buyer: vec![], // Would extract from topics[1]
                    sold_id: event.sold_id,
                    tokens_sold: event.tokens_sold,
                    bought_id: event.bought_id,
                    tokens_bought: event.tokens_bought,
                });
                transaction.logs.push(create_log(log, event));
            }

            // AddLiquidity event
            if let Some(event) = curvefi::events::AddLiquidity::match_and_decode(log) {
                total_add_liquidity += 1;
                let event = pb::log::Log::AddLiquidity(pb::AddLiquidity {
                    provider: event.provider,
                    token_amounts: event.token_amounts,
                    fees: event.fees,
                    invariant: event.invariant,
                    token_supply: event.token_supply,
                });
                transaction.logs.push(create_log(log, event));
            }

            // RemoveLiquidity event
            if let Some(event) = curvefi::events::RemoveLiquidity::match_and_decode(log) {
                total_remove_liquidity += 1;
                let event = pb::log::Log::RemoveLiquidity(pb::RemoveLiquidity {
                    provider: event.provider,
                    token_amounts: event.token_amounts,
                    fees: event.fees,
                    token_supply: event.token_supply,
                });
                transaction.logs.push(create_log(log, event));
            }

            // RemoveLiquidityOne event
            if let Some(event) = curvefi::events::RemoveLiquidityOne::match_and_decode(log) {
                total_remove_liquidity_one += 1;
                let event = pb::log::Log::RemoveLiquidityOne(pb::RemoveLiquidityOne {
                    provider: event.provider,
                    token_amount: event.token_amount,
                    coin_amount: event.coin_amount,
                });
                transaction.logs.push(create_log(log, event));
            }

            // RemoveLiquidityImbalance event
            if let Some(event) = curvefi::events::RemoveLiquidityImbalance::match_and_decode(log) {
                total_remove_liquidity_imbalance += 1;
                let event = pb::log::Log::RemoveLiquidityImbalance(pb::RemoveLiquidityImbalance {
                    provider: event.provider,
                    token_amounts: event.token_amounts,
                    fees: event.fees,
                    invariant: event.invariant,
                    token_supply: event.token_supply,
                });
                transaction.logs.push(create_log(log, event));
            }

            // CommitNewAdmin event
            if let Some(event) = curvefi::events::CommitNewAdmin::match_and_decode(log) {
                total_commit_new_admin += 1;
                let event = pb::log::Log::CommitNewAdmin(pb::CommitNewAdmin {
                    deadline: event.deadline,
                    admin: event.admin,
                });
                transaction.logs.push(create_log(log, event));
            }

            // NewAdmin event
            if let Some(event) = curvefi::events::NewAdmin::match_and_decode(log) {
                total_new_admin += 1;
                let event = pb::log::Log::NewAdmin(pb::NewAdmin { admin: event.admin });
                transaction.logs.push(create_log(log, event));
            }

            // CommitNewFee event
            if let Some(event) = curvefi::events::CommitNewFee::match_and_decode(log) {
                total_commit_new_fee += 1;
                let event = pb::log::Log::CommitNewFee(pb::CommitNewFee {
                    deadline: event.deadline,
                    fee: event.fee,
                    admin_fee: event.admin_fee,
                });
                transaction.logs.push(create_log(log, event));
            }

            // NewFee event
            if let Some(event) = curvefi::events::NewFee::match_and_decode(log) {
                total_new_fee += 1;
                let event = pb::log::Log::NewFee(pb::NewFee {
                    fee: event.fee,
                    admin_fee: event.admin_fee,
                });
                transaction.logs.push(create_log(log, event));
            }

            // RampA event
            if let Some(event) = curvefi::events::RampA::match_and_decode(log) {
                total_ramp_a += 1;
                let event = pb::log::Log::RampA(pb::RampA {
                    old_a: event.old_a,
                    new_a: event.new_a,
                    initial_time: event.initial_time,
                    future_time: event.future_time,
                });
                transaction.logs.push(create_log(log, event));
            }

            // StopRampA event
            if let Some(event) = curvefi::events::StopRampA::match_and_decode(log) {
                total_stop_ramp_a += 1;
                let event = pb::log::Log::StopRampA(pb::StopRampA { a: event.a, t: event.t });
                transaction.logs.push(create_log(log, event));
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
    Ok(events)
}
