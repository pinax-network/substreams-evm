use proto::pb::curvefi::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- Init (direct deployment, no factory) ----
            if let Some(pb::log::Log::Init(init)) = &log.log {
                let payload = pb::StorePool {
                    factory: vec![],
                    coins: init.coins.clone(),
                };
                store.set(1, Hex::encode(&init.address), &payload);
            }
            // ---- PlainPoolDeployed ----
            if let Some(pb::log::Log::PlainPoolDeployed(plain_pool_deployed)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    coins: plain_pool_deployed.coins.clone(),
                };
                store.set(1, Hex::encode(&plain_pool_deployed.address), &payload);
            }
            // ---- MetaPoolDeployed ----
            if let Some(pb::log::Log::MetaPoolDeployed(meta_pool_deployed)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    coins: vec![meta_pool_deployed.coin.clone()],
                };
                store.set(1, Hex::encode(&meta_pool_deployed.address), &payload);
            }
            // ---- CryptoPoolDeployed (CryptoSwapFactory) ----
            if let Some(pb::log::Log::CryptoPoolDeployed(crypto_pool_deployed)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    coins: crypto_pool_deployed.coins.clone(),
                };
                store.set(1, Hex::encode(&crypto_pool_deployed.address), &payload);
            }
        }
    }
}
