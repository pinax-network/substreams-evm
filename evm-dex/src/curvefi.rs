use common::clickhouse::{log_key, set_clock, set_template_call, set_template_log, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::curvefi::v1::{self as curvefi};
use substreams::{pb::substreams::Clock, store::FoundationalStore};
use substreams_database_change::tables::Tables;

use crate::store::{get_pool_by_address, tokens_csv, PoolMetadata};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &curvefi::Events, store: &FoundationalStore) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                // ── Pool / StableSwap (shared topic hashes) ──────────────────────────
                Some(curvefi::log::Log::TokenExchange(event)) => {
                    process_token_exchange(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::AddLiquidity(event)) => {
                    process_add_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidity(event)) => {
                    process_remove_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidityOne(event)) => {
                    process_remove_liquidity_one(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidityImbalance(event)) => {
                    process_remove_liquidity_imbalance(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CommitNewAdmin(event)) => {
                    process_commit_new_admin(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::NewAdmin(event)) => {
                    process_new_admin(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CommitNewFee(event)) => {
                    process_commit_new_fee(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::NewFee(event)) => {
                    process_new_fee(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RampA(event)) => {
                    process_ramp_a(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::StopRampA(event)) => {
                    process_stop_ramp_a(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                // ── Factory ──────────────────────────────────────────────────────────
                Some(curvefi::log::Log::PlainPoolDeployed(event)) => {
                    process_plain_pool_deployed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::MetaPoolDeployed(event)) => {
                    process_meta_pool_deployed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::BasePoolAdded(event)) => {
                    process_base_pool_added(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::LiquidityGaugeDeployed(event)) => {
                    process_liquidity_gauge_deployed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                // ── Direct pool deployment (constructor calldata) ──────────────────
                Some(curvefi::log::Log::Init(event)) => {
                    process_init(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                // ── CryptoSwap ───────────────────────────────────────────────────────
                Some(curvefi::log::Log::CryptoswapTokenExchange(event)) => {
                    process_cryptoswap_token_exchange(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapAddLiquidity(event)) => {
                    process_cryptoswap_add_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapRemoveLiquidity(event)) => {
                    process_cryptoswap_remove_liquidity(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapRemoveLiquidityOne(event)) => {
                    process_cryptoswap_remove_liquidity_one(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapClaimAdminFee(event)) => {
                    process_cryptoswap_claim_admin_fee(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapCommitNewParameters(event)) => {
                    process_cryptoswap_commit_new_parameters(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapNewParameters(event)) => {
                    process_cryptoswap_new_parameters(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapRampAgamma(event)) => {
                    process_cryptoswap_ramp_agamma(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapStopRampA(event)) => {
                    process_cryptoswap_stop_ramp_a(encoding, store, tables, clock, tx, log, tx_index, log_index, event);
                }
                // ── CryptoSwapFactory ────────────────────────────────────────────────
                Some(curvefi::log::Log::CryptoPoolDeployed(event)) => {
                    process_crypto_pool_deployed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapfactoryLiquidityGaugeDeployed(event)) => {
                    process_cryptoswapfactory_liquidity_gauge_deployed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapfactoryTransferOwnership(event)) => {
                    process_cryptoswapfactory_transfer_ownership(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapfactoryUpdateFeeReceiver(event)) => {
                    process_cryptoswapfactory_update_fee_receiver(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapfactoryUpdateGaugeImplementation(event)) => {
                    process_cryptoswapfactory_update_gauge_implementation(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapfactoryUpdatePoolImplementation(event)) => {
                    process_cryptoswapfactory_update_pool_implementation(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::CryptoswapfactoryUpdateTokenImplementation(event)) => {
                    process_cryptoswapfactory_update_token_implementation(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: PoolMetadata, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("coins", tokens_csv(encoding, &value));
}

fn parse_coin(encoding: &Encoding, id: String, coins: &[Vec<u8>]) -> Option<String> {
    if let Ok(index) = id.parse::<usize>() {
        return coins.get(index).map(|c| bytes_to_string(c, encoding));
    }
    None
}

// ── Pool / StableSwap handlers ────────────────────────────────────────────────

fn process_token_exchange(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::TokenExchange,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let sold_token = parse_coin(encoding, event.sold_id.clone(), &pool.tokens);
        let bought_token = parse_coin(encoding, event.bought_id.clone(), &pool.tokens);
        if sold_token.is_none() || bought_token.is_none() {
            return;
        }

        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_token_exchange", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("buyer", bytes_to_string(&event.buyer, encoding));
        row.set("sold_id", &event.sold_id);
        row.set("sold_amount", &event.tokens_sold);
        row.set("sold_token", sold_token.unwrap());
        row.set("bought_id", &event.bought_id);
        row.set("bought_amount", &event.tokens_bought);
        row.set("bought_token", bought_token.unwrap());
    }
}

fn process_add_liquidity(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::AddLiquidity,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_add_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("fees", event.fees.join(","));
        row.set("invariant", &event.invariant);
        row.set("token_supply", &event.token_supply);
    }
}

fn process_remove_liquidity(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidity,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_remove_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("fees", event.fees.join(","));
        row.set("token_supply", &event.token_supply);
    }
}

fn process_remove_liquidity_one(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityOne,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_remove_liquidity_one", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amount", &event.token_amount);
        row.set("coin_amount", &event.coin_amount);
    }
}

fn process_remove_liquidity_imbalance(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityImbalance,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_remove_liquidity_imbalance", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("fees", event.fees.join(","));
        row.set("invariant", &event.invariant);
        row.set("token_supply", &event.token_supply);
    }
}

fn process_commit_new_admin(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CommitNewAdmin,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_commit_new_admin", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    // pool lookup is best-effort; many contracts emit this without being a tracked pool
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        set_pool(encoding, pool, row);
    } else {
        row.set("factory", "");
        row.set("coins", "");
    }

    row.set("deadline", &event.deadline);
    row.set("admin", bytes_to_string(&event.admin, encoding));
}

fn process_new_admin(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::NewAdmin,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_new_admin", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    if let Some(pool) = get_pool_by_address(store, &log.address) {
        set_pool(encoding, pool, row);
    } else {
        row.set("factory", "");
        row.set("coins", "");
    }

    row.set("admin", bytes_to_string(&event.admin, encoding));
}

fn process_commit_new_fee(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CommitNewFee,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_commit_new_fee", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("deadline", &event.deadline);
        row.set("fee", &event.fee);
        row.set("admin_fee", &event.admin_fee);
    }
}

fn process_new_fee(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::NewFee,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_new_fee", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("fee", &event.fee);
        row.set("admin_fee", &event.admin_fee);
    }
}

fn process_ramp_a(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RampA,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_ramp_a", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("old_a", &event.old_a);
        row.set("new_a", &event.new_a);
        row.set("initial_time", &event.initial_time);
        row.set("future_time", &event.future_time);
    }
}

fn process_stop_ramp_a(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::StopRampA,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_stop_ramp_a", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("a", &event.a);
        row.set("t", &event.t);
    }
}

// ── Factory handlers ──────────────────────────────────────────────────────────

fn process_plain_pool_deployed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::PlainPoolDeployed,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_plain_pool_deployed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("coins", event.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
    row.set("a", &event.a);
    row.set("fee", &event.fee);
    row.set("deployer", bytes_to_string(&event.deployer, encoding));
}

fn process_meta_pool_deployed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::MetaPoolDeployed,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_meta_pool_deployed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("coin", bytes_to_string(&event.coin, encoding));
    row.set("base_pool", bytes_to_string(&event.base_pool, encoding));
    row.set("a", &event.a);
    row.set("fee", &event.fee);
    row.set("deployer", bytes_to_string(&event.deployer, encoding));
}

fn process_base_pool_added(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::BasePoolAdded,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_base_pool_added", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("base_pool", bytes_to_string(&event.base_pool, encoding));
}

fn process_liquidity_gauge_deployed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::LiquidityGaugeDeployed,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_liquidity_gauge_deployed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("gauge", bytes_to_string(&event.gauge, encoding));
}

fn process_init(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::Init,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_pool_init", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("owner", bytes_to_string(&event.owner, encoding));
    row.set("coins", event.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
    row.set("pool_token", bytes_to_string(&event.pool_token, encoding));
    row.set("a", &event.a);
    row.set("fee", &event.fee);
    row.set("admin_fee", &event.admin_fee);
}

// ── CryptoSwap handlers ───────────────────────────────────────────────────────

fn process_cryptoswap_token_exchange(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapTokenExchange,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let sold_token = parse_coin(encoding, event.sold_id.clone(), &pool.tokens);
        let bought_token = parse_coin(encoding, event.bought_id.clone(), &pool.tokens);
        if sold_token.is_none() || bought_token.is_none() {
            return;
        }

        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_token_exchange", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("buyer", bytes_to_string(&event.buyer, encoding));
        row.set("sold_id", &event.sold_id);
        row.set("sold_amount", &event.tokens_sold);
        row.set("sold_token", sold_token.unwrap());
        row.set("bought_id", &event.bought_id);
        row.set("bought_amount", &event.tokens_bought);
        row.set("bought_token", bought_token.unwrap());
    }
}

fn process_cryptoswap_add_liquidity(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapAddLiquidity,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_add_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("fee", &event.fee);
        row.set("token_supply", &event.token_supply);
    }
}

fn process_cryptoswap_remove_liquidity(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapRemoveLiquidity,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_remove_liquidity", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amounts", event.token_amounts.join(","));
        row.set("token_supply", &event.token_supply);
    }
}

fn process_cryptoswap_remove_liquidity_one(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapRemoveLiquidityOne,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_remove_liquidity_one", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("provider", bytes_to_string(&event.provider, encoding));
        row.set("token_amount", &event.token_amount);
        row.set("coin_index", &event.coin_index);
        row.set("coin_amount", &event.coin_amount);
    }
}

fn process_cryptoswap_claim_admin_fee(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapClaimAdminFee,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_claim_admin_fee", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("admin", bytes_to_string(&event.admin, encoding));
        row.set("tokens", &event.tokens);
    }
}

fn process_cryptoswap_commit_new_parameters(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapCommitNewParameters,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_commit_new_parameters", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("deadline", &event.deadline);
        row.set("admin_fee", &event.admin_fee);
        row.set("mid_fee", &event.mid_fee);
        row.set("out_fee", &event.out_fee);
        row.set("fee_gamma", &event.fee_gamma);
        row.set("allowed_extra_profit", &event.allowed_extra_profit);
        row.set("adjustment_step", &event.adjustment_step);
        row.set("ma_half_time", &event.ma_half_time);
    }
}

fn process_cryptoswap_new_parameters(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapNewParameters,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_new_parameters", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("admin_fee", &event.admin_fee);
        row.set("mid_fee", &event.mid_fee);
        row.set("out_fee", &event.out_fee);
        row.set("fee_gamma", &event.fee_gamma);
        row.set("allowed_extra_profit", &event.allowed_extra_profit);
        row.set("adjustment_step", &event.adjustment_step);
        row.set("ma_half_time", &event.ma_half_time);
    }
}

fn process_cryptoswap_ramp_agamma(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapRampAgamma,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_ramp_agamma", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("initial_a", &event.initial_a);
        row.set("future_a", &event.future_a);
        row.set("initial_gamma", &event.initial_gamma);
        row.set("future_gamma", &event.future_gamma);
        row.set("initial_time", &event.initial_time);
        row.set("future_time", &event.future_time);
    }
}

fn process_cryptoswap_stop_ramp_a(
    encoding: &Encoding,
    store: &FoundationalStore,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapStopRampA,
) {
    if let Some(pool) = get_pool_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("curvefi_cryptoswap_stop_ramp_a", key);

        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_template_call(encoding, log, row);
        set_pool(encoding, pool, row);

        row.set("current_a", &event.current_a);
        row.set("current_gamma", &event.current_gamma);
        row.set("time", &event.time);
    }
}

// ── CryptoSwapFactory handlers ────────────────────────────────────────────────

fn process_crypto_pool_deployed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoPoolDeployed,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_cryptoswapfactory_crypto_pool_deployed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("address", bytes_to_string(&event.address, encoding));
    row.set("token", bytes_to_string(&event.token, encoding));
    row.set("coins", event.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
    row.set("a", &event.a);
    row.set("gamma", &event.gamma);
    row.set("mid_fee", &event.mid_fee);
    row.set("out_fee", &event.out_fee);
    row.set("allowed_extra_profit", &event.allowed_extra_profit);
    row.set("fee_gamma", &event.fee_gamma);
    row.set("adjustment_step", &event.adjustment_step);
    row.set("admin_fee", &event.admin_fee);
    row.set("ma_half_time", &event.ma_half_time);
    row.set("initial_price", &event.initial_price);
    row.set("deployer", bytes_to_string(&event.deployer, encoding));
}

fn process_cryptoswapfactory_liquidity_gauge_deployed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapFactoryLiquidityGaugeDeployed,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_cryptoswapfactory_liquidity_gauge_deployed", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("pool", bytes_to_string(&event.pool, encoding));
    row.set("token", bytes_to_string(&event.token, encoding));
    row.set("gauge", bytes_to_string(&event.gauge, encoding));
}

fn process_cryptoswapfactory_transfer_ownership(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapFactoryTransferOwnership,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_cryptoswapfactory_transfer_ownership", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("old_owner", bytes_to_string(&event.old_owner, encoding));
    row.set("new_owner", bytes_to_string(&event.new_owner, encoding));
}

fn process_cryptoswapfactory_update_fee_receiver(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapFactoryUpdateFeeReceiver,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_cryptoswapfactory_update_fee_receiver", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("old_fee_receiver", bytes_to_string(&event.old_fee_receiver, encoding));
    row.set("new_fee_receiver", bytes_to_string(&event.new_fee_receiver, encoding));
}

fn process_cryptoswapfactory_update_gauge_implementation(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapFactoryUpdateGaugeImplementation,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_cryptoswapfactory_update_gauge_implementation", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("old_gauge_implementation", bytes_to_string(&event.old_gauge_implementation, encoding));
    row.set("new_gauge_implementation", bytes_to_string(&event.new_gauge_implementation, encoding));
}

fn process_cryptoswapfactory_update_pool_implementation(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapFactoryUpdatePoolImplementation,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_cryptoswapfactory_update_pool_implementation", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("old_pool_implementation", bytes_to_string(&event.old_pool_implementation, encoding));
    row.set("new_pool_implementation", bytes_to_string(&event.new_pool_implementation, encoding));
}

fn process_cryptoswapfactory_update_token_implementation(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::CryptoSwapFactoryUpdateTokenImplementation,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_cryptoswapfactory_update_token_implementation", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);

    row.set("old_token_implementation", bytes_to_string(&event.old_token_implementation, encoding));
    row.set("new_token_implementation", bytes_to_string(&event.new_token_implementation, encoding));
}
