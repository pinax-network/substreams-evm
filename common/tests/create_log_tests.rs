use common::create::{CreateCall, CreateLog, CreateSyntheticLog, CreateTransaction};
use proto::pb::erc20::transfers::v1 as pb;
use proto::pb::native::transfers::v1 as native_pb;
use substreams_ethereum::pb::eth::v2::{BigInt, Call, CallType, Log, TransactionReceipt, TransactionTrace};

fn sample_call() -> Call {
    Call {
        index: 7,
        parent_index: 3,
        depth: 2,
        call_type: CallType::Delegate as i32,
        caller: vec![0x22; 20],
        address: vec![0x33; 20],
        value: Some(BigInt { bytes: vec![0x2a] }),
        gas_limit: 99,
        gas_consumed: 77,
        begin_ordinal: 100,
        end_ordinal: 200,
        input: vec![0xde, 0xad, 0xbe, 0xef],
        ..Default::default()
    }
}

fn sample_transaction() -> TransactionTrace {
    TransactionTrace {
        to: vec![0x77; 20],
        nonce: 9,
        gas_price: Some(BigInt { bytes: vec![0x64] }),
        gas_limit: 500,
        value: Some(BigInt { bytes: vec![0x2a] }),
        input: vec![0xca, 0xfe],
        gas_used: 321,
        hash: vec![0x88; 32],
        from: vec![0x99; 20],
        receipt: Some(TransactionReceipt::default()),
        ..Default::default()
    }
}

#[test]
fn create_call_maps_rich_call_metadata() {
    let call = sample_call();

    let created = pb::Call::create_call(&call);

    assert_eq!(created.index, 7);
    assert_eq!(created.parent_index, 3);
    assert_eq!(created.begin_ordinal, 100);
    assert_eq!(created.end_ordinal, 200);
    assert_eq!(created.caller, vec![0x22; 20]);
    assert_eq!(created.address, vec![0x33; 20]);
    assert_eq!(created.value, "42");
    assert_eq!(created.gas_limit, 99);
    assert_eq!(created.gas_consumed, 77);
    assert_eq!(created.call_type, CallType::Delegate as i32);
}

#[test]
fn create_call_maps_native_transfers_call_metadata() {
    let mut call = sample_call();
    call.call_type = CallType::Create as i32;

    let created = native_pb::Call::create_call(&call);

    assert_eq!(created.index, 7);
    assert_eq!(created.parent_index, 3);
    assert_eq!(created.value, "42");
    assert_eq!(created.call_type, native_pb::CallType::Create as i32);
}

#[test]
fn create_transaction_populates_metadata_fields() {
    let trx = sample_transaction();

    let created = pb::Transaction::create_transaction(&trx);

    assert_eq!(created.hash, vec![0x88; 32]);
    assert_eq!(created.from, vec![0x99; 20]);
    assert_eq!(created.to, Some(vec![0x77; 20]));
    assert_eq!(created.nonce, 9);
    assert_eq!(created.gas_price, "100");
    assert_eq!(created.gas_limit, 500);
    assert_eq!(created.value, "42");
}

#[test]
fn create_log_with_call_maps_native_positions_and_rich_call_metadata() {
    let log = Log {
        address: vec![0x11; 20],
        topics: vec![vec![0xaa; 32], vec![0xbb; 32]],
        data: vec![0xcc, 0xdd],
        index: 4,
        block_index: 12,
        ordinal: 42,
    };
    let call = sample_call();
    let event = pb::log::Log::Transfer(pb::Transfer {
        from: vec![0x44; 20],
        to: vec![0x55; 20],
        amount: "123".to_string(),
    });

    let created = pb::Log::create_log_with_call(&log, event, Some(&call));

    assert_eq!(created.block_index, 12);
    assert_eq!(created.ordinal, 42);

    let created_call = created.call.expect("call metadata should be populated");
    assert_eq!(created_call.index, 7);
    assert_eq!(created_call.parent_index, 3);
    assert_eq!(created_call.begin_ordinal, 100);
    assert_eq!(created_call.end_ordinal, 200);
    assert_eq!(created_call.caller, vec![0x22; 20]);
    assert_eq!(created_call.address, vec![0x33; 20]);
    assert_eq!(created_call.value, "42");
    assert_eq!(created_call.gas_limit, 99);
    assert_eq!(created_call.gas_consumed, 77);
    assert_eq!(created_call.call_type, CallType::Delegate as i32);
}

#[test]
fn create_synthetic_log_with_call_uses_empty_log_fields_and_call_metadata() {
    let call = sample_call();
    let event = pb::log::Log::Transfer(pb::Transfer {
        from: vec![0x44; 20],
        to: vec![0x55; 20],
        amount: "123".to_string(),
    });

    let created = pb::Log::create_synthetic_log_with_call(&[0x66; 20], 555, 0, event, Some(&call));

    assert_eq!(created.address, vec![0x66; 20]);
    assert_eq!(created.ordinal, 555);
    assert_eq!(created.block_index, 0);
    assert!(created.topics.is_empty());
    assert!(created.data.is_empty());

    let created_call = created.call.expect("call metadata should be populated");
    assert_eq!(created_call.index, 7);
    assert_eq!(created_call.value, "42");
}
