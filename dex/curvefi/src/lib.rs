use common::create::{CreateLog, CreateSyntheticLog, CreateTransaction};
use proto::pb::curvefi::v1 as pb;
use substreams_abis::dex::curvefi;
use substreams_ethereum::pb::eth::v2::{Block, CallType, TransactionTrace};
use substreams_ethereum::Event;

const STABLESWAP_CONSTRUCTOR_INPUT_LEN: usize = 32 * 8;

fn get_create_address(trx: &TransactionTrace) -> Option<Vec<u8>> {
    for call in trx.calls.iter() {
        if call.call_type == CallType::Create as i32 {
            return Some(call.address.to_vec());
        }
    }
    None
}

fn try_decode_pool_init_constructor(
    input: &[u8],
) -> Option<curvefi::stableswap::constructor::Constructor> {
    // Direct deployments prepend init bytecode and append the ABI-encoded StableSwap
    // constructor args as a fixed-size tail (8 static slots). Decode only that tail
    // and round-trip it to ensure we matched the constructor payload exactly.
    let suffix = input.get(input.len().checked_sub(STABLESWAP_CONSTRUCTOR_INPUT_LEN)?..)?;
    let constructor = curvefi::stableswap::constructor::Constructor::decode(suffix).ok()?;
    (constructor.encode() == suffix).then_some(constructor)
}

