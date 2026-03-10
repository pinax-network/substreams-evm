use common::create::{CreateCall, CreateLog};
use proto::pb::erc20::transfers::v1 as pb;
use substreams_ethereum::pb::eth::v2::{BigInt, Call, CallType, Log};

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
