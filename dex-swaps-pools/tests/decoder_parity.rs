//! Frozen legacy collector contracts, independent of the extracted implementation.
//!
//! These are synthetic ABI fixtures, not mainnet captures. Event declarations come from
//! substreams-abis v1.5.0 (62b75b660dc0035bd4767d4e4d8ccdb92fcd98a3), abi/dex/.
//! Expected records preserve dex-swaps-pools at 6bd0034, including Curve's existing
//! first-CREATE association and constructor-tail heuristic; those are not provenance proofs.

use dex_pools::ethereum::{Block, Call, CallType, Log, TransactionReceipt, TransactionTrace};
use dex_pools::{decode_log, decode_log_bytes, project, PoolEntry};
use ethabi::{param_type::Reader, ParamType, Token};

fn address(value: u8) -> Vec<u8> {
    vec![value; 20]
}

fn addr(value: u8) -> Token {
    Token::Address(ethabi::Address::from_slice(&address(value)))
}

fn uint(value: u64) -> Token {
    Token::Uint(value.into())
}

fn int(value: u64) -> Token {
    Token::Int(value.into())
}

fn event(name: &str, fields: Vec<(&str, Token, bool)>) -> Log {
    let types: Vec<ParamType> = fields.iter().map(|(kind, _, _)| Reader::read(kind).unwrap()).collect();
    let mut topics = vec![ethabi::long_signature(name, &types).as_bytes().to_vec()];
    let mut data = Vec::new();
    for (_, value, indexed) in fields {
        if indexed {
            // Every indexed fixture field is static and occupies one ABI word.
            topics.push(ethabi::encode(&[value]));
        } else {
            data.push(value);
        }
    }
    Log {
        address: address(90),
        topics,
        data: ethabi::encode(&data),
        ..Default::default()
    }
}

struct Fixture {
    name: &'static str,
    log: Log,
    expected: PoolEntry,
    needs_create: bool,
}

fn fixture(name: &'static str, log: Log, tokens: &[u8]) -> Fixture {
    Fixture {
        name,
        log,
        expected: PoolEntry {
            address: address(9),
            tokens: tokens.iter().map(|value| address(*value)).collect(),
            factory: address(90),
        },
        needs_create: false,
    }
}

