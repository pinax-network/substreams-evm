use proto::pb::bancor::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- FeaturesAddition (for Legacy Pools)----
            if let Some(pb::log::Log::FeaturesAddition(event)) = &log.log {
                let payload = pb::StorePool { factory: log.address.clone() };
                store.set(1, Hex::encode(&event.address), &payload);
            }

            // ---- NewConverter ----
            if let Some(pb::log::Log::NewConverter(event)) = &log.log {
                let payload = pb::StorePool { factory: log.address.clone() };
                store.set(1, Hex::encode(&event.converter), &payload);
            }
        }
    }
}
