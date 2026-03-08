use common::NULL_ADDRESS;
use proto::pb::curvefi::v1 as pb;
use substreams::store::StoreSetProto;
use substreams::{hex, prelude::*, Hex};

const TRI_POOL_ADDRESS: [u8; 20] = hex!("bebc44782c7db0a1a60cb6fe97d0b483032ff1c7");
const DAI_ADDRESS: [u8; 20] = hex!("6b175474e89094c44da98b954eedeac495271d0f");
const USDC_ADDRESS: [u8; 20] = hex!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
const USDT_ADDRESS: [u8; 20] = hex!("dac17f958d2ee523a2206206994597c13d831ec7");

fn known_pool(address: &[u8]) -> Option<pb::StorePool> {
    if address == TRI_POOL_ADDRESS {
        return Some(pb::StorePool {
            factory: NULL_ADDRESS.to_vec(),
            coins: vec![DAI_ADDRESS.to_vec(), USDC_ADDRESS.to_vec(), USDT_ADDRESS.to_vec()],
        });
    }

    None
}

fn seed_known_pools(store: &StoreSetProto<pb::StorePool>) {
    if let Some(pool) = known_pool(&TRI_POOL_ADDRESS) {
        store.set(0, Hex::encode(TRI_POOL_ADDRESS), &pool);
    }
}

fn has_known_pool_activity(events: &pb::Events) -> bool {
    events
        .transactions
        .iter()
        .flat_map(|trx| trx.logs.iter())
        .any(|log| known_pool(&log.address).is_some())
}

#[substreams::handlers::store]
pub fn store_pool(events: pb::Events, store: StoreSetProto<pb::StorePool>) {
    if has_known_pool_activity(&events) {
        seed_known_pools(&store);
    }

    for trx in events.transactions.iter() {
        for log in trx.logs.iter() {
            // ---- PlainPoolDeployed ----
            if let Some(pb::log::Log::PlainPoolDeployed(plain_pool_deployed)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    coins: plain_pool_deployed.coins.clone(),
                };
                store.set(1, Hex::encode(&plain_pool_deployed.address), &payload);
            }
            // ---- MetaPoolDeployed ----
            if let Some(pb::log::Log::MetaPoolDeployed(meta_pool_deployed)) = &log.log {
                let payload = pb::StorePool {
                    factory: log.address.clone(),
                    coins: vec![meta_pool_deployed.coin.clone()],
                };
                store.set(1, Hex::encode(&meta_pool_deployed.address), &payload);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_known_tripool_metadata() {
        let pool = known_pool(&TRI_POOL_ADDRESS).expect("TriPool metadata should be seeded");

        assert_eq!(pool.factory, NULL_ADDRESS.to_vec());
        assert_eq!(
            pool.coins,
            vec![DAI_ADDRESS.to_vec(), USDC_ADDRESS.to_vec(), USDT_ADDRESS.to_vec()]
        );
    }

    #[test]
    fn ignores_unknown_pool_addresses() {
        assert!(known_pool(&hex!("1111111111111111111111111111111111111111")).is_none());
    }

    #[test]
    fn detects_tripool_activity() {
        let mut events = pb::Events::default();
        let mut transaction = pb::Transaction::default();
        transaction.logs.push(pb::Log {
            address: TRI_POOL_ADDRESS.to_vec(),
            ..Default::default()
        });
        events.transactions.push(transaction);

        assert!(has_known_pool_activity(&events));
    }
}
