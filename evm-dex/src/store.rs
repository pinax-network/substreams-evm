use std::collections::{HashMap, HashSet};

use common::{bytes_to_string, Encoding};
use prost::Message;
use prost_types::Any;
use proto::pb::dex::foundational_store::v1::Pool;
use substreams::{
    pb::sf::substreams::foundational_store::model::v2::{QueriedEntries, ResponseCode},
    store::FoundationalStore,
};

pub type PoolMetadata = Pool;
pub type PoolMetadataMap = HashMap<Vec<u8>, PoolMetadata>;

pub fn collect_address(addresses: &mut HashSet<Vec<u8>>, address: &[u8]) {
    if !address.is_empty() {
        addresses.insert(address.to_vec());
    }
}

pub fn get_pools_by_address(store: &FoundationalStore, addresses: &HashSet<Vec<u8>>) -> PoolMetadataMap {
    if addresses.is_empty() {
        return PoolMetadataMap::default();
    }

    let keys = addresses.iter().cloned().collect::<Vec<_>>();
    let queried = store.get_first(&keys);
    decode_queried_pools(keys, queried)
}

pub fn get_pool_by_address<'a>(pools: &'a PoolMetadataMap, address: &[u8]) -> Option<&'a PoolMetadata> {
    pools.get(address)
}

pub fn token(pool: &PoolMetadata, index: usize) -> &[u8] {
    // Some protocols expose partial or single-token metadata at initialization time, so
    // downstream normalization intentionally falls back to an empty byte slice when a token slot
    // is unavailable in the foundational payload.
    pool.tokens.get(index).map(Vec::as_slice).unwrap_or_default()
}

pub fn tokens_csv(encoding: &Encoding, pool: &PoolMetadata) -> String {
    pool.tokens.iter().map(|token| bytes_to_string(token, encoding)).collect::<Vec<_>>().join(",")
}

fn decode_pool(value: Any) -> Option<PoolMetadata> {
    (value.type_url == "type.googleapis.com/dex.foundational_store.v1.Pool").then_some(())?;
    PoolMetadata::decode(value.value.as_slice()).ok()
}

fn decode_queried_pools(keys: Vec<Vec<u8>>, queried: QueriedEntries) -> PoolMetadataMap {
    keys.into_iter()
        .zip(queried.entries)
        .filter_map(|(key, queried)| decode_queried_pool(queried).map(|pool| (key, pool)))
        .collect()
}

fn decode_queried_pool(queried: substreams::pb::sf::substreams::foundational_store::model::v2::QueriedEntry) -> Option<PoolMetadata> {
    if queried.code != ResponseCode::Found as i32 {
        return None;
    }

    queried.entry.and_then(|entry| entry.value).and_then(decode_pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use substreams::pb::sf::substreams::foundational_store::model::v2::{Entry, Key, QueriedEntries, QueriedEntry};

    #[test]
    fn decode_pool_accepts_foundational_any_payload() {
        let pool = PoolMetadata { tokens: vec![vec![0xaa], vec![0xbb]], factory: vec![0xcc] };
        let any = Any {
            type_url: "type.googleapis.com/dex.foundational_store.v1.Pool".into(),
            value: pool.encode_to_vec(),
        };

        let decoded = decode_pool(any).unwrap();
        assert_eq!(decoded.tokens, pool.tokens);
        assert_eq!(decoded.factory, pool.factory);
    }

    #[test]
    fn decode_pool_rejects_wrong_type_url() {
        let any = Any { type_url: "type.googleapis.com/google.protobuf.Empty".into(), value: vec![] };
        assert!(decode_pool(any).is_none());
    }

    #[test]
    fn decode_queried_pools_preserves_requested_key_mapping() {
        let pool_a = PoolMetadata { tokens: vec![vec![0xaa]], factory: vec![0x01] };
        let pool_c = PoolMetadata { tokens: vec![vec![0xcc]], factory: vec![0x03] };
        let keys = vec![vec![0x0a], vec![0x0b], vec![0x0c]];
        let queried = QueriedEntries {
            entries: vec![
                QueriedEntry {
                    code: ResponseCode::Found as i32,
                    entry: Some(Entry {
                        key: Some(Key { bytes: vec![0xff] }),
                        value: Some(Any {
                            type_url: "type.googleapis.com/dex.foundational_store.v1.Pool".into(),
                            value: pool_a.encode_to_vec(),
                        }),
                    }),
                },
                QueriedEntry { code: ResponseCode::NotFound as i32, entry: None },
                QueriedEntry {
                    code: ResponseCode::Found as i32,
                    entry: Some(Entry {
                        key: Some(Key { bytes: vec![0xee] }),
                        value: Some(Any {
                            type_url: "type.googleapis.com/dex.foundational_store.v1.Pool".into(),
                            value: pool_c.encode_to_vec(),
                        }),
                    }),
                },
            ],
        };

        let pools = decode_queried_pools(keys, queried);
        assert_eq!(pools.len(), 2);
        assert_eq!(pools.get(&vec![0x0a]).unwrap().factory, vec![0x01]);
        assert_eq!(pools.get(&vec![0x0c]).unwrap().tokens, vec![vec![0xcc]]);
    }

    #[test]
    fn collect_address_skips_empty_addresses() {
        let mut addresses = HashSet::new();
        collect_address(&mut addresses, &[]);
        collect_address(&mut addresses, &[0xaa]);
        collect_address(&mut addresses, &[0xaa]);

        assert_eq!(addresses.len(), 1);
        assert!(addresses.contains(&vec![0xaa]));
    }
}
