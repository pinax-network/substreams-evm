use proto::pb::curvefi::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
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
        }
    }
}
