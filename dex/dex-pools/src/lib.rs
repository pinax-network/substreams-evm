use proto::pb::{aerodrome, balancer, bancor, cow, curvefi, dodo, kyber_elastic, sunpump, traderjoe, uniswap, woofi};
use proto::pb::dex::v1 as dex;
use substreams::Hex;
use substreams::errors::Error;
use substreams::prelude::*;

// ── map_dex_pools ────────────────────────────────────────────────────────────

/// Aggregates pool creation / initialization events from all supported DEX
/// protocols into a single unified `DexPools` output.
///
/// Each `DexPool` carries:
/// - `address`  : pool contract address (store key)
/// - `protocol` : human-readable protocol identifier
/// - `factory`  : factory contract that deployed the pool (may be empty)
/// - `coins`    : ordered token array (generalizes token0/token1 for any N)
#[substreams::handlers::map]
pub fn map_dex_pools(
    // Tron DEX
    events_sunpump: sunpump::v1::Events,
    // Ethereum DEX
    events_balancer: balancer::v1::Events,
    events_bancor: bancor::v1::Events,
    events_cow: cow::v1::Events,
    events_curvefi: curvefi::v1::Events,
    // New DEX
    events_aerodrome: aerodrome::v1::Events,
    events_dodo: dodo::v1::Events,
    events_woofi: woofi::v1::Events,
    events_traderjoe: traderjoe::v1::Events,
    events_kyber_elastic: kyber_elastic::v1::Events,
    // Uniswap DEX
    events_uniswap_v1: uniswap::v1::Events,
    events_uniswap_v2: uniswap::v2::Events,
    events_uniswap_v3: uniswap::v3::Events,
    events_uniswap_v4: uniswap::v4::Events,
) -> Result<dex::DexPools, Error> {
    let mut pools = dex::DexPools::default();

    extract_sunpump_pools(&mut pools.pools, &events_sunpump);
    extract_balancer_pools(&mut pools.pools, &events_balancer);
    extract_bancor_pools(&mut pools.pools, &events_bancor);
    // CoW Protocol has no pool factory
    let _ = events_cow;
    extract_curvefi_pools(&mut pools.pools, &events_curvefi);
    extract_aerodrome_pools(&mut pools.pools, &events_aerodrome);
    // DODO and WooFi have no pool factory
    let _ = events_dodo;
    let _ = events_woofi;
    extract_traderjoe_pools(&mut pools.pools, &events_traderjoe);
    extract_kyber_elastic_pools(&mut pools.pools, &events_kyber_elastic);
    extract_uniswap_v1_pools(&mut pools.pools, &events_uniswap_v1);
    extract_uniswap_v2_pools(&mut pools.pools, &events_uniswap_v2);
    extract_uniswap_v3_pools(&mut pools.pools, &events_uniswap_v3);
    extract_uniswap_v4_pools(&mut pools.pools, &events_uniswap_v4);

    Ok(pools)
}

// ── store_dex_pools ──────────────────────────────────────────────────────────

/// Stores every `DexPool` keyed by its pool address (lowercase hex, no 0x
/// prefix) so downstream swap modules can look up token addresses quickly.
#[substreams::handlers::store]
pub fn store_dex_pools(pools: dex::DexPools, store: StoreSetProto<dex::DexPool>) {
    for pool in pools.pools {
        let key = Hex::encode(&pool.address);
        store.set(pool.log_ordinal, key, &pool);
    }
}

// ── Protocol extractors ──────────────────────────────────────────────────────

fn make_pool(tx: &[u8], log_index: u32, log_ordinal: u64, address: Vec<u8>, protocol: &str, factory: Vec<u8>, coins: Vec<Vec<u8>>) -> dex::DexPool {
    dex::DexPool {
        address,
        protocol: protocol.to_string(),
        factory,
        coins,
        tx_hash: tx.to_vec(),
        log_index,
        log_ordinal,
    }
}

fn extract_sunpump_pools(out: &mut Vec<dex::DexPool>, events: &sunpump::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(sunpump::v1::log::Log::TokenCreate(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.token_address.clone(),
                    "sunpump",
                    log.address.clone(), // factory = SunPump contract
                    vec![event.token_address.clone()], // single token pool
                ));
            } else if let Some(sunpump::v1::log::Log::TokenCreateLegacy(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.token_address.clone(),
                    "sunpump",
                    log.address.clone(),
                    vec![event.token_address.clone()],
                ));
            }
        }
    }
}

