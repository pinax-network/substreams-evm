use proto::pb::tron::sunswap::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_new_exchange(events: pb::justswap::v1::Events, store: StoreSetProto<NewExchange>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- NewExchange ----
            if let Some(pb::justswap::v1::log::Log::NewExchange(new_exchange)) = &log.log {
                let payload = NewExchange {
                    exchange: new_exchange.exchange.clone(),
                    factory: log.address.clone(),
                    token: new_exchange.token.clone(),
                };
                store.set(1, Hex::encode(&new_exchange.exchange), &payload);
            }
        }
    }
}