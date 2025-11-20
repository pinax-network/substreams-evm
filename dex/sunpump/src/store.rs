use proto::pb::sunpump::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_token_create(sunpump: pb::Events, store: StoreSetProto<pb::TokenCreate>) {
    for trx in sunpump.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- TokenCreate ----
            if let Some(pb::log::Log::TokenCreate(token_create)) = &log.log {
                let payload = pb::TokenCreate {
                    token_address: token_create.token_address.clone(),
                    factory: log.address.clone(),
                    token_index: token_create.token_index.clone(),
                    creator: token_create.creator.clone(),
                };
                store.set(1, Hex::encode(&token_create.token_address), &payload);
            }
        }
    }
}