fn extract_balancer_pools(out: &mut Vec<dex::DexPool>, events: &balancer::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(balancer::v1::log::Log::PoolRegistered(event)) = &log.log {
                let coins: Vec<Vec<u8>> = event.token_config.iter().map(|tc| tc.token.clone()).collect();
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.pool.clone(),
                    "balancer",
                    event.factory.clone(),
                    coins,
                ));
            }
        }
    }
}

fn extract_bancor_pools(out: &mut Vec<dex::DexPool>, events: &bancor::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(bancor::v1::log::Log::NewConverter(event)) = &log.log {
                // Bancor converters don't have explicit token lists in the event
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.converter.clone(),
                    "bancor",
                    log.address.clone(), // ContractRegistry is the factory
                    vec![], // tokens populated by Conversion events
                ));
            }
        }
    }
}

fn extract_curvefi_pools(out: &mut Vec<dex::DexPool>, events: &curvefi::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            match &log.log {
                Some(curvefi::v1::log::Log::PlainPoolDeployed(event)) => {
                    out.push(make_pool(
                        &trx.hash,
                        log_index as u32,
                        log.ordinal,
                        event.address.clone(),
                        "curvefi",
                        log.address.clone(),
                        event.coins.clone(),
                    ));
                }
                Some(curvefi::v1::log::Log::MetaPoolDeployed(event)) => {
                    out.push(make_pool(
                        &trx.hash,
                        log_index as u32,
                        log.ordinal,
                        event.address.clone(),
                        "curvefi",
                        log.address.clone(),
                        vec![event.coin.clone()],
                    ));
                }
                Some(curvefi::v1::log::Log::CryptoPoolDeployed(event)) => {
                    out.push(make_pool(
                        &trx.hash,
                        log_index as u32,
                        log.ordinal,
                        event.address.clone(),
                        "curvefi",
                        log.address.clone(),
                        event.coins.clone(),
                    ));
                }
                Some(curvefi::v1::log::Log::Init(event)) => {
                    // Direct pool deployment (non-factory)
                    out.push(make_pool(
                        &trx.hash,
                        log_index as u32,
                        log.ordinal,
                        event.address.clone(),
                        "curvefi",
                        vec![], // no factory for direct deployments
                        event.coins.clone(),
                    ));
                }
                _ => {}
            }
        }
    }
}

fn extract_aerodrome_pools(out: &mut Vec<dex::DexPool>, events: &aerodrome::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(aerodrome::v1::log::Log::PoolCreated(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.pool.clone(),
                    "aerodrome",
                    log.address.clone(),
                    vec![event.token0.clone(), event.token1.clone()],
                ));
            }
        }
    }
}

fn extract_traderjoe_pools(out: &mut Vec<dex::DexPool>, events: &traderjoe::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(traderjoe::v1::log::Log::LbPairCreated(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.lb_pair.clone(),
                    "traderjoe",
                    log.address.clone(),
                    vec![event.token_x.clone(), event.token_y.clone()],
                ));
            }
        }
    }
}

fn extract_kyber_elastic_pools(out: &mut Vec<dex::DexPool>, events: &kyber_elastic::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(kyber_elastic::v1::log::Log::PoolCreated(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.pool.clone(),
                    "kyber_elastic",
                    log.address.clone(),
                    vec![event.token0.clone(), event.token1.clone()],
                ));
            }
        }
    }
}

fn extract_uniswap_v1_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v1::log::Log::NewExchange(event)) = &log.log {
                // V1 exchanges hold one ERC20 token; ETH is the other side
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.exchange.clone(),
                    "uniswap_v1",
                    log.address.clone(),
                    vec![event.token.clone()], // ETH is the implicit second coin
                ));
            }
        }
    }
}

fn extract_uniswap_v2_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v2::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v2::log::Log::PairCreated(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.pair.clone(),
                    "uniswap_v2",
                    log.address.clone(),
                    vec![event.token0.clone(), event.token1.clone()],
                ));
            }
        }
    }
}

fn extract_uniswap_v3_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v3::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v3::log::Log::PoolCreated(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.pool.clone(),
                    "uniswap_v3",
                    log.address.clone(),
                    vec![event.token0.clone(), event.token1.clone()],
                ));
            }
        }
    }
}

fn extract_uniswap_v4_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v4::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v4::log::Log::Initialize(event)) = &log.log {
                out.push(make_pool(
                    &trx.hash,
                    log_index as u32,
                    log.ordinal,
                    event.id.clone(),
                    "uniswap_v4",
                    log.address.clone(), // PoolManager is the factory
                    vec![event.currency0.clone(), event.currency1.clone()],
                ));
            }
        }
    }
}
