//! Executes committed package binaries, not native handler substitutes.
//! The captured fixture is one receipt-only transaction; synthetic cases cover
//! duplicate first observations and 32-byte Uniswap V4 pool identifiers.
#[path = "support/package_wasm.rs"]
mod host;

use dex_pools::ethereum::{Block, Log, TransactionReceipt, TransactionTrace};
use dex_swaps_pools::{pb, POOL_TYPE_URL};
use host::{Package, StoreWrite};
use prost::Message;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;
use substreams::pb::sf::substreams::foundational_store::model::v2::SinkEntries;
use substreams::pb::substreams::{module::Kind, store_delta::Operation, StoreDelta, StoreDeltas};

fn package(file: &str) -> Package {
    Package::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("../spkg").join(file))
}

fn canonical() -> &'static Package {
    static PACKAGE: OnceLock<Package> = OnceLock::new();
    PACKAGE.get_or_init(|| package("dex-swaps-pools-v0.9.0.spkg"))
}

fn facade() -> &'static Package {
    static PACKAGE: OnceLock<Package> = OnceLock::new();
    PACKAGE.get_or_init(|| package("evm-dex-foundational-store-v0.7.0.spkg"))
}

fn legacy() -> &'static Package {
    static PACKAGE: OnceLock<Package> = OnceLock::new();
    PACKAGE.get_or_init(|| package("dex-swaps-pools-v0.8.0.spkg"))
}

