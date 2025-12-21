use proto::pb::polymarket::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_token(events: pb::Events, store: StoreSetProto<pb::StoreToken>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- TokenRegistered ----
            if let Some(pb::log::Log::TokenRegistered(token_registered)) = &log.log {
                let payload = pb::StoreToken {
                    token0: token_registered.token0.clone(),
                    token1: token_registered.token1.clone(),
                    exchange: log.address.clone(),
                };
                store.set(1, Hex::encode(&token_registered.condition_id), &payload);
            }
        }
    }
}
