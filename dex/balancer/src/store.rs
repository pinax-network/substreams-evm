use proto::pb::balancer::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PoolRegistered ----
            if let Some(pb::log::Log::PoolRegistered(pool_registered)) = &log.log {
                let payload = pb::StorePool { factory: log.address.clone() };
                store.set(1, Hex::encode(&pool_registered.pool), &payload);
            }
        }
    }
}
