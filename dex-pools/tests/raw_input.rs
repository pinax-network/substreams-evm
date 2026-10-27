use dex_pools::{decode_log, decode_log_bytes, ethereum, project, Log, PoolEntry};

fn word(value: u8) -> Vec<u8> {
    let mut bytes = vec![0; 12];
    bytes.extend([value; 20]);
    bytes
}

fn pair_log() -> Log {
    Log {
        address: vec![4; 20],
        topics: vec![
            vec![
                13, 54, 72, 189, 15, 107, 168, 1, 52, 163, 59, 169, 39, 90, 197, 133, 217, 211, 21, 240, 173, 131, 85, 205, 222, 253, 227, 26, 250, 40, 208,
                233,
            ],
            word(1),
            word(2),
        ],
        data: [word(3), vec![0; 32]].concat(),
        index: 7,
        block_index: 11,
        ordinal: 42,
    }
}

#[test]
fn borrowed_transport_decoding_matches_owned_model_without_block_conversion() {
    let source = pair_log();
    let topics: Vec<&[u8]> = source.topics.iter().map(Vec::as_slice).collect();
    let actual = decode_log_bytes(&source.address, &topics, &source.data, None);
    assert_eq!(actual, decode_log(&source, None));
    assert_eq!(
        actual,
        Some(PoolEntry {
            address: vec![3; 20],
            tokens: vec![vec![1; 20], vec![2; 20]],
            factory: vec![4; 20]
        })
    );
}

#[test]
fn raw_adapter_rejects_unrelated_and_incomplete_events() {
    let source = pair_log();
    let topics: Vec<&[u8]> = source.topics.iter().map(Vec::as_slice).collect();
    assert!(decode_log_bytes(&source.address, &[], &source.data, None).is_none());
    assert!(decode_log_bytes(&source.address, &topics, &source.data[..32], None).is_none());
    let unknown = [0u8; 32];
    assert!(decode_log_bytes(&source.address, &[&unknown, &source.topics[1], &source.topics[2]], &source.data, None).is_none());
}

#[test]
fn owned_projection_preserves_duplicate_observations_for_callers_first_write_policy() {
    let block = ethereum::Block {
        transaction_traces: vec![ethereum::TransactionTrace {
            status: 1,
            receipt: Some(ethereum::TransactionReceipt {
                logs: vec![pair_log(), pair_log()],
                ..Default::default()
            }),
            ..Default::default()
        }],
        ..Default::default()
    };
    let entries = project(&block);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0], entries[1]);
}
