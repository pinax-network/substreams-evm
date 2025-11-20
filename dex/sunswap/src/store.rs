use proto::pb::dex::sunswap::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pair_created(events: pb::Events, store: StoreSetProto<pb::PairCreated>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PairCreated ----
            if let Some(pb::log::Log::PairCreated(pair_created)) = &log.log {
                let payload = pb::PairCreated {
                    factory: log.address.clone(),
                    pair: pair_created.pair.clone(),
                    token0: pair_created.token0.clone(),
                    token1: pair_created.token1.clone(),
                    extra_data: pair_created.extra_data.clone(),
                };
                store.set(1, Hex::encode(&pair_created.pair), &payload);
            }
        }
    }
}