fn fixtures() -> Vec<Fixture> {
    let mut cases = vec![
        fixture(
            "uniswap v1",
            event("NewExchange", vec![("address", addr(1), true), ("address", addr(9), true)]),
            &[1],
        ),
        fixture(
            "uniswap v2",
            event(
                "PairCreated",
                vec![
                    ("address", addr(1), true),
                    ("address", addr(2), true),
                    ("address", addr(9), false),
                    ("uint256", uint(1), false),
                ],
            ),
            &[1, 2],
        ),
        fixture(
            "uniswap v3",
            event(
                "PoolCreated",
                vec![
                    ("address", addr(1), true),
                    ("address", addr(2), true),
                    ("uint24", uint(500), true),
                    ("int24", int(10), false),
                    ("address", addr(9), false),
                ],
            ),
            &[1, 2],
        ),
        fixture(
            "uniswap v4",
            event(
                "Initialize",
                vec![
                    ("bytes32", Token::FixedBytes(vec![9; 32]), true),
                    ("address", addr(0), true),
                    ("address", addr(2), true),
                    ("uint24", uint(500), false),
                    ("int24", int(10), false),
                    ("address", addr(3), false),
                    ("uint160", uint(1), false),
                    ("int24", int(0), false),
                ],
            ),
            &[0, 2],
        ),
        fixture(
            "aerodrome",
            event(
                "PoolCreated",
                vec![
                    ("address", addr(1), true),
                    ("address", addr(2), true),
                    ("bool", Token::Bool(true), true),
                    ("address", addr(9), false),
                    ("uint256", uint(1), false),
                ],
            ),
            &[1, 2],
        ),
        // Kyber and Uniswap v3 intentionally have the same event signature and normalized fields.
        fixture(
            "kyber elastic",
            event(
                "PoolCreated",
                vec![
                    ("address", addr(1), true),
                    ("address", addr(2), true),
                    ("uint24", uint(300), true),
                    ("int24", int(60), false),
                    ("address", addr(9), false),
                ],
            ),
            &[1, 2],
        ),
        fixture(
            "traderjoe",
            event(
                "LBPairCreated",
                vec![
                    ("address", addr(1), true),
                    ("address", addr(2), true),
                    ("uint256", uint(20), true),
                    ("address", addr(9), false),
                    ("uint256", uint(1), false),
                ],
            ),
            &[1, 2],
        ),
        fixture(
            "bancor",
            event(
                "NewConverter",
                vec![("uint16", uint(1), true), ("address", addr(9), true), ("address", addr(3), true)],
            ),
            &[],
        ),
        fixture(
            "bancor legacy",
            event("NewConverter", vec![("address", addr(9), true), ("address", addr(3), true)]),
            &[],
        ),
        fixture(
            "bancor features",
            event("FeaturesAddition", vec![("address", addr(9), true), ("uint256", uint(1), false)]),
            &[],
        ),
        fixture(
            "sunpump",
            event(
                "TokenCreate",
                vec![("address", addr(9), false), ("uint256", uint(1), false), ("address", addr(3), false)],
            ),
            &[],
        ),
        fixture(
            "sunpump legacy",
            event(
                "TokenCreate",
                vec![
                    ("address", addr(9), false),
                    ("address", addr(3), false),
                    ("uint64", uint(100), false),
                    ("uint64", uint(10), false),
                    ("string", Token::String("Synthetic token".into()), false),
                    ("string", Token::String("TEST".into()), false),
                ],
            ),
            &[],
        ),
        fixture(
            "curve plain",
            event(
                "PlainPoolDeployed",
                vec![
                    ("address[]", Token::Array(vec![addr(1), addr(2), addr(3)]), false),
                    ("uint256", uint(100), false),
                    ("uint256", uint(1), false),
                    ("address", addr(4), false),
                ],
            ),
            &[1, 2, 3],
        ),
        fixture(
            "curve meta",
            event(
                "MetaPoolDeployed",
                vec![
                    ("address", addr(1), false),
                    ("address", addr(2), false),
                    ("uint256", uint(100), false),
                    ("uint256", uint(1), false),
                    ("address", addr(4), false),
                ],
            ),
            &[1],
        ),
        fixture(
            "curve crypto",
            event(
                "CryptoPoolDeployed",
                vec![
                    ("address", addr(8), false),
                    ("address[2]", Token::FixedArray(vec![addr(1), addr(2)]), false),
                    ("uint256", uint(100), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("uint256", uint(1), false),
                    ("address", addr(4), false),
                ],
            ),
            &[1, 2],
        ),
    ];
    cases.iter_mut().find(|case| case.name == "uniswap v4").unwrap().expected.address = vec![9; 32];
    for case in &mut cases {
        case.needs_create = case.name.starts_with("curve ");
    }
    let mut hooks = vec![Token::Bool(false); 10];
    hooks.push(addr(4));
    let mut balancer = fixture(
        "balancer v3",
        event(
            "PoolRegistered",
            vec![
                ("address", addr(9), true),
                ("address", addr(91), true),
                (
                    "(address,uint8,address,bool)[]",
                    Token::Array(vec![
                        Token::Tuple(vec![addr(1), uint(0), addr(0), Token::Bool(false)]),
                        Token::Tuple(vec![addr(2), uint(1), addr(3), Token::Bool(true)]),
                        Token::Tuple(vec![addr(5), uint(0), addr(0), Token::Bool(false)]),
                    ]),
                    false,
                ),
                ("uint256", uint(1), false),
                ("uint32", uint(1), false),
                ("(address,address,address)", Token::Tuple(vec![addr(3), addr(4), addr(5)]), false),
                ("(bool,bool,bool,bool,bool,bool,bool,bool,bool,bool,address)", Token::Tuple(hooks), false),
                ("(bool,bool,bool,bool)", Token::Tuple(vec![Token::Bool(false); 4]), false),
            ],
        ),
        &[1, 2, 5],
    );
    // The vault emits the log, but the event supplies the factory.
    balancer.expected.factory = address(91);
    cases.push(balancer);
    cases
}

fn receipt_transaction(logs: Vec<Log>) -> TransactionTrace {
    TransactionTrace {
        status: 1,
        to: address(90),
        receipt: Some(TransactionReceipt { logs, ..Default::default() }),
        ..Default::default()
    }
}

fn block(transactions: Vec<TransactionTrace>) -> Block {
    Block {
        transaction_traces: transactions,
        ..Default::default()
    }
}

fn create(value: u8) -> Call {
    Call {
        index: 1,
        parent_index: 0,
        depth: 1,
        call_type: CallType::Create as i32,
        address: address(value),
        ..Default::default()
    }
}

fn same(actual: &PoolEntry, expected: &PoolEntry, context: &str) {
    assert_eq!(actual.address, expected.address, "{context}: pool identity");
    assert_eq!(actual.tokens, expected.tokens, "{context}: token order");
    assert_eq!(actual.factory, expected.factory, "{context}: factory");
}

#[test]
fn all_sixteen_legacy_event_layouts_keep_their_normalized_records() {
    for fixture in fixtures() {
        let create_address = address(9);
        let create_address = fixture.needs_create.then_some(create_address.as_slice());
        let decoded = decode_log(&fixture.log, create_address).expect(fixture.name);
        same(&decoded, &fixture.expected, fixture.name);
        let topics = fixture.log.topics.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let borrowed = decode_log_bytes(&fixture.log.address, &topics, &fixture.log.data, create_address).expect(fixture.name);
        same(&borrowed, &fixture.expected, fixture.name);
        let mut trx = receipt_transaction(vec![fixture.log.clone()]);
        if fixture.needs_create {
            trx.calls = vec![
                Call {
                    logs: vec![fixture.log],
                    ..Default::default()
                },
                create(9),
            ];
        }
        let records = project(&block(vec![trx]));
        assert_eq!(records.len(), 1, "{}", fixture.name);
        same(&records[0], &fixture.expected, fixture.name);
    }
}

#[test]
fn receipt_and_trace_sources_preserve_committed_log_order_and_duplicates() {
    let mut cases = fixtures();
    let mut later = cases.remove(1).log;
    later.ordinal = 20;
    let mut earlier = cases.remove(0).log;
    earlier.ordinal = 10;
    let receipt = block(vec![receipt_transaction(vec![earlier.clone(), later.clone(), later.clone()])]);
    let mut traced = receipt_transaction(vec![]);
    traced.calls = vec![
        Call {
            logs: vec![later.clone(), later],
            ..Default::default()
        },
        Call {
            logs: vec![earlier],
            ..Default::default()
        },
    ];
    let expected = project(&receipt);
    let actual = project(&block(vec![traced]));
    assert_eq!(expected.len(), 3, "The decoder must leave first-write policy to its consumers");
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(&expected) {
        same(actual, expected, "receipt/trace parity");
    }
    assert_eq!(actual[0].tokens, vec![address(1)]);
    assert_eq!(actual[1].tokens, vec![address(1), address(2)]);
}

#[test]
fn transaction_order_is_preserved_without_globally_sorting_log_ordinals() {
    let mut cases = fixtures();
    let mut first_log = cases.remove(0).log;
    first_log.ordinal = 100;
    let mut second_log = cases.remove(0).log;
    second_log.ordinal = 1;
    let mut first = receipt_transaction(vec![first_log]);
    first.index = 1;
    let mut second = receipt_transaction(vec![second_log]);
    second.index = 2;
    let pools = project(&block(vec![first, second]));
    assert_eq!(pools.len(), 2);
    assert_eq!(pools[0].tokens, vec![address(1)]);
    assert_eq!(pools[1].tokens, vec![address(1), address(2)]);
}

#[test]
fn unified_map_keeps_log_order_instead_of_old_hosted_protocol_group_order() {
    let mut cases = fixtures();
    let v1 = cases.remove(0).log;
    let v2 = cases.remove(0).log;
    let v2_first = block(vec![receipt_transaction(vec![v2.clone(), v1.clone()])]);
    assert_eq!(project(&v2_first).len(), 2, "The native decoder retains both observations");
    let pools = dex_swaps_pools::map_pool_creations_impl(&v2_first);
    assert_eq!(pools.pools.len(), 1);
    assert_eq!(pools.pools[0].address, hex::encode(address(9)));
    assert_eq!(pools.pools[0].pool.as_ref().unwrap().tokens, vec![address(1), address(2)]);

    // Hosted 0.6 emitted the V1 protocol group before V2 regardless of log order.
    // The unified map intentionally follows the legacy store's observation order.
    let v1_first = block(vec![receipt_transaction(vec![v1, v2])]);
    assert_eq!(project(&v1_first).len(), 2);
    let pools = dex_swaps_pools::map_pool_creations_impl(&v1_first);
    assert_eq!(pools.pools.len(), 1);
    assert_eq!(pools.pools[0].pool.as_ref().unwrap().tokens, vec![address(1)]);
}

#[test]
fn only_successful_transactions_and_non_reverted_call_logs_are_admitted() {
    let log = fixtures().remove(0).log;
    let mut failed = receipt_transaction(vec![log.clone()]);
    failed.status = 2;
    let mut unknown = failed.clone();
    unknown.status = 0;
    let mut traced = receipt_transaction(vec![log.clone()]);
    traced.calls = vec![
        Call {
            logs: vec![log.clone()],
            state_reverted: true,
            ..Default::default()
        },
        Call {
            logs: vec![log.clone()],
            ..Default::default()
        },
    ];
    let result = project(&block(vec![failed, unknown, traced, receipt_transaction(vec![log])]));
    assert_eq!(result.len(), 2);
}

#[test]
fn present_calls_do_not_fall_back_to_duplicate_receipt_logs() {
    let mut trx = receipt_transaction(vec![fixtures().remove(0).log]);
    trx.calls = vec![Call::default()];
    assert!(project(&block(vec![trx])).is_empty());
    assert!(project(&Block::default()).is_empty());
}

#[test]
fn malformed_and_unrecognized_events_do_not_create_records() {
    for fixture in fixtures() {
        let mut wrong_topic = fixture.log.clone();
        wrong_topic.topics[0] = vec![0; 32];
        let mut missing_topic = fixture.log.clone();
        missing_topic.topics.clear();
        let mut short_topic = fixture.log.clone();
        short_topic.topics[0].truncate(31);
        let mut malformed_data = fixture.log;
        if !malformed_data.data.is_empty() {
            malformed_data.data.truncate(1);
        } else {
            // All-data-indexed events still require complete indexed ABI words.
            malformed_data.topics[1].truncate(1);
        }
        let logs = vec![wrong_topic, missing_topic, short_topic, malformed_data];
        let mut trx = receipt_transaction(vec![]);
        trx.calls = vec![Call { logs, ..Default::default() }, create(9)];
        assert!(project(&block(vec![trx])).is_empty(), "{}", fixture.name);
    }
}

#[test]
fn empty_curve_identity_is_omitted_but_twenty_and_thirty_two_byte_keys_remain_distinct() {
    let mut cases = fixtures();
    let curve = cases.iter().find(|case| case.name == "curve plain").unwrap().log.clone();
    let mut empty_create = create(9);
    empty_create.address.clear();
    let mut trx = receipt_transaction(vec![]);
    trx.calls = vec![
        Call {
            logs: vec![curve],
            ..Default::default()
        },
        empty_create,
    ];
    assert!(project(&block(vec![trx])).is_empty());
    let pair = cases.remove(1).log;
    let v4 = cases.into_iter().find(|case| case.name == "uniswap v4").unwrap().log;
    let pools = project(&block(vec![receipt_transaction(vec![pair, v4])]));
    assert_eq!(pools[0].address, vec![9; 20]);
    assert_eq!(pools[1].address, vec![9; 32]);
    assert_eq!(pools[1].tokens[0], vec![0; 20], "v4 native currency must remain valid");
}

#[test]
fn curve_first_create_association_is_characterized_without_claiming_provenance() {
    let curve = fixtures().into_iter().find(|case| case.name == "curve plain").unwrap().log;
    let mut trx = receipt_transaction(vec![]);
    let mut unrelated_first = create(7);
    unrelated_first.state_reverted = true;
    // Legacy behavior deliberately remains visible: the global first CREATE wins even
    // when it is unrelated or reverted. Fixing identity association is a separate change.
    trx.calls = vec![
        unrelated_first,
        create(9),
        Call {
            logs: vec![curve],
            ..Default::default()
        },
    ];
    let pools = project(&block(vec![trx]));
    assert_eq!(pools.len(), 1);
    assert_eq!(pools[0].address, address(7));
}

#[test]
fn curve_constructor_tail_remains_before_logs_and_requires_root_create() {
    // StableSwap.json constructor: owner, coins[3], pool_token, A, fee, admin_fee.
    let constructor = ethabi::encode(&[
        addr(8),
        Token::FixedArray(vec![addr(1), addr(2), addr(3)]),
        addr(4),
        uint(100),
        uint(1),
        uint(1),
    ]);
    let mut input = vec![0x60, 0x00]; // Synthetic init bytecode; no deployed-code identity claim.
    input.extend(constructor);
    let mut root = create(7);
    root.depth = 0;
    root.index = 0;
    root.logs = vec![fixtures().remove(0).log];
    let trx = TransactionTrace {
        status: 1,
        input,
        calls: vec![root],
        ..Default::default()
    };
    let pools = project(&block(vec![trx.clone()]));
    assert_eq!(pools.len(), 2);
    assert_eq!(pools[0].address, address(7));
    assert_eq!(pools[0].tokens, vec![address(1), address(2), address(3)]);
    assert!(pools[0].factory.is_empty());
    assert_eq!(pools[1].tokens, vec![address(1)]);
    let mut internal = trx;
    internal.calls[0].depth = 1;
    internal.to = address(90);
    assert_eq!(project(&block(vec![internal])).len(), 1);
}

#[test]
fn captured_mainnet_receipt_projects_the_observed_uniswap_v2_pair() {
    // This selected transaction is wrapped in a receipt-only Block. The fixture has no
    // execution traces or Firehose ordinals, and does not represent a complete block.
    let receipt: serde_json::Value = serde_json::from_str(include_str!("fixtures/uniswap-v2-pair-created-receipt.json")).unwrap();
    let provenance: serde_json::Value = serde_json::from_str(include_str!("fixtures/uniswap-v2-pair-created-provenance.json")).unwrap();
    fn bytes(value: &serde_json::Value) -> Vec<u8> {
        substreams::Hex::decode(value.as_str().unwrap().trim_start_matches("0x")).unwrap()
    }
    fn number(value: &serde_json::Value) -> u64 {
        u64::from_str_radix(value.as_str().unwrap().trim_start_matches("0x"), 16).unwrap()
    }
    assert_eq!(receipt["status"], "0x1");
    assert_eq!(receipt["blockHash"], provenance["block_hash"]);
    assert_eq!(receipt["transactionHash"], provenance["transaction_hash"]);
    let logs = receipt["logs"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(index, log)| {
            assert_eq!(log["removed"], false);
            Log {
                address: bytes(&log["address"]),
                topics: log["topics"].as_array().unwrap().iter().map(bytes).collect(),
                data: bytes(&log["data"]),
                index: index as u32,
                block_index: number(&log["logIndex"]) as u32,
                ..Default::default()
            }
        })
        .collect();
    let mut trx = receipt_transaction(logs);
    trx.hash = bytes(&receipt["transactionHash"]);
    trx.index = number(&receipt["transactionIndex"]) as u32;
    trx.to = bytes(&receipt["to"]);
    let mut selected = block(vec![trx]);
    selected.number = number(&receipt["blockNumber"]);
    selected.hash = bytes(&receipt["blockHash"]);
    let pools = project(&selected);
    assert_eq!(pools.len(), 1);
    same(
        &pools[0],
        &PoolEntry {
            address: bytes(&provenance["pool_address"]),
            tokens: provenance["tokens"].as_array().unwrap().iter().map(bytes).collect(),
            factory: bytes(&provenance["factory_address"]),
        },
        "mainnet Uniswap v2 receipt",
    );
}
