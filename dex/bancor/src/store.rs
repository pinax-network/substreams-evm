use proto::pb::bancor::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_new_converter(events: pb::Events, store: StoreSetProto<pb::NewConverter>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- NewConverter ----
            if let Some(pb::log::Log::NewConverter(event)) = &log.log {
                let payload = pb::NewConverter {
                    factory: event.factory.clone(),
                    converter_type: event.converter_type,
                    owner: event.owner.clone(),
                    converter: event.converter.clone(),
                };
                store.set(1, Hex::encode(&event.converter), &payload);
            }
        }
    }
}
