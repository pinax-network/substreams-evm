use proto::pb::uniswap::v3 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool_created(events: pb::Events, store: StoreSetProto<pb::PoolCreated>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PoolCreated ----
            if let Some(pb::log::Log::PoolCreated(pool_created)) = &log.log {
                let payload = pb::PoolCreated {
                    factory: log.address.clone(),
                    pool: pool_created.pool.clone(),
                    token0: pool_created.token0.clone(),
                    token1: pool_created.token1.clone(),
                    fee: pool_created.fee,
                    tick_spacing: pool_created.tick_spacing,
                };
                store.set(1, Hex::encode(&pool_created.pool), &payload);
            }
        }
    }
}
