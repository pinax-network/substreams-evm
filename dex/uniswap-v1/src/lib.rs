use proto::pb::evm::uniswap::v1 as pb;
use substreams_abis::evm::uniswap::v1 as uniswap;
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
    let mut total_token_purchases = 0;
    let mut total_eth_purchases = 0;
    let mut total_add_liquidity = 0;
    let mut total_remove_liquidity = 0;
    let mut total_transfers = 0;
    let mut total_approvals = 0;
    let mut total_new_exchanges = 0;

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

            // TokenPurchase event
            if let Some(event) = uniswap::exchange::events::TokenPurchase::match_and_decode(log) {
                total_token_purchases += 1;
                let event = pb::log::Log::TokenPurchase(pb::TokenPurchase {
                    buyer: event.buyer.to_vec(),
                    eth_sold: event.eth_sold.to_string(),
                    tokens_bought: event.tokens_bought.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // EthPurchase event
            if let Some(event) = uniswap::exchange::events::EthPurchase::match_and_decode(log) {
                total_eth_purchases += 1;
                let event = pb::log::Log::EthPurchase(pb::EthPurchase {
                    buyer: event.buyer.to_vec(),
                    tokens_sold: event.tokens_sold.to_string(),
                    eth_bought: event.eth_bought.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // AddLiquidity event
            if let Some(event) = uniswap::exchange::events::AddLiquidity::match_and_decode(log) {
                total_add_liquidity += 1;
                let event = pb::log::Log::AddLiquidity(pb::AddLiquidity {
                    provider: event.provider.to_vec(),
                    eth_amount: event.eth_amount.to_string(),
                    token_amount: event.token_amount.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // RemoveLiquidity event
            if let Some(event) = uniswap::exchange::events::RemoveLiquidity::match_and_decode(log) {
                total_remove_liquidity += 1;
                let event = pb::log::Log::RemoveLiquidity(pb::RemoveLiquidity {
                    provider: event.provider.to_vec(),
                    eth_amount: event.eth_amount.to_string(),
                    token_amount: event.token_amount.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // Transfer event
            if let Some(event) = uniswap::exchange::events::Transfer::match_and_decode(log) {
                total_transfers += 1;
                let event = pb::log::Log::Transfer(pb::Transfer {
                    from: event.from.to_vec(),
                    to: event.to.to_vec(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // Approval event
            if let Some(event) = uniswap::exchange::events::Approval::match_and_decode(log) {
                total_approvals += 1;
                let event = pb::log::Log::Approval(pb::Approval {
                    owner: event.owner.to_vec(),
                    spender: event.spender.to_vec(),
                    value: event.value.to_string(),
                });
                transaction.logs.push(create_log(log, event));
            }

            // NewExchange event
            if let Some(event) = uniswap::factory::events::NewExchange::match_and_decode(log) {
                total_new_exchanges += 1;
                let event = pb::log::Log::NewExchange(pb::NewExchange {
                    factory: log.address.to_vec(),
                    token: event.token.to_vec(),
                    exchange: event.exchange.to_vec(),
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
    substreams::log::info!("Total TokenPurchase events: {}", total_token_purchases);
    substreams::log::info!("Total EthPurchase events: {}", total_eth_purchases);
    substreams::log::info!("Total AddLiquidity events: {}", total_add_liquidity);
    substreams::log::info!("Total RemoveLiquidity events: {}", total_remove_liquidity);
    substreams::log::info!("Total Transfer events: {}", total_transfers);
    substreams::log::info!("Total Approval events: {}", total_approvals);
    substreams::log::info!("Total NewExchange events: {}", total_new_exchanges);
    Ok(events)
}