fn receipt_block(logs: Vec<Log>) -> Block {
    Block {
        transaction_traces: vec![TransactionTrace {
            status: 1,
            receipt: Some(TransactionReceipt { logs, ..Default::default() }),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn pair(pool: u8, token: u8) -> Log {
    let word = |value| [vec![0; 12], vec![value; 20]].concat();
    Log {
        address: vec![90; 20],
        topics: vec![
            hex::decode("0d3648bd0f6ba80134a33ba9275ac585d9d315f0ad8355cddefde31afa28d0e9").unwrap(),
            word(token),
            word(2),
        ],
        data: [word(pool), vec![0; 32]].concat(),
        ..Default::default()
    }
}

fn synthetic() -> (Block, pb::Pools) {
    use ethabi::{ParamType, Token};
    let types = [
        ParamType::FixedBytes(32),
        ParamType::Address,
        ParamType::Address,
        ParamType::Uint(24),
        ParamType::Int(24),
        ParamType::Address,
        ParamType::Uint(160),
        ParamType::Int(24),
    ];
    let v4 = Log {
        address: vec![91; 20],
        topics: vec![
            ethabi::long_signature("Initialize", &types).as_bytes().to_vec(),
            vec![9; 32],
            vec![0; 32],
            [vec![0; 12], vec![2; 20]].concat(),
        ],
        data: ethabi::encode(&[
            Token::Uint(500.into()),
            Token::Int(10.into()),
            Token::Address(ethabi::Address::from_slice(&[3; 20])),
            Token::Uint(1.into()),
            Token::Int(0.into()),
        ]),
        ..Default::default()
    };
    // The later conflicting duplicate must not replace the first pair metadata.
    let mut block = receipt_block(vec![pair(9, 1), pair(9, 7), v4]);
    let mut failed = receipt_block(vec![pair(8, 8)]).transaction_traces.remove(0);
    failed.status = 2;
    block.transaction_traces.push(failed);
    let expected = pb::Pools {
        pools: vec![
            pb::PoolCreated {
                address: hex::encode([9; 20]),
                pool: Some(pb::Pool {
                    tokens: vec![vec![1; 20], vec![2; 20]],
                    factory: vec![90; 20],
                }),
            },
            pb::PoolCreated {
                address: hex::encode([9; 32]),
                pool: Some(pb::Pool {
                    tokens: vec![vec![0; 20], vec![2; 20]],
                    factory: vec![91; 20],
                }),
            },
        ],
    };
    (block, expected)
}

fn captured() -> (Block, pb::Pools) {
    let bytes = include_bytes!("fixtures/uniswap-v2-pair-created-receipt.json");
    let receipt: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let provenance: serde_json::Value = serde_json::from_str(include_str!("fixtures/uniswap-v2-pair-created-provenance.json")).unwrap();
    let decode = |value: &serde_json::Value| hex::decode(value.as_str().unwrap().trim_start_matches("0x")).unwrap();
    let number = |value: &serde_json::Value| u64::from_str_radix(value.as_str().unwrap().trim_start_matches("0x"), 16).unwrap();
    assert_eq!(host::sha256(bytes), provenance["receipt_sha256"].as_str().unwrap());
    assert_eq!(receipt["status"], "0x1");
    assert_eq!(receipt["transactionHash"], provenance["transaction_hash"]);
    assert_eq!(receipt["blockHash"], provenance["block_hash"]);
    let logs = receipt["logs"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, log)| {
            assert_eq!(log["removed"], false);
            Log {
                address: decode(&log["address"]),
                topics: log["topics"].as_array().unwrap().iter().map(decode).collect(),
                data: decode(&log["data"]),
                index: index as u32,
                block_index: number(&log["logIndex"]) as u32,
                ..Default::default()
            }
        })
        .collect();
    let mut block = receipt_block(logs);
    block.number = number(&receipt["blockNumber"]);
    block.hash = decode(&receipt["blockHash"]);
    block.transaction_traces[0].hash = decode(&receipt["transactionHash"]);
    block.transaction_traces[0].index = number(&receipt["transactionIndex"]) as u32;
    block.transaction_traces[0].to = decode(&receipt["to"]);
    // No call traces or Firehose ordinals are available in the captured receipt.
    let expected = pb::Pools {
        pools: vec![pb::PoolCreated {
            address: hex::encode(decode(&provenance["pool_address"])),
            pool: Some(pb::Pool {
                tokens: provenance["tokens"].as_array().unwrap().iter().map(decode).collect(),
                factory: decode(&provenance["factory_address"]),
            }),
        }],
    };
    (block, expected)
}

fn persist(store: &mut BTreeMap<String, Vec<u8>>, writes: Vec<StoreWrite>) {
    for write in writes {
        assert_eq!(write.ordinal, 0, "immutable registry has no execution timing ordinal");
        store.entry(write.key).or_insert(write.value);
    }
}

#[test]
fn packed_graph_has_one_block_decoder_and_facade_aliases_the_same_binary() {
    let package = canonical();
    assert_eq!(package.meta[0], ("dex_swaps_pools".into(), "v0.9.0".into()));
    assert_eq!(package.module_names(), ["map_pool_creations", "map_pool_entries", "map_pools", "store_pools"]);
    assert_eq!(
        package.wiring("map_pool_creations", ""),
        "map_pool_creations <- [source sf.ethereum.type.v2.Block]"
    );
    assert_eq!(package.wiring("map_pool_entries", ""), "map_pool_entries <- [map map_pool_creations]");
    assert_eq!(package.wiring("store_pools", ""), "store_pools <- [map map_pool_creations]");
    assert_eq!(package.wiring("map_pools", ""), "map_pools <- [store store_pools deltas]");
    for name in package.module_names() {
        assert_eq!(package.binary_of(name), package.binary_of("map_pool_creations"));
    }
    let Some(Kind::KindStore(store)) = &package.module("store_pools").kind else {
        panic!("store kind")
    };
    assert_eq!(store.value_type, "proto:dex.foundational_store.v1.Pool");
    let Some(Kind::KindMap(map)) = &package.module("map_pool_entries").kind else {
        panic!("map kind")
    };
    assert_eq!(map.output_type, "proto:sf.substreams.foundational_store.model.v2.SinkEntries");
    for (schema, message) in [
        ("dex.foundational_store.v1", "Pool"),
        ("sf.substreams.foundational_store.model.v2", "SinkEntries"),
    ] {
        assert!(
            package
                .descriptors
                .iter()
                .any(|file| file.package.as_deref() == Some(schema) && file.message_type.iter().any(|ty| ty.name.as_deref() == Some(message))),
            "missing descriptor {schema}.{message}"
        );
    }
    let facade = facade();
    assert_eq!(facade.meta[0], ("evm_dex_foundational_store".into(), "v0.7.0".into()));
    for alias in ["map_entries", "map_pool_entries"] {
        assert_eq!(facade.binary_of(alias), package.binary_of("map_pool_entries"));
        assert_eq!(facade.wiring(alias, "pools:"), package.wiring("map_pool_entries", ""));
    }
    assert_eq!(facade.binary_of("pools:map_pool_creations"), package.binary_of("map_pool_creations"));
    assert_eq!(facade.wiring("pools:map_pool_creations", "pools:"), package.wiring("map_pool_creations", ""));
}

#[test]
fn packed_handlers_preserve_pool_values_and_first_writes_for_synthetic_and_captured_inputs() {
    let package = canonical();
    let legacy = legacy();
    assert_eq!(
        legacy.sha256, "bb8fc8fb7f5867027fb116330993d420ca34e036f3b46439adfa4bf6e1d0e5d4",
        "released baseline changed"
    );
    let mut old_store = BTreeMap::new();
    let mut new_store = BTreeMap::new();
    let mut hosted_store = BTreeMap::new();
    let (synthetic, expected) = synthetic();
    let later = (
        receipt_block(vec![pair(9, 6)]),
        pb::Pools {
            pools: vec![pb::PoolCreated {
                address: hex::encode([9; 20]),
                pool: Some(pb::Pool {
                    tokens: vec![vec![6; 20], vec![2; 20]],
                    factory: vec![90; 20],
                }),
            }],
        },
    );
    for (block, expected) in [(synthetic, expected), captured(), later, (Block::default(), pb::Pools::default())] {
        let block = block.encode_to_vec();
        let creations = package.evaluate("map_pool_creations", &block);
        assert_eq!(pb::Pools::decode(creations.as_slice()).unwrap(), expected);
        let entries = package.evaluate("map_pool_entries", &block);
        let hosted = SinkEntries::decode(entries.as_slice()).unwrap();
        assert!(hosted.if_not_exist, "including empty batches");
        assert_eq!(hosted.entries.len(), expected.pools.len());
        for (entry, expected) in hosted.entries.iter().zip(&expected.pools) {
            let key = &entry.key.as_ref().expect("raw key").bytes;
            assert_eq!(key, &hex::decode(&expected.address).unwrap());
            let value = entry.value.as_ref().expect("Any value");
            assert_eq!(value.type_url, POOL_TYPE_URL);
            assert_eq!(value.value, expected.pool.as_ref().unwrap().encode_to_vec());
            assert_eq!(pb::Pool::decode(value.value.as_slice()).unwrap(), *expected.pool.as_ref().unwrap());
            hosted_store.entry(hex::encode(key)).or_insert(value.value.clone());
        }
        for alias in ["map_entries", "map_pool_entries"] {
            assert_eq!(facade().evaluate(alias, &block), entries, "facade {alias} output differs");
        }
        let writes = package.run_store("store_pools", &[&creations]).unwrap();
        assert_eq!(writes.len(), expected.pools.len());
        for (write, expected) in writes.iter().zip(&expected.pools) {
            assert_eq!(write.key, expected.address);
            assert_eq!(write.value, expected.pool.as_ref().unwrap().encode_to_vec());
        }
        persist(&mut new_store, writes);
        persist(&mut old_store, legacy.run_store("store_pools", &[&block]).unwrap());
        assert_eq!(new_store, old_store, "persisted first writes differ from released v0.8");
        assert_eq!(hosted_store, old_store, "Hosted and legacy values differ");
    }
    assert_eq!(pb::Pool::decode(new_store[&hex::encode([9; 20])].as_slice()).unwrap().tokens[0], vec![1; 20]);
}

#[test]
fn packed_map_pools_emits_only_create_deltas() {
    let (_, expected) = synthetic();
    let mut deltas = Vec::new();
    for (index, operation) in [Operation::Update, Operation::Create, Operation::Delete, Operation::Create]
        .into_iter()
        .enumerate()
    {
        let pool = &expected.pools[index % 2];
        deltas.push(StoreDelta {
            operation: operation as i32,
            ordinal: index as u64,
            key: pool.address.clone(),
            old_value: pool.pool.as_ref().unwrap().encode_to_vec(),
            new_value: pool.pool.as_ref().unwrap().encode_to_vec(),
        });
    }
    let output = canonical().run("map_pools", &[&StoreDeltas { deltas }.encode_to_vec()]).unwrap();
    let pools = pb::Pools::decode(output.as_slice()).unwrap();
    assert_eq!(pools.pools, vec![expected.pools[1].clone(), expected.pools[1].clone()]);
    let empty = canonical().run("map_pools", &[&StoreDeltas::default().encode_to_vec()]).unwrap();
    assert!(pb::Pools::decode(empty.as_slice()).unwrap().pools.is_empty());
}

#[test]
fn packed_adapters_reject_invalid_identity_and_missing_pool_value() {
    let (_, valid) = synthetic();
    let mut invalid = valid.clone();
    invalid.pools.push(pb::PoolCreated {
        address: "AB".repeat(20),
        pool: valid.pools[0].pool.clone(),
    });
    let mut missing = valid;
    missing.pools[0].pool = None;
    for pools in [invalid, missing] {
        let bytes = pools.encode_to_vec();
        assert!(canonical().run("map_pool_entries", &[&bytes]).is_none());
        assert!(canonical().run_store("store_pools", &[&bytes]).is_none());
    }
}
