use prost::Message;
use prost_types::Any;
use proto::pb::dex::foundational_store::v1 as foundational;
use proto::pb::{
    aerodrome::v1 as aerodrome, balancer::v1 as balancer, bancor::v1 as bancor, curvefi::v1 as curvefi,
    kyber_elastic::v1 as kyber_elastic, sunpump::v1 as sunpump,
    traderjoe::v1 as traderjoe, uniswap,
};
use substreams::pb::sf::substreams::foundational_store::model::v2::{Entry, Key, SinkEntries};

// The foundational payload is intentionally restricted to shared pool metadata only:
// tokens[] plus factory when available. Protocol-specific initialization fields like
// Aerodrome `stable`, TraderJoe `bin_step`, and Kyber `swap_fee_units` /
// `tick_distance` remain available on the original pool-creation events.

#[substreams::handlers::map]
pub fn map_entries(
    events_sunpump: sunpump::Events,
    events_balancer: balancer::Events,
    events_bancor: bancor::Events,
    events_curvefi: curvefi::Events,
    events_aerodrome: aerodrome::Events,
    events_traderjoe: traderjoe::Events,
    events_kyber_elastic: kyber_elastic::Events,
    events_uniswap_v1: uniswap::v1::Events,
    events_uniswap_v2: uniswap::v2::Events,
    events_uniswap_v3: uniswap::v3::Events,
    events_uniswap_v4: uniswap::v4::Events,
) -> Result<SinkEntries, substreams::errors::Error> {
    let mut entries = Vec::new();

    collect_sunpump(&mut entries, &events_sunpump);
    collect_balancer(&mut entries, &events_balancer);
    collect_bancor(&mut entries, &events_bancor);
    collect_curvefi(&mut entries, &events_curvefi);
    collect_aerodrome(&mut entries, &events_aerodrome);
    collect_traderjoe(&mut entries, &events_traderjoe);
    collect_kyber_elastic(&mut entries, &events_kyber_elastic);
    collect_uniswap_v1(&mut entries, &events_uniswap_v1);
    collect_uniswap_v2(&mut entries, &events_uniswap_v2);
    collect_uniswap_v3(&mut entries, &events_uniswap_v3);
    collect_uniswap_v4(&mut entries, &events_uniswap_v4);

    if entries.is_empty() {
        return Ok(SinkEntries::default());
    }

    Ok(SinkEntries { entries, if_not_exist: true })
}

fn push_pool_entry(entries: &mut Vec<Entry>, pool: &[u8], tokens: Vec<Vec<u8>>, factory: Vec<u8>) {
    if pool.is_empty() {
        return;
    }

    let value = Any {
        type_url: "type.googleapis.com/dex.foundational_store.v1.Pool".to_string(),
        value: foundational::Pool { tokens, factory }.encode_to_vec(),
    };

    entries.push(Entry {
        key: Some(Key { bytes: pool.to_vec() }),
        value: Some(value),
    });
}

fn push_balancer_pool(entries: &mut Vec<Entry>, pool: &[u8], factory: &[u8], token_config: &[balancer::TokenConfig]) {
    push_pool_entry(
        entries,
        pool,
        token_config.iter().map(|config| config.token.clone()).collect(),
        factory.to_vec(),
    );
}

fn collect_aerodrome(entries: &mut Vec<Entry>, events: &aerodrome::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(aerodrome::log::Log::PoolCreated(pool_created)) = &log.log {
                push_pool_entry(
                    entries,
                    &pool_created.pool,
                    vec![pool_created.token0.clone(), pool_created.token1.clone()],
                    log.address.clone(),
                );
            }
        }
    }
}

fn collect_balancer(entries: &mut Vec<Entry>, events: &balancer::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(balancer::log::Log::PoolRegistered(pool_registered)) = &log.log {
                push_balancer_pool(entries, &pool_registered.pool, &pool_registered.factory, &pool_registered.token_config);
            }
        }
    }
}

fn collect_bancor(entries: &mut Vec<Entry>, events: &bancor::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            match &log.log {
                Some(bancor::log::Log::FeaturesAddition(event)) => {
                    push_pool_entry(entries, &event.address, vec![], log.address.clone());
                }
                Some(bancor::log::Log::NewConverter(event)) => {
                    push_pool_entry(entries, &event.converter, vec![], log.address.clone());
                }
                _ => {}
            }
        }
    }
}

