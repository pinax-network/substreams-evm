use proto::pb::uniswap::v3 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PoolCreated ----
            if let Some(pb::log::Log::PoolCreated(pool_created)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    currency0: pool_created.token0.clone(),
                    currency1: pool_created.token1.clone(),
                };
                store.set(1, Hex::encode(&pool_created.pool), &payload);
            }
        }
    }
}
