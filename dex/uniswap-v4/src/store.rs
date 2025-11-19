use proto::pb::evm::uniswap::v4 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_initialize(events: pb::Events, store: StoreSetProto<pb::Initialize>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- Initialize ----
            if let Some(pb::log::Log::Initialize(initialize)) = &log.log {
                let payload = pb::Initialize {
                    factory: log.address.clone(),
                    id: initialize.id.clone(),
                    currency0: initialize.currency0.clone(),
                    currency1: initialize.currency1.clone(),
                    fee: initialize.fee,
                    tick_spacing: initialize.tick_spacing,
                    hooks: initialize.hooks.clone(),
                    sqrt_price_x96: initialize.sqrt_price_x96.clone(),
                    tick: initialize.tick,
                };
                store.set(1, Hex::encode(&initialize.id), &payload);
            }
        }
    }
}
