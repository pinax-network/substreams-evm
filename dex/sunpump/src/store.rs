use proto::pb::sunpump::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(sunpump: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in sunpump.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- TokenCreate ----
            if let Some(pb::log::Log::TokenCreate(events)) = &log.log {
                let payload = pb::StorePool { factory: log.address.clone() };
                store.set(1, Hex::encode(&events.token_address), &payload);
            }
            // ---- TokenCreateLegacy ----
            if let Some(pb::log::Log::TokenCreateLegacy(event)) = &log.log {
                let payload = pb::StorePool { factory: log.address.clone() };
                store.set(1, Hex::encode(&event.token_address), &payload);
            }
        }
    }
}
