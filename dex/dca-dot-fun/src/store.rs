use proto::pb::dca_dot_fun::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::prelude::*;

#[substreams::handlers::store]
pub fn store_order(events: pb::Events, store: StoreSetProto<pb::StoreOrder>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            if let Some(pb::log::Log::CreateOrder(order)) = &log.log {
                let payload = pb::StoreOrder {
                    token_in: order.token_in.clone(),
                    token_out: order.token_out.clone(),
                };
                store.set(1, &order.order_id, &payload);
            }
        }
    }
}
