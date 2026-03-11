/// Pool extraction helpers used by the `map_dex_pools` handler.
/// Extracts pool creation / initialization events from each DEX protocol and
/// normalizes them into the unified `DexPool` format.
use proto::pb::{aerodrome, balancer, bancor, curvefi, kyber_elastic, sunpump, traderjoe, uniswap};
use proto::pb::dex::v1 as dex;

pub fn make_pool(tx_hash: &[u8], log_index: u32, log_ordinal: u64, address: Vec<u8>, protocol: &str, factory: Vec<u8>, coins: Vec<Vec<u8>>) -> dex::DexPool {
    dex::DexPool {
        address,
        protocol: protocol.to_string(),
        factory,
        coins,
        tx_hash: tx_hash.to_vec(),
        log_index,
        log_ordinal,
    }
}

pub fn extract_sunpump_pools(out: &mut Vec<dex::DexPool>, events: &sunpump::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(sunpump::v1::log::Log::TokenCreate(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.token_address.clone(), "sunpump", log.address.clone(), vec![event.token_address.clone()]));
            } else if let Some(sunpump::v1::log::Log::TokenCreateLegacy(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.token_address.clone(), "sunpump", log.address.clone(), vec![event.token_address.clone()]));
            }
        }
    }
}

pub fn extract_balancer_pools(out: &mut Vec<dex::DexPool>, events: &balancer::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(balancer::v1::log::Log::PoolRegistered(event)) = &log.log {
                let coins: Vec<Vec<u8>> = event.token_config.iter().map(|tc| tc.token.clone()).collect();
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.pool.clone(), "balancer", event.factory.clone(), coins));
            }
        }
    }
}

pub fn extract_bancor_pools(out: &mut Vec<dex::DexPool>, events: &bancor::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(bancor::v1::log::Log::NewConverter(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.converter.clone(), "bancor", log.address.clone(), vec![]));
            }
        }
    }
}

pub fn extract_curvefi_pools(out: &mut Vec<dex::DexPool>, events: &curvefi::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            match &log.log {
                Some(curvefi::v1::log::Log::PlainPoolDeployed(event)) => {
                    out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.address.clone(), "curvefi", log.address.clone(), event.coins.clone()));
                }
                Some(curvefi::v1::log::Log::MetaPoolDeployed(event)) => {
                    out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.address.clone(), "curvefi", log.address.clone(), vec![event.coin.clone()]));
                }
                Some(curvefi::v1::log::Log::CryptoPoolDeployed(event)) => {
                    out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.address.clone(), "curvefi", log.address.clone(), event.coins.clone()));
                }
                Some(curvefi::v1::log::Log::Init(event)) => {
                    out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.address.clone(), "curvefi", vec![], event.coins.clone()));
                }
                _ => {}
            }
        }
    }
}

pub fn extract_aerodrome_pools(out: &mut Vec<dex::DexPool>, events: &aerodrome::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(aerodrome::v1::log::Log::PoolCreated(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.pool.clone(), "aerodrome", log.address.clone(), vec![event.token0.clone(), event.token1.clone()]));
            }
        }
    }
}

pub fn extract_traderjoe_pools(out: &mut Vec<dex::DexPool>, events: &traderjoe::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(traderjoe::v1::log::Log::LbPairCreated(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.lb_pair.clone(), "traderjoe", log.address.clone(), vec![event.token_x.clone(), event.token_y.clone()]));
            }
        }
    }
}

pub fn extract_kyber_elastic_pools(out: &mut Vec<dex::DexPool>, events: &kyber_elastic::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(kyber_elastic::v1::log::Log::PoolCreated(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.pool.clone(), "kyber_elastic", log.address.clone(), vec![event.token0.clone(), event.token1.clone()]));
            }
        }
    }
}

pub fn extract_uniswap_v1_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v1::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v1::log::Log::NewExchange(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.exchange.clone(), "uniswap_v1", log.address.clone(), vec![event.token.clone()]));
            }
        }
    }
}

pub fn extract_uniswap_v2_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v2::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v2::log::Log::PairCreated(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.pair.clone(), "uniswap_v2", log.address.clone(), vec![event.token0.clone(), event.token1.clone()]));
            }
        }
    }
}

pub fn extract_uniswap_v3_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v3::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v3::log::Log::PoolCreated(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.pool.clone(), "uniswap_v3", log.address.clone(), vec![event.token0.clone(), event.token1.clone()]));
            }
        }
    }
}

pub fn extract_uniswap_v4_pools(out: &mut Vec<dex::DexPool>, events: &uniswap::v4::Events) {
    for trx in &events.transactions {
        for (log_index, log) in trx.logs.iter().enumerate() {
            if let Some(uniswap::v4::log::Log::Initialize(event)) = &log.log {
                out.push(make_pool(&trx.hash, log_index as u32, log.ordinal, event.id.clone(), "uniswap_v4", log.address.clone(), vec![event.currency0.clone(), event.currency1.clone()]));
            }
        }
    }
}
