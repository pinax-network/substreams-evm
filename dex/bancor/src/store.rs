use proto::pb::bancor::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_activation(events: pb::Events, store: StoreSetProto<pb::Activation>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- Activation ----
            if let Some(pb::log::Log::Activation(activation)) = &log.log {
                // Only store activations (not deactivations)
                if activation.activated {
                    let payload = pb::Activation {
                        factory: activation.factory.clone(),
                        converter_type: activation.converter_type,
                        anchor: activation.anchor.clone(),
                        activated: activation.activated,
                    };
                    store.set(1, Hex::encode(&activation.anchor), &payload);
                }
            }
        }
    }
}
