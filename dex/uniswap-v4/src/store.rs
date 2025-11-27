use proto::pb::uniswap::v4 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- Initialize ----
            if let Some(pb::log::Log::Initialize(initialize)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    currency0: initialize.currency0.clone(),
                    currency1: initialize.currency1.clone(),
                };
                store.set(1, Hex::encode(&initialize.id), &payload);
            }
        }
    }
}
