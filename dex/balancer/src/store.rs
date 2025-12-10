use proto::pb::balancer::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PoolRegistered ----
            if let Some(pb::log::Log::PoolRegistered(pool_registered)) = &log.log {
                let payload = pb::StorePool {
                    factory: pool_registered.factory.clone(),
                    token_config: pool_registered.token_config.clone(),
                };
                store.set(1, Hex::encode(&pool_registered.pool), &payload);
            }
        }
    }
}
