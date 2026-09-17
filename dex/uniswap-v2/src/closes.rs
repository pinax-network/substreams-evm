use proto::pb::uniswap::v2 as pb;
use std::collections::{BTreeMap, BTreeSet};
use substreams::errors::Error;
use substreams_abis::dex::uniswap::v2::pair::events::Sync;
use substreams_ethereum::{pb::eth::v2::Block, Event};

/// Collect complete-block closing reserve observations without interpreting
/// token identities, decimal scaling, liquidity qualification or USD prices.
pub(crate) fn extract(block: &Block) -> Result<pb::BlockPoolCloses, Error> {
    let header = block.header.as_ref().ok_or_else(|| Error::msg("missing block header"))?;
    let timestamp = header.timestamp.as_ref().ok_or_else(|| Error::msg("missing block timestamp"))?;
    if block.hash.len() != 32
        || header.parent_hash.len() != 32
        || header.number != block.number
        || timestamp.seconds < 0
        || !(0..1_000_000_000).contains(&timestamp.nanos)
    {
        return Err(Error::msg("invalid block identity or timestamp"));
    }
    let mut pools = BTreeMap::<Vec<u8>, pb::PoolClose>::new();
    let mut positions = BTreeSet::new();
    for transaction in block.transactions() {
        // Call traces carry reverted logs, so use the SDK's state-revert filter.
        // BASE models have receipt logs instead. Never read both representations.
        let logs: Vec<_> = if transaction.calls.is_empty() {
            transaction
                .receipt
                .as_ref()
                .ok_or_else(|| Error::msg("missing transaction receipt"))?
                .logs
                .iter()
                .collect()
        } else {
            transaction.logs_with_calls().map(|(log, _)| log).collect()
        };
        for log in logs {
            let Some(sync) = Sync::match_and_decode(log) else {
                continue;
            };
            if log.address.len() != 20 || !positions.insert(log.block_index) {
                return Err(Error::msg("invalid or duplicate canonical log position"));
            }
            if pools.get(&log.address).is_some_and(|old| old.block_log_index > log.block_index) {
                continue;
            }
            pools.insert(
                log.address.clone(),
                pb::PoolClose {
                    pool: log.address.clone(),
                    reserve0: sync.reserve0.to_string(),
                    reserve1: sync.reserve1.to_string(),
                    block_log_index: log.block_index,
                    ordinal: log.ordinal,
                },
            );
        }
    }
    Ok(pb::BlockPoolCloses {
        block_number: block.number,
        block_hash: block.hash.clone(),
        parent_hash: header.parent_hash.clone(),
        timestamp_seconds: timestamp.seconds,
        timestamp_nanos: timestamp.nanos,
        pools: pools.into_values().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use substreams_ethereum::pb::eth::v2::{BlockHeader, Call, Log, TransactionReceipt, TransactionTrace};

    fn sync(pool: u8, index: u32, reserve0: u128, reserve1: u128) -> Log {
        // Sync(uint112,uint112), encoded as two unsigned ABI words.
        let mut data = vec![];
        for reserve in [reserve0, reserve1] {
            data.extend([0u8; 16]);
            data.extend(reserve.to_be_bytes());
        }
        Log {
            address: vec![pool; 20],
            block_index: index,
            ordinal: u64::from(index) * 10,
            topics: vec![vec![
                0x1c, 0x41, 0x1e, 0x9a, 0x96, 0xe0, 0x71, 0x24, 0x1c, 0x2f, 0x21, 0xf7, 0x72, 0x6b, 0x17, 0xae, 0x89, 0xe3, 0xca, 0xb4, 0xc7, 0x8b, 0xe5, 0x0e,
                0x06, 0x2b, 0x03, 0xa9, 0xff, 0xfb, 0xba, 0xd1,
            ]],
            data,
            ..Default::default()
        }
    }
    fn block(logs: Vec<Log>) -> Block {
        Block {
            number: 100,
            hash: vec![1; 32],
            header: Some(BlockHeader {
                number: 100,
                parent_hash: vec![2; 32],
                timestamp: Some(Default::default()),
                ..Default::default()
            }),
            transaction_traces: vec![TransactionTrace {
                status: 1,
                receipt: Some(TransactionReceipt { logs, ..Default::default() }),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn retains_only_final_reserves_after_a_round_trip_and_preserves_empty_liquidity() {
        let output = extract(&block(vec![
            sync(1, 0, 1000, 1000),
            sync(1, 1, 1, 1000000),
            sync(1, 2, 1000, 1000),
            sync(2, 3, 0, 0),
        ]))
        .unwrap();
        assert_eq!(output.pools.len(), 2);
        assert_eq!(output.pools[0].reserve0, "1000");
        assert_eq!(output.pools[0].block_log_index, 2);
        assert_eq!(output.pools[1].reserve0, "0");
    }

    #[test]
    fn log_order_is_canonical_and_reserves_do_not_pass_through_floating_point() {
        let amount = (1u128 << 112) - 1;
        let output = extract(&block(vec![sync(1, 8, amount, 1), sync(1, 2, 10, 10)])).unwrap();
        assert_eq!(output.pools[0].reserve0, "5192296858534827628530496329220095");
        assert_eq!(output.pools[0].block_log_index, 8);
    }

    #[test]
    fn failed_transactions_and_reverted_call_logs_never_set_the_close() {
        let mut input = block(vec![sync(1, 0, 100, 100)]);
        input.transaction_traces[0].calls = vec![
            Call {
                logs: vec![sync(1, 0, 100, 100)],
                ..Default::default()
            },
            Call {
                state_reverted: true,
                logs: vec![sync(1, 2, 1, 100000)],
                ..Default::default()
            },
        ];
        input.transaction_traces.push(TransactionTrace {
            status: 2,
            receipt: Some(TransactionReceipt {
                logs: vec![sync(1, 3, 1, 100000)],
                ..Default::default()
            }),
            ..Default::default()
        });
        let output = extract(&input).unwrap();
        assert_eq!(output.pools[0].reserve0, "100");
        assert_eq!(output.pools[0].block_log_index, 0);
    }

    #[test]
    fn emits_empty_complete_blocks_with_parent_and_subsecond_time() {
        let mut input = block(vec![]);
        let timestamp = input.header.as_mut().unwrap().timestamp.as_mut().unwrap();
        timestamp.seconds = 1_700_000_000;
        timestamp.nanos = 123_456_789;
        let output = extract(&input).unwrap();
        assert!(output.pools.is_empty());
        assert_eq!(output.block_hash, input.hash);
        assert_eq!(output.parent_hash, vec![2; 32]);
        assert_eq!(output.timestamp_nanos, 123_456_789);
    }

    #[test]
    fn incomplete_identity_and_duplicate_positions_fail_instead_of_guessing() {
        assert!(extract(&block(vec![sync(1, 0, 1, 1), sync(2, 0, 2, 2)])).is_err());
        let mut input = block(vec![]);
        input.header = None;
        assert!(extract(&input).is_err());
        let mut input = block(vec![]);
        input.transaction_traces[0].receipt = None;
        assert!(extract(&input).is_err());
    }
}
