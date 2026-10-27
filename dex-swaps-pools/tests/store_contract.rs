use std::cell::RefCell;
use std::collections::BTreeMap;

use dex_swaps_pools::{map_pools_impl, pb, pool_entries, store_pools_impl, POOL_TYPE_URL};
use prost::Message;
use substreams::pb::substreams::store_delta::Operation;
use substreams::store::{DeltaProto, Deltas, StoreDelete, StoreNew, StoreSetIfNotExists};

#[derive(Default)]
struct Registry {
    values: RefCell<BTreeMap<String, Vec<u8>>>,
    writes: RefCell<Vec<(u64, String, Vec<u8>)>>,
    creates: RefCell<Vec<DeltaProto<pb::Pool>>>,
}

impl StoreNew for Registry {
    fn new() -> Self {
        Self::default()
    }
}
impl StoreDelete for Registry {
    fn delete_prefix(&self, _: i64, _: &String) {
        panic!("immutable registry must never delete");
    }
}
impl StoreSetIfNotExists<pb::Pool> for Registry {
    fn set_if_not_exists<K: AsRef<str>>(&self, ordinal: u64, key: K, value: &pb::Pool) {
        let key = key.as_ref().to_owned();
        let bytes = value.encode_to_vec();
        self.writes.borrow_mut().push((ordinal, key.clone(), bytes.clone()));
        if let std::collections::btree_map::Entry::Vacant(entry) = self.values.borrow_mut().entry(key.clone()) {
            entry.insert(bytes);
            self.creates.borrow_mut().push(DeltaProto {
                operation: Operation::Create,
                ordinal,
                key,
                old_value: pb::Pool::default(),
                new_value: value.clone(),
            });
        }
    }
    fn set_if_not_exists_many<K: AsRef<str>>(&self, ordinal: u64, keys: &Vec<K>, value: &pb::Pool) {
        for key in keys {
            self.set_if_not_exists(ordinal, key, value);
        }
    }
}

fn entry(key: Vec<u8>, tokens: Vec<Vec<u8>>, factory: Vec<u8>) -> pb::PoolCreated {
    pb::PoolCreated {
        address: hex::encode(key),
        pool: Some(pb::Pool { tokens, factory }),
    }
}

#[test]
fn hosted_and_legacy_values_match_without_losing_key_width_or_empty_fields() {
    let mut key20 = vec![0; 20];
    key20[19] = 1;
    let mut key32 = vec![0; 32];
    key32[31] = 1;
    let pools = pb::Pools {
        pools: vec![
            entry(key20.clone(), vec![], vec![]),
            entry(key32.clone(), vec![vec![0; 20], vec![2; 20], vec![3; 20]], vec![4; 20]),
        ],
    };
    let hosted = pool_entries(&pools).unwrap();
    assert!(hosted.if_not_exist);
    assert_eq!(hosted.entries.len(), 2);
    let registry = Registry::new();
    store_pools_impl(&pools, &registry).unwrap();
    for ((hosted, expected), key) in hosted.entries.iter().zip(&pools.pools).zip([key20, key32]) {
        assert_eq!(hosted.key.as_ref().unwrap().bytes, key);
        let any = hosted.value.as_ref().unwrap();
        assert_eq!(any.type_url, POOL_TYPE_URL);
        assert_eq!(any.value, registry.values.borrow()[&expected.address]);
        assert_eq!(pb::Pool::decode(any.value.as_slice()).unwrap(), *expected.pool.as_ref().unwrap());
    }
    assert!(registry.writes.borrow().iter().all(|(ordinal, _, _)| *ordinal == 0));
    assert!(pool_entries(&pb::Pools::default()).unwrap().if_not_exist);
    assert!(pool_entries(&pb::Pools::default()).unwrap().entries.is_empty());
}

#[test]
fn both_adapters_keep_the_first_payload_within_and_across_blocks() {
    let first = entry(vec![1; 20], vec![], vec![]);
    let conflicting = entry(vec![1; 20], vec![vec![2; 20]], vec![3; 20]);
    let second = entry(vec![4; 32], vec![vec![5; 20]], vec![6; 20]);
    let registry = Registry::new();
    let mut hosted_state = BTreeMap::new();
    for pools in [
        pb::Pools {
            pools: vec![first.clone(), conflicting.clone(), second.clone()],
        },
        pb::Pools {
            pools: vec![conflicting, second.clone()],
        },
    ] {
        for entry in pool_entries(&pools).unwrap().entries {
            hosted_state.entry(hex::encode(entry.key.unwrap().bytes)).or_insert(entry.value.unwrap().value);
        }
        store_pools_impl(&pools, &registry).unwrap();
    }
    assert_eq!(*registry.values.borrow(), hosted_state);
    assert_eq!(registry.values.borrow()[&first.address], first.pool.as_ref().unwrap().encode_to_vec());
    let output = map_pools_impl(Deltas {
        deltas: registry.creates.take(),
    });
    assert_eq!(output.pools, vec![first, second]);
    assert!(map_pools_impl(Deltas { deltas: vec![] }).pools.is_empty());
}

#[test]
fn legacy_delta_map_emits_only_create_operations_in_input_order() {
    let value = pb::Pool {
        tokens: vec![vec![1; 20]],
        factory: vec![],
    };
    let deltas = [Operation::Update, Operation::Create, Operation::Delete, Operation::Create]
        .into_iter()
        .enumerate()
        .map(|(i, operation)| DeltaProto {
            operation,
            ordinal: i as u64,
            key: format!("{i:040x}"),
            old_value: pb::Pool::default(),
            new_value: value.clone(),
        })
        .collect();
    let output = map_pools_impl(Deltas { deltas });
    assert_eq!(
        output.pools.iter().map(|entry| entry.address.as_str()).collect::<Vec<_>>(),
        vec!["0000000000000000000000000000000000000001", "0000000000000000000000000000000000000003",]
    );
    assert!(output.pools.iter().all(|entry| entry.pool.as_ref() == Some(&value)));
}

#[test]
fn invalid_batches_fail_before_either_adapter_can_write() {
    let valid = entry(vec![0xab; 20], vec![], vec![]);
    let mut missing = valid.clone();
    missing.pool = None;
    let mut malformed = vec![missing];
    for address in [
        "zz".into(),
        "ab".repeat(19),
        "ab".repeat(21),
        "ab".repeat(31),
        "ab".repeat(33),
        "AB".repeat(20),
        format!("0x{}", valid.address),
    ] {
        malformed.push(pb::PoolCreated {
            address,
            pool: Some(pb::Pool::default()),
        });
    }
    for invalid in malformed {
        let pools = pb::Pools {
            pools: vec![valid.clone(), invalid],
        };
        assert!(pool_entries(&pools).is_err());
        let registry = Registry::new();
        assert!(store_pools_impl(&pools, &registry).is_err());
        assert!(registry.writes.borrow().is_empty());
    }
}

#[test]
fn pool_wire_contract_remains_tokens_field_one_and_factory_field_two() {
    // Frozen bytes use two token occurrences (field 1) followed by factory (field 2).
    let bytes = [0x0a, 0x02, 0x00, 0x01, 0x0a, 0x01, 0x02, 0x12, 0x01, 0x03];
    let pool = pb::Pool::decode(bytes.as_slice()).unwrap();
    assert_eq!(pool.tokens, vec![vec![0, 1], vec![2]]);
    assert_eq!(pool.factory, vec![3]);
    assert_eq!(pool.encode_to_vec(), bytes);
}
