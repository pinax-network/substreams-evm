use proto::pb::uniswap::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pair_created(events: pb::Events, store: StoreSetProto<pb::NewExchange>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- NewExchange ----
            if let Some(pb::log::Log::NewExchange(new_exchange)) = &log.log {
                let payload = pb::NewExchange {
                    factory: log.address.clone(),
                    exchange: new_exchange.exchange.clone(),
                    token: new_exchange.token.clone(),
                };
                store.set(1, Hex::encode(&new_exchange.exchange), &payload);
            }
        }
    }
}
