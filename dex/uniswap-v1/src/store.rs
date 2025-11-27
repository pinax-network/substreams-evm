use proto::pb::uniswap::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- NewExchange ----
            if let Some(pb::log::Log::NewExchange(new_exchange)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    currency0: new_exchange.token.clone(),
                };
                store.set(1, Hex::encode(&new_exchange.exchange), &payload);
            }
        }
    }
}
