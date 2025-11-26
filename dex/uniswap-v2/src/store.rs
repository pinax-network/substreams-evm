use proto::pb::uniswap::v2 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pair_created(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PairCreated ----
            if let Some(pb::log::Log::PairCreated(pair_created)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    currency0: pair_created.token0.clone(),
                    currency1: pair_created.token1.clone(),
                };
                store.set(1, Hex::encode(&pair_created.pair), &payload);
            }
        }
    }
}