/// Attempt to extract a CurveFi pool `Init` event from a direct (non-factory) deployment
/// transaction by decoding the StableSwap constructor calldata from the transaction input tail.
///
/// Returns `None` if the transaction is not a direct deployment or the calldata does not
/// match the expected CurveFi StableSwap constructor format.
/// On success, returns the decoded `Init` event together with a reference to the root
/// CREATE call so the caller can populate call metadata without a second iteration.
fn try_extract_pool_init<'a>(trx: &'a TransactionTrace) -> Option<(pb::Init, &'a substreams_ethereum::pb::eth::v2::Call)> {
    // Only process direct deployment transactions (to field is empty/null)
    if !trx.to.is_empty() {
        return None;
    }

    // Find the root CREATE call
    let create_call = trx.calls.iter().find(|c| c.call_type == CallType::Create as i32 && c.depth == 0)?;
    let address = create_call.address.to_vec();
    let constructor = try_decode_pool_init_constructor(&trx.input)?;
    let curvefi::stableswap::constructor::Constructor {
        owner,
        coins,
        pool_token,
        a,
        fee,
        admin_fee,
    } = constructor;

    Some((
        pb::Init {
            address,
            owner,
            coins: coins.into_iter().collect(),
            pool_token,
            a: a.to_string(),
            fee: fee.to_string(),
            admin_fee: admin_fee.to_string(),
        },
        create_call,
    ))
}

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();

    // Pool / StableSwap counters (shared topic hashes)
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
    // Factory counters
    let mut total_plain_pool_deployed = 0;
    let mut total_meta_pool_deployed = 0;
    let mut total_base_pool_added = 0;
    let mut total_liquidity_gauge_deployed = 0;
    // CryptoSwap counters
    let mut total_cryptoswap_token_exchange = 0;
    let mut total_cryptoswap_add_liquidity = 0;
    let mut total_cryptoswap_remove_liquidity = 0;
    let mut total_cryptoswap_remove_liquidity_one = 0;
    let mut total_cryptoswap_claim_admin_fee = 0;
    let mut total_cryptoswap_commit_new_parameters = 0;
    let mut total_cryptoswap_new_parameters = 0;
    let mut total_cryptoswap_ramp_agamma = 0;
    let mut total_cryptoswap_stop_ramp_a = 0;
    // CryptoSwapFactory counters
    let mut total_crypto_pool_deployed = 0;
    let mut total_cryptoswapfactory_liquidity_gauge_deployed = 0;
    let mut total_cryptoswapfactory_transfer_ownership = 0;
    let mut total_cryptoswapfactory_update_fee_receiver = 0;
    let mut total_cryptoswapfactory_update_gauge_implementation = 0;
    let mut total_cryptoswapfactory_update_pool_implementation = 0;
    let mut total_cryptoswapfactory_update_token_implementation = 0;
    // Direct pool deployment (constructor decoding)
    let mut total_pool_init = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        // ── Direct pool deployment: decode constructor calldata ───────────────
        if let Some((init, create_call)) = try_extract_pool_init(trx) {
            total_pool_init += 1;
            let init_address = init.address.clone();
            let log_entry = pb::Log::create_synthetic_log_with_call(&init_address, create_call.begin_ordinal, 0, pb::log::Log::Init(init), Some(create_call));
            transaction.logs.push(log_entry);
        }

        let logs_with_calls: Vec<(&substreams_ethereum::pb::eth::v2::Log, Option<&substreams_ethereum::pb::eth::v2::Call>)> = if trx.calls.is_empty() {
            trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
        } else {
            trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
        };

        for (log, call) in logs_with_calls {
            // ── Pool / StableSwap events (shared topic hashes across both contracts) ──
            // These decoders match on topic0 only; they will fire for any contract
            // that emits the identical event signature (Pool and StableSwap share all 11).

            if let Some(event) = curvefi::pool::events::TokenExchange::match_and_decode(log) {
                total_token_exchange += 1;
                let event = pb::log::Log::TokenExchange(pb::TokenExchange {
                    buyer: event.buyer,
                    sold_id: event.sold_id.to_string(),
                    tokens_sold: event.tokens_sold.to_string(),
                    bought_id: event.bought_id.to_string(),
                    tokens_bought: event.tokens_bought.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

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
                continue;
            }

            if let Some(event) = curvefi::pool::events::RemoveLiquidity::match_and_decode(log) {
                total_remove_liquidity += 1;
                let event = pb::log::Log::RemoveLiquidity(pb::RemoveLiquidity {
                    provider: event.provider,
                    token_amounts: event.token_amounts.iter().map(|amt| amt.to_string()).collect(),
                    fees: event.fees.iter().map(|fee| fee.to_string()).collect(),
                    token_supply: event.token_supply.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::pool::events::RemoveLiquidityOne::match_and_decode(log) {
                total_remove_liquidity_one += 1;
                let event = pb::log::Log::RemoveLiquidityOne(pb::RemoveLiquidityOne {
                    provider: event.provider,
                    token_amount: event.token_amount.to_string(),
                    coin_amount: event.coin_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

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
                continue;
            }

            // CommitNewAdmin — shared across Pool, StableSwap, CryptoSwap, MetaPoolRegistry
            if let Some(event) = curvefi::pool::events::CommitNewAdmin::match_and_decode(log) {
                total_commit_new_admin += 1;
                let event = pb::log::Log::CommitNewAdmin(pb::CommitNewAdmin {
                    deadline: event.deadline.to_string(),
                    admin: event.admin,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            // NewAdmin — shared across Pool, StableSwap, CryptoSwap, MetaPoolRegistry
            if let Some(event) = curvefi::pool::events::NewAdmin::match_and_decode(log) {
                total_new_admin += 1;
                let event = pb::log::Log::NewAdmin(pb::NewAdmin { admin: event.admin });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::pool::events::CommitNewFee::match_and_decode(log) {
                total_commit_new_fee += 1;
                let event = pb::log::Log::CommitNewFee(pb::CommitNewFee {
                    deadline: event.deadline.to_string(),
                    fee: event.fee.to_string(),
                    admin_fee: event.admin_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::pool::events::NewFee::match_and_decode(log) {
                total_new_fee += 1;
                let event = pb::log::Log::NewFee(pb::NewFee {
                    fee: event.fee.to_string(),
                    admin_fee: event.admin_fee.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::pool::events::RampA::match_and_decode(log) {
                total_ramp_a += 1;
                let event = pb::log::Log::RampA(pb::RampA {
                    old_a: event.old_a.to_string(),
                    new_a: event.new_a.to_string(),
                    initial_time: event.initial_time.to_string(),
                    future_time: event.future_time.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::pool::events::StopRampA::match_and_decode(log) {
                total_stop_ramp_a += 1;
                let event = pb::log::Log::StopRampA(pb::StopRampA {
                    a: event.a.to_string(),
                    t: event.t.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            // ── Factory events ─────────────────────────────────────────────────────────

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
                continue;
            }

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
                continue;
            }

            if let Some(event) = curvefi::factory::events::BasePoolAdded::match_and_decode(log) {
                total_base_pool_added += 1;
                let event = pb::log::Log::BasePoolAdded(pb::BasePoolAdded { base_pool: event.base_pool });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::factory::events::LiquidityGaugeDeployed::match_and_decode(log) {
                total_liquidity_gauge_deployed += 1;
                let event = pb::log::Log::LiquidityGaugeDeployed(pb::LiquidityGaugeDeployed {
                    pool: event.pool,
                    gauge: event.gauge,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            // ── CryptoSwap events (contract-specific: unique topic hashes) ─────────────

            if let Some(event) = curvefi::cryptoswap::events::TokenExchange::match_and_decode(log) {
                total_cryptoswap_token_exchange += 1;
                let event = pb::log::Log::CryptoswapTokenExchange(pb::CryptoSwapTokenExchange {
                    buyer: event.buyer,
                    sold_id: event.sold_id.to_string(),
                    tokens_sold: event.tokens_sold.to_string(),
                    bought_id: event.bought_id.to_string(),
                    tokens_bought: event.tokens_bought.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::AddLiquidity::match_and_decode(log) {
                total_cryptoswap_add_liquidity += 1;
                let event = pb::log::Log::CryptoswapAddLiquidity(pb::CryptoSwapAddLiquidity {
                    provider: event.provider,
                    token_amounts: event.token_amounts.iter().map(|amt| amt.to_string()).collect(),
                    fee: event.fee.to_string(),
                    token_supply: event.token_supply.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::RemoveLiquidity::match_and_decode(log) {
                total_cryptoswap_remove_liquidity += 1;
                let event = pb::log::Log::CryptoswapRemoveLiquidity(pb::CryptoSwapRemoveLiquidity {
                    provider: event.provider,
                    token_amounts: event.token_amounts.iter().map(|amt| amt.to_string()).collect(),
                    token_supply: event.token_supply.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::RemoveLiquidityOne::match_and_decode(log) {
                total_cryptoswap_remove_liquidity_one += 1;
                let event = pb::log::Log::CryptoswapRemoveLiquidityOne(pb::CryptoSwapRemoveLiquidityOne {
                    provider: event.provider,
                    token_amount: event.token_amount.to_string(),
                    coin_index: event.coin_index.to_string(),
                    coin_amount: event.coin_amount.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::ClaimAdminFee::match_and_decode(log) {
                total_cryptoswap_claim_admin_fee += 1;
                let event = pb::log::Log::CryptoswapClaimAdminFee(pb::CryptoSwapClaimAdminFee {
                    admin: event.admin,
                    tokens: event.tokens.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::CommitNewParameters::match_and_decode(log) {
                total_cryptoswap_commit_new_parameters += 1;
                let event = pb::log::Log::CryptoswapCommitNewParameters(pb::CryptoSwapCommitNewParameters {
                    deadline: event.deadline.to_string(),
                    admin_fee: event.admin_fee.to_string(),
                    mid_fee: event.mid_fee.to_string(),
                    out_fee: event.out_fee.to_string(),
                    fee_gamma: event.fee_gamma.to_string(),
                    allowed_extra_profit: event.allowed_extra_profit.to_string(),
                    adjustment_step: event.adjustment_step.to_string(),
                    ma_half_time: event.ma_half_time.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::NewParameters::match_and_decode(log) {
                total_cryptoswap_new_parameters += 1;
                let event = pb::log::Log::CryptoswapNewParameters(pb::CryptoSwapNewParameters {
                    admin_fee: event.admin_fee.to_string(),
                    mid_fee: event.mid_fee.to_string(),
                    out_fee: event.out_fee.to_string(),
                    fee_gamma: event.fee_gamma.to_string(),
                    allowed_extra_profit: event.allowed_extra_profit.to_string(),
                    adjustment_step: event.adjustment_step.to_string(),
                    ma_half_time: event.ma_half_time.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::RampAgamma::match_and_decode(log) {
                total_cryptoswap_ramp_agamma += 1;
                let event = pb::log::Log::CryptoswapRampAgamma(pb::CryptoSwapRampAgamma {
                    initial_a: event.initial_a.to_string(),
                    future_a: event.future_a.to_string(),
                    initial_gamma: event.initial_gamma.to_string(),
                    future_gamma: event.future_gamma.to_string(),
                    initial_time: event.initial_time.to_string(),
                    future_time: event.future_time.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswap::events::StopRampA::match_and_decode(log) {
                total_cryptoswap_stop_ramp_a += 1;
                let event = pb::log::Log::CryptoswapStopRampA(pb::CryptoSwapStopRampA {
                    current_a: event.current_a.to_string(),
                    current_gamma: event.current_gamma.to_string(),
                    time: event.time.to_string(),
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            // ── CryptoSwapFactory events ────────────────────────────────────────────────

            if let Some(event) = curvefi::cryptoswapfactory::events::CryptoPoolDeployed::match_and_decode(log) {
                if let Some(address) = get_create_address(trx) {
                    total_crypto_pool_deployed += 1;
                    let event = pb::log::Log::CryptoPoolDeployed(pb::CryptoPoolDeployed {
                        address,
                        token: event.token,
                        coins: event.coins.to_vec(),
                        a: event.a.to_string(),
                        gamma: event.gamma.to_string(),
                        mid_fee: event.mid_fee.to_string(),
                        out_fee: event.out_fee.to_string(),
                        allowed_extra_profit: event.allowed_extra_profit.to_string(),
                        fee_gamma: event.fee_gamma.to_string(),
                        adjustment_step: event.adjustment_step.to_string(),
                        admin_fee: event.admin_fee.to_string(),
                        ma_half_time: event.ma_half_time.to_string(),
                        initial_price: event.initial_price.to_string(),
                        deployer: event.deployer,
                    });
                    transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                }
                continue;
            }

            if let Some(event) = curvefi::cryptoswapfactory::events::LiquidityGaugeDeployed::match_and_decode(log) {
                total_cryptoswapfactory_liquidity_gauge_deployed += 1;
                let event = pb::log::Log::CryptoswapfactoryLiquidityGaugeDeployed(pb::CryptoSwapFactoryLiquidityGaugeDeployed {
                    pool: event.pool,
                    token: event.token,
                    gauge: event.gauge,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswapfactory::events::TransferOwnership::match_and_decode(log) {
                total_cryptoswapfactory_transfer_ownership += 1;
                let event = pb::log::Log::CryptoswapfactoryTransferOwnership(pb::CryptoSwapFactoryTransferOwnership {
                    old_owner: event.old_owner,
                    new_owner: event.new_owner,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswapfactory::events::UpdateFeeReceiver::match_and_decode(log) {
                total_cryptoswapfactory_update_fee_receiver += 1;
                let event = pb::log::Log::CryptoswapfactoryUpdateFeeReceiver(pb::CryptoSwapFactoryUpdateFeeReceiver {
                    old_fee_receiver: event.old_fee_receiver,
                    new_fee_receiver: event.new_fee_receiver,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswapfactory::events::UpdateGaugeImplementation::match_and_decode(log) {
                total_cryptoswapfactory_update_gauge_implementation += 1;
                let event = pb::log::Log::CryptoswapfactoryUpdateGaugeImplementation(pb::CryptoSwapFactoryUpdateGaugeImplementation {
                    old_gauge_implementation: event.old_gauge_implementation,
                    new_gauge_implementation: event.new_gauge_implementation,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswapfactory::events::UpdatePoolImplementation::match_and_decode(log) {
                total_cryptoswapfactory_update_pool_implementation += 1;
                let event = pb::log::Log::CryptoswapfactoryUpdatePoolImplementation(pb::CryptoSwapFactoryUpdatePoolImplementation {
                    old_pool_implementation: event.old_pool_implementation,
                    new_pool_implementation: event.new_pool_implementation,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }

            if let Some(event) = curvefi::cryptoswapfactory::events::UpdateTokenImplementation::match_and_decode(log) {
                total_cryptoswapfactory_update_token_implementation += 1;
                let event = pb::log::Log::CryptoswapfactoryUpdateTokenImplementation(pb::CryptoSwapFactoryUpdateTokenImplementation {
                    old_token_implementation: event.old_token_implementation,
                    new_token_implementation: event.new_token_implementation,
                });
                transaction.logs.push(pb::Log::create_log_with_call(log, event, call));
                continue;
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    // Pool / StableSwap
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
    // Factory
    substreams::log::info!("Total PlainPoolDeployed events: {}", total_plain_pool_deployed);
    substreams::log::info!("Total MetaPoolDeployed events: {}", total_meta_pool_deployed);
    substreams::log::info!("Total BasePoolAdded events: {}", total_base_pool_added);
    substreams::log::info!("Total LiquidityGaugeDeployed events: {}", total_liquidity_gauge_deployed);
    // CryptoSwap
    substreams::log::info!("Total CryptoSwap TokenExchange events: {}", total_cryptoswap_token_exchange);
    substreams::log::info!("Total CryptoSwap AddLiquidity events: {}", total_cryptoswap_add_liquidity);
    substreams::log::info!("Total CryptoSwap RemoveLiquidity events: {}", total_cryptoswap_remove_liquidity);
    substreams::log::info!("Total CryptoSwap RemoveLiquidityOne events: {}", total_cryptoswap_remove_liquidity_one);
    substreams::log::info!("Total CryptoSwap ClaimAdminFee events: {}", total_cryptoswap_claim_admin_fee);
    substreams::log::info!("Total CryptoSwap CommitNewParameters events: {}", total_cryptoswap_commit_new_parameters);
    substreams::log::info!("Total CryptoSwap NewParameters events: {}", total_cryptoswap_new_parameters);
    substreams::log::info!("Total CryptoSwap RampAgamma events: {}", total_cryptoswap_ramp_agamma);
    substreams::log::info!("Total CryptoSwap StopRampA events: {}", total_cryptoswap_stop_ramp_a);
    // CryptoSwapFactory
    substreams::log::info!("Total CryptoPoolDeployed events: {}", total_crypto_pool_deployed);
    substreams::log::info!(
        "Total CryptoSwapFactory LiquidityGaugeDeployed events: {}",
        total_cryptoswapfactory_liquidity_gauge_deployed
    );
    substreams::log::info!(
        "Total CryptoSwapFactory TransferOwnership events: {}",
        total_cryptoswapfactory_transfer_ownership
    );
    substreams::log::info!(
        "Total CryptoSwapFactory UpdateFeeReceiver events: {}",
        total_cryptoswapfactory_update_fee_receiver
    );
    substreams::log::info!(
        "Total CryptoSwapFactory UpdateGaugeImplementation events: {}",
        total_cryptoswapfactory_update_gauge_implementation
    );
    substreams::log::info!(
        "Total CryptoSwapFactory UpdatePoolImplementation events: {}",
        total_cryptoswapfactory_update_pool_implementation
    );
    substreams::log::info!(
        "Total CryptoSwapFactory UpdateTokenImplementation events: {}",
        total_cryptoswapfactory_update_token_implementation
    );
    // Direct pool deployment
    substreams::log::info!("Total Init (direct pool deployment) events: {}", total_pool_init);

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::try_extract_pool_init;
    use substreams::scalar::BigInt;
    use substreams_abis::dex::curvefi;
    use substreams_ethereum::pb::eth::v2::{Call, CallType, TransactionTrace};

    fn sample_stableswap_constructor() -> curvefi::stableswap::constructor::Constructor {
        curvefi::stableswap::constructor::Constructor {
            owner: vec![0x11; 20],
            coins: [vec![0x21; 20], vec![0x22; 20], vec![0x23; 20]],
            pool_token: vec![0x31; 20],
            a: BigInt::from(2000u64),
            fee: BigInt::from(4_000_000u64),
            admin_fee: BigInt::from(5_000_000_000u64),
        }
    }

    #[test]
    fn extracts_stableswap_init_from_create_input_tail() {
        let constructor = sample_stableswap_constructor();
        let constructor_input = constructor.encode();
        let mut transaction_input = vec![0x60, 0x60, 0x60, 0x40, 0x52];
        transaction_input.extend(constructor_input.clone());

        let trx = TransactionTrace {
            input: transaction_input,
            calls: vec![Call {
                call_type: CallType::Create as i32,
                depth: 0,
                address: vec![0xaa; 20],
                begin_ordinal: 42,
                input: vec![],
                ..Default::default()
            }],
            ..Default::default()
        };

        let (init, create_call) = try_extract_pool_init(&trx).expect("expected init event");

        assert_eq!(trx.input[..5], [0x60, 0x60, 0x60, 0x40, 0x52]);
        assert_eq!(trx.input[5..], constructor_input);
        assert_eq!(init.address, vec![0xaa; 20]);
        assert_eq!(init.owner, constructor.owner);
        assert_eq!(init.coins, constructor.coins.to_vec());
        assert_eq!(init.pool_token, constructor.pool_token);
        assert_eq!(init.a, "2000");
        assert_eq!(init.fee, "4000000");
        assert_eq!(init.admin_fee, "5000000000");
        assert_eq!(create_call.begin_ordinal, 42);
    }

    #[test]
    fn ignores_non_contract_creation_transactions() {
        let trx = TransactionTrace {
            to: vec![0xbb; 20],
            calls: vec![Call {
                call_type: CallType::Create as i32,
                depth: 0,
                input: sample_stableswap_constructor().encode(),
                ..Default::default()
            }],
            ..Default::default()
        };

        assert!(try_extract_pool_init(&trx).is_none());
    }

    #[test]
    fn ignores_create_calls_without_stableswap_constructor_tail() {
        let trx = TransactionTrace {
            input: vec![0x01, 0x02, 0x03],
            calls: vec![Call {
                call_type: CallType::Create as i32,
                depth: 0,
                input: vec![],
                ..Default::default()
            }],
            ..Default::default()
        };

        assert!(try_extract_pool_init(&trx).is_none());
    }
}
