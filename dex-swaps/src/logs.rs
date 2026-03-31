use std::collections::{HashMap, HashSet};

use prost::Message;
use prost_types::Any;
use proto::pb::dex::foundational_store::v1::Pool;
use substreams::{
    pb::sf::substreams::foundational_store::model::v2::{QueriedEntry, ResponseCode},
    store::FoundationalStore,
};
use substreams_abis::dex::balancer;
use substreams_abis::dex::uniswap::v4 as uniswap_v4;
use substreams_ethereum::{pb::eth::v2::{Block, Log}, Event};

pub(crate) type PoolMetadata = Pool;
pub(crate) type PoolMetadataMap = HashMap<Vec<u8>, PoolMetadata>;

pub(crate) fn collect_log_addresses(block: &Block) -> HashSet<Vec<u8>> {
    let mut addresses = HashSet::new();

    for log in block.logs() {
        collect_log_address(log.log, &mut addresses);
    }

    addresses
}

fn collect_log_address(log: &Log, addresses: &mut HashSet<Vec<u8>>) {
    if !log.address.is_empty() {
        addresses.insert(log.address.clone());
    }

    if let Some(event) = uniswap_v4::poolmanager::events::Swap::match_and_decode(log) {
        addresses.insert(event.id.to_vec());
    }

    if let Some(event) = balancer::v3::vault::events::Swap::match_and_decode(log) {
        addresses.insert(event.pool.to_vec());
    }
}

pub(crate) fn get_pools_by_address(store: &FoundationalStore, addresses: &HashSet<Vec<u8>>) -> PoolMetadataMap {
    if addresses.is_empty() {
        return PoolMetadataMap::default();
    }

    let keys = addresses.iter().cloned().collect::<Vec<_>>();
    let queried = store.get_first(&keys);

    keys.into_iter()
        .zip(queried.entries)
        .filter_map(|(key, queried)| decode_queried_pool(queried).map(|pool| (key, pool)))
        .collect()
}

fn decode_queried_pool(queried: QueriedEntry) -> Option<PoolMetadata> {
    if queried.code != ResponseCode::Found as i32 {
        return None;
    }

    let value = queried.entry?.value?;
    decode_pool(value)
}

fn decode_pool(value: Any) -> Option<PoolMetadata> {
    (value.type_url == "type.googleapis.com/dex.foundational_store.v1.Pool").then_some(())?;
    PoolMetadata::decode(value.value.as_slice()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use substreams::pb::sf::substreams::foundational_store::model::v2::{Entry, Key};

    #[test]
    fn decodes_foundational_pool_payload() {
        let queried = QueriedEntry {
            code: ResponseCode::Found as i32,
            entry: Some(Entry {
                key: Some(Key { bytes: vec![0x01] }),
                value: Some(Any {
                    type_url: "type.googleapis.com/dex.foundational_store.v1.Pool".into(),
                    value: PoolMetadata {
                        tokens: vec![vec![0xaa], vec![0xbb]],
                        factory: vec![0xcc],
                    }
                    .encode_to_vec(),
                }),
            }),
        };

        let pool = decode_queried_pool(queried).unwrap();
        assert_eq!(pool.tokens, vec![vec![0xaa], vec![0xbb]]);
        assert_eq!(pool.factory, vec![0xcc]);
    }
}
