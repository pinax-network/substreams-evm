use proto::pb::curvefi::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{prelude::*, Hex};

#[substreams::handlers::store]
pub fn store_plain_pool_deployed(events: pb::Events, store: StoreSetProto<pb::PlainPoolDeployed>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PlainPoolDeployed ----
            if let Some(pb::log::Log::PlainPoolDeployed(plain_pool_deployed)) = &log.log {
                let payload = pb::PlainPoolDeployed {
                    factory: plain_pool_deployed.factory.clone(),
                    address: plain_pool_deployed.address.clone(),
                    a: plain_pool_deployed.a.clone(),
                    coins: plain_pool_deployed.coins.clone(),
                    deployer: plain_pool_deployed.deployer.clone(),
                    fee: plain_pool_deployed.fee.clone(),
                };
                store.set(1, Hex::encode(&plain_pool_deployed.address), &payload);
            }
        }
    }
}

#[substreams::handlers::store]
pub fn store_meta_pool_deployed(events: pb::Events, store: StoreSetProto<pb::MetaPoolDeployed>) {
    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- MetaPoolDeployed ----
            if let Some(pb::log::Log::MetaPoolDeployed(meta_pool_deployed)) = &log.log {
                let payload = pb::MetaPoolDeployed {
                    factory: meta_pool_deployed.factory.clone(),
                    address: meta_pool_deployed.address.clone(),
                    a: meta_pool_deployed.a.clone(),
                    base_pool: meta_pool_deployed.base_pool.clone(),
                    coin: meta_pool_deployed.coin.clone(),
                    deployer: meta_pool_deployed.deployer.clone(),
                    fee: meta_pool_deployed.fee.clone(),
                };
                store.set(1, Hex::encode(&meta_pool_deployed.address), &payload);
            }
        }
    }
}
