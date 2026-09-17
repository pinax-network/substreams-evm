use proto::pb::{
    dex::pool_state::v1::BlockPoolState,
    uniswap::{v2, v3},
};
use substreams::errors::Error;

#[substreams::handlers::map]
fn map_pool_state(v2: v2::BlockPoolCloses, v3: v3::BlockPoolChanges) -> Result<BlockPoolState, Error> {
    combine(v2, v3)
}

fn combine(v2: v2::BlockPoolCloses, v3: v3::BlockPoolChanges) -> Result<BlockPoolState, Error> {
    if v2.block_hash.len() != 32
        || v2.parent_hash.len() != 32
        || v2.timestamp_seconds < 0
        || !(0..1_000_000_000).contains(&v2.timestamp_nanos)
        || v2.block_number != v3.block_number
        || v2.block_hash != v3.block_hash
        || v2.parent_hash != v3.parent_hash
        || v2.timestamp_seconds != v3.timestamp_seconds
        || v2.timestamp_nanos != v3.timestamp_nanos
    {
        return Err(Error::msg("protocol outputs do not describe the same complete block"));
    }
    Ok(BlockPoolState {
        block_number: v2.block_number,
        block_hash: v2.block_hash,
        parent_hash: v2.parent_hash,
        timestamp_seconds: v2.timestamp_seconds,
        timestamp_nanos: v2.timestamp_nanos,
        v2_pools: v2.pools,
        v3_pools: v3.pools,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    fn fixture() -> (v2::BlockPoolCloses, v3::BlockPoolChanges) {
        let v2 = v2::BlockPoolCloses {
            block_number: 100,
            block_hash: vec![1; 32],
            parent_hash: vec![2; 32],
            timestamp_seconds: 123,
            timestamp_nanos: 456,
            pools: vec![v2::PoolClose {
                pool: vec![3; 20],
                reserve0: "5192296858534827628530496329220095".into(),
                reserve1: "0".into(),
                block_log_index: 1,
                ordinal: 10,
            }],
        };
        let v3 = v3::BlockPoolChanges {
            block_number: v2.block_number,
            block_hash: v2.block_hash.clone(),
            parent_hash: v2.parent_hash.clone(),
            timestamp_seconds: v2.timestamp_seconds,
            timestamp_nanos: v2.timestamp_nanos,
            pools: vec![v3::PoolChanges {
                pool: vec![4; 20],
                changes: vec![
                    v3::PoolChange {
                        block_log_index: 2,
                        ordinal: 20,
                        change: Some(v3::pool_change::Change::Swap(v3::PoolPriceState {
                            sqrt_price_x96: "79228162514264337593543950336".into(),
                            tick: -1,
                            liquidity: "10".into(),
                        })),
                    },
                    v3::PoolChange {
                        block_log_index: 3,
                        ordinal: 30,
                        change: Some(v3::pool_change::Change::Liquidity(v3::PoolLiquidityChange {
                            tick_lower: -100,
                            tick_upper: 100,
                            liquidity_delta: "-7".into(),
                        })),
                    },
                    v3::PoolChange {
                        block_log_index: 4,
                        ordinal: 40,
                        change: Some(v3::pool_change::Change::Invalid(v3::InvalidPoolChange { event_name: "Swap".into() })),
                    },
                ],
            }],
        };
        (v2, v3)
    }
    #[test]
    fn both_protocols_round_trip_without_losing_precision_order_or_invalid_markers() {
        let (v2, v3) = fixture();
        let output = combine(v2.clone(), v3.clone()).unwrap();
        let output = BlockPoolState::decode(output.encode_to_vec().as_slice()).unwrap();
        assert_eq!(output.v2_pools, v2.pools);
        assert_eq!(output.v3_pools, v3.pools);
        assert_eq!(output.timestamp_nanos, 456);
    }
    #[test]
    fn empty_protocol_outputs_still_emit_the_complete_block() {
        for empty_v2 in [true, false] {
            for empty_v3 in [true, false] {
                let (mut v2, mut v3) = fixture();
                if empty_v2 {
                    v2.pools.clear();
                }
                if empty_v3 {
                    v3.pools.clear();
                }
                let output = combine(v2, v3).unwrap();
                assert_eq!(output.block_number, 100);
                assert_eq!(output.v2_pools.is_empty(), empty_v2);
                assert_eq!(output.v3_pools.is_empty(), empty_v3);
            }
        }
    }
    #[test]
    fn every_header_mismatch_and_invalid_envelope_is_rejected() {
        for mutation in 0..9 {
            let (mut v2, mut v3) = fixture();
            match mutation {
                0 => v3.block_number += 1,
                1 => v3.block_hash[0] ^= 1,
                2 => v3.parent_hash[0] ^= 1,
                3 => v3.timestamp_seconds += 1,
                4 => v3.timestamp_nanos += 1,
                5 => {
                    v2.block_hash.clear();
                    v3.block_hash.clear();
                }
                6 => {
                    v2.parent_hash.clear();
                    v3.parent_hash.clear();
                }
                7 => {
                    v2.timestamp_seconds = -1;
                    v3.timestamp_seconds = -1;
                }
                _ => {
                    v2.timestamp_nanos = 1_000_000_000;
                    v3.timestamp_nanos = 1_000_000_000;
                }
            }
            assert!(combine(v2, v3).is_err(), "accepted mutation {mutation}");
        }
    }
}