fn collect_curvefi(entries: &mut Vec<Entry>, events: &curvefi::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            match &log.log {
                Some(curvefi::log::Log::Init(init)) => {
                    // CurveFi exposes pool constituents as `coins`; foundational consumers read them
                    // through the shared `tokens[]` field in the normalized payload.
                    push_pool_entry(entries, &init.address, init.coins.clone(), vec![]);
                }
                Some(curvefi::log::Log::PlainPoolDeployed(event)) => {
                    push_pool_entry(entries, &event.address, event.coins.clone(), log.address.clone());
                }
                Some(curvefi::log::Log::MetaPoolDeployed(event)) => {
                    push_pool_entry(entries, &event.address, vec![event.coin.clone()], log.address.clone());
                }
                Some(curvefi::log::Log::CryptoPoolDeployed(event)) => {
                    push_pool_entry(entries, &event.address, event.coins.clone(), log.address.clone());
                }
                _ => {}
            }
        }
    }
}

fn collect_kyber_elastic(entries: &mut Vec<Entry>, events: &kyber_elastic::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(kyber_elastic::log::Log::PoolCreated(pool_created)) = &log.log {
                push_pool_entry(
                    entries,
                    &pool_created.pool,
                    vec![pool_created.token0.clone(), pool_created.token1.clone()],
                    log.address.clone(),
                );
            }
        }
    }
}

fn collect_sunpump(entries: &mut Vec<Entry>, events: &sunpump::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            match &log.log {
                Some(sunpump::log::Log::TokenCreate(event)) => {
                    push_pool_entry(entries, &event.token_address, vec![], log.address.clone());
                }
                Some(sunpump::log::Log::TokenCreateLegacy(event)) => {
                    push_pool_entry(entries, &event.token_address, vec![], log.address.clone());
                }
                _ => {}
            }
        }
    }
}

fn collect_traderjoe(entries: &mut Vec<Entry>, events: &traderjoe::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(traderjoe::log::Log::LbPairCreated(pair_created)) = &log.log {
                push_pool_entry(
                    entries,
                    &pair_created.lb_pair,
                    vec![pair_created.token_x.clone(), pair_created.token_y.clone()],
                    log.address.clone(),
                );
            }
        }
    }
}

fn collect_uniswap_v1(entries: &mut Vec<Entry>, events: &uniswap::v1::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(uniswap::v1::log::Log::NewExchange(new_exchange)) = &log.log {
                push_pool_entry(entries, &new_exchange.exchange, vec![new_exchange.token.clone()], log.address.clone());
            }
        }
    }
}

fn collect_uniswap_v2(entries: &mut Vec<Entry>, events: &uniswap::v2::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(uniswap::v2::log::Log::PairCreated(pair_created)) = &log.log {
                push_pool_entry(
                    entries,
                    &pair_created.pair,
                    vec![pair_created.token0.clone(), pair_created.token1.clone()],
                    log.address.clone(),
                );
            }
        }
    }
}

fn collect_uniswap_v3(entries: &mut Vec<Entry>, events: &uniswap::v3::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(uniswap::v3::log::Log::PoolCreated(pool_created)) = &log.log {
                push_pool_entry(
                    entries,
                    &pool_created.pool,
                    vec![pool_created.token0.clone(), pool_created.token1.clone()],
                    log.address.clone(),
                );
            }
        }
    }
}

fn collect_uniswap_v4(entries: &mut Vec<Entry>, events: &uniswap::v4::Events) {
    for trx in &events.transactions {
        for log in &trx.logs {
            if let Some(uniswap::v4::log::Log::Initialize(initialize)) = &log.log {
                push_pool_entry(
                    entries,
                    &initialize.id,
                    vec![initialize.currency0.clone(), initialize.currency1.clone()],
                    log.address.clone(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_pool(entry: &Entry) -> foundational::Pool {
        let value = entry.value.as_ref().unwrap();
        assert_eq!(value.type_url, "type.googleapis.com/dex.foundational_store.v1.Pool");
        foundational::Pool::decode(value.value.as_slice()).unwrap()
    }

    #[test]
    fn stores_single_token_without_factory() {
        let mut entries = Vec::new();
        push_pool_entry(&mut entries, &[0x10], vec![vec![0xaa]], vec![]);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].key.as_ref().unwrap().bytes, vec![0x10]);

        let pool = decode_pool(&entries[0]);
        assert_eq!(pool.tokens, vec![vec![0xaa]]);
        assert!(pool.factory.is_empty());
    }

    #[test]
    fn extracts_balancer_tokens_from_token_config() {
        let mut entries = Vec::new();
        push_balancer_pool(
            &mut entries,
            &[0x01],
            &[0x02],
            &[
                balancer::TokenConfig { token: vec![0xaa], token_type: 0, rate_provider: vec![], pays_yield_fees: false },
                balancer::TokenConfig { token: vec![0xbb], token_type: 0, rate_provider: vec![], pays_yield_fees: false },
            ],
        );

        let pool = decode_pool(&entries[0]);
        assert_eq!(pool.tokens, vec![vec![0xaa], vec![0xbb]]);
        assert_eq!(pool.factory, vec![0x02]);
    }
}
