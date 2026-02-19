use proto::pb::traderjoe::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- LbPairCreated ----
            if let Some(pb::log::Log::LbPairCreated(pair_created)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    currency0: pair_created.token_x.clone(),
                    currency1: pair_created.token_y.clone(),
                    bin_step: pair_created.bin_step,
                };
                store.set(1, Hex::encode(&pair_created.lb_pair), &payload);
            }
        }
    }
}
