use proto::pb::sunpump::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_token_create(sunpump: pb::Events, store: StoreSetProto<pb::TokenCreate>) {
    for trx in sunpump.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- TokenCreate ----
            if let Some(pb::log::Log::TokenCreate(events)) = &log.log {
                let payload = pb::TokenCreate {
                    token_address: events.token_address.clone(),
                    factory: log.address.clone(),
                    token_index: events.token_index.clone(),
                    creator: events.creator.clone(),
                };
                store.set(1, Hex::encode(&events.token_address), &payload);
            }
            // ---- TokenCreateLegacy ----
            if let Some(pb::log::Log::TokenCreateLegacy(event)) = &log.log {
                let payload = pb::TokenCreate {
                    token_address: event.token_address.clone(),
                    factory: log.address.clone(),
                    token_index: 0.to_string(),
                    creator: event.creator.clone(),
                };
                store.set(1, Hex::encode(&event.token_address), &payload);
            }
        }
    }
}
