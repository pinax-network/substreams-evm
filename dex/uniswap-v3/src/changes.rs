use common::bigint_to_i32;
use proto::pb::uniswap::v3 as pb;
use std::collections::{BTreeMap, BTreeSet};
use substreams::{errors::Error, scalar::BigInt};
use substreams_abis::dex::uniswap::v3::pool::events;
use substreams_ethereum::{
    pb::eth::v2::{Block, Log},
    Event,
};

// Event topics from the existing generated Uniswap V3 ABI dependency.
const INITIALIZE: [u8; 32] = [
    152, 99, 96, 54, 203, 102, 169, 193, 154, 55, 67, 94, 252, 30, 144, 20, 33, 144, 33, 78, 138, 190, 184, 33, 189, 186, 63, 41, 144, 221, 76, 149,
];
const SWAP: [u8; 32] = [
    196, 32, 121, 249, 74, 99, 80, 215, 230, 35, 95, 41, 23, 73, 36, 249, 40, 204, 42, 200, 24, 235, 100, 254, 216, 0, 78, 17, 95, 188, 202, 103,
];
const MINT: [u8; 32] = [
    122, 83, 8, 11, 164, 20, 21, 139, 231, 236, 105, 185, 135, 181, 251, 125, 7, 222, 225, 1, 254, 133, 72, 143, 8, 83, 174, 22, 35, 157, 11, 222,
];
const BURN: [u8; 32] = [
    12, 57, 108, 217, 137, 163, 159, 68, 89, 181, 250, 26, 237, 106, 154, 141, 205, 188, 69, 144, 138, 207, 214, 126, 2, 140, 213, 104, 218, 152, 152, 44,
];

fn uint(value: &BigInt, bits: usize) -> Result<String, Error> {
    let text = value.to_string();
    let bytes = value.to_bytes_be().1;
    if text.starts_with('-') || bytes.len() > bits / 8 {
        return Err(Error::msg("invalid unsigned pool state"));
    }
    Ok(text)
}
fn tick(value: &BigInt) -> Result<i32, Error> {
    bigint_to_i32(value)
        .filter(|n| (-8_388_608..=8_388_607).contains(n))
        .ok_or_else(|| Error::msg("invalid int24 tick"))
}
fn decode<E: Event>(log: &Log) -> Result<E, Error> {
    if !E::match_log(log) {
        return Err(Error::msg("invalid event shape"));
    }
    E::decode(log).map_err(|_| Error::msg("invalid event encoding"))
}
fn change(log: &Log, topic: &[u8]) -> Result<pb::pool_change::Change, Error> {
    use pb::pool_change::Change;
    if topic == INITIALIZE {
        let value = decode::<events::Initialize>(log)?;
        Ok(Change::Initialize(pb::PoolPriceState {
            sqrt_price_x96: uint(&value.sqrt_price_x96, 160)?,
            tick: tick(&value.tick)?,
            liquidity: "0".into(),
        }))
    } else if topic == SWAP {
        let value = decode::<events::Swap>(log)?;
        Ok(Change::Swap(pb::PoolPriceState {
            sqrt_price_x96: uint(&value.sqrt_price_x96, 160)?,
            tick: tick(&value.tick)?,
            liquidity: uint(&value.liquidity, 128)?,
        }))
    } else {
        let (lower, upper, amount, burn) = if topic == MINT {
            let value = decode::<events::Mint>(log)?;
            (value.tick_lower, value.tick_upper, value.amount, false)
        } else {
            let value = decode::<events::Burn>(log)?;
            (value.tick_lower, value.tick_upper, value.amount, true)
        };
        let amount = uint(&amount, 128)?;
        Ok(Change::Liquidity(pb::PoolLiquidityChange {
            tick_lower: tick(&lower)?,
            tick_upper: tick(&upper)?,
            liquidity_delta: if burn && amount != "0" { format!("-{amount}") } else { amount },
        }))
    }
}

/// Complete blocks, including empty ones. These ordered changes require a
/// canonical initial state; the last Swap alone is not closing liquidity.
pub(crate) fn extract(block: &Block) -> Result<pb::BlockPoolChanges, Error> {
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
    let mut pools = BTreeMap::<Vec<u8>, Vec<pb::PoolChange>>::new();
    let mut positions = BTreeSet::new();
    for transaction in block.transactions() {
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
            let Some(topic) = log.topics.first() else {
                continue;
            };
            let name = if topic == &INITIALIZE {
                "Initialize"
            } else if topic == &SWAP {
                "Swap"
            } else if topic == &MINT {
                "Mint"
            } else if topic == &BURN {
                "Burn"
            } else {
                continue;
            };
            if log.address.len() != 20 || !positions.insert(log.block_index) {
                return Err(Error::msg("invalid or duplicate canonical log position"));
            }
            // An unrelated contract can emit a malformed matching topic. Preserve
            // a marker for that pool without poisoning other pools' output.
            let state = change(log, topic).unwrap_or_else(|_| pb::pool_change::Change::Invalid(pb::InvalidPoolChange { event_name: name.into() }));
            pools.entry(log.address.clone()).or_default().push(pb::PoolChange {
                block_log_index: log.block_index,
                ordinal: log.ordinal,
                change: Some(state),
            });
        }
    }
    let pools = pools
        .into_iter()
        .map(|(pool, mut changes)| {
            changes.sort_by_key(|c| c.block_log_index);
            pb::PoolChanges { pool, changes }
        })
        .collect();
    Ok(pb::BlockPoolChanges {
        block_number: block.number,
        block_hash: block.hash.clone(),
        parent_hash: header.parent_hash.clone(),
        timestamp_seconds: timestamp.seconds,
        timestamp_nanos: timestamp.nanos,
        pools,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use substreams_ethereum::pb::eth::v2::{BlockHeader, Call, TransactionReceipt, TransactionTrace};
    fn unsigned(value: u128) -> Vec<u8> {
        [vec![0; 16], value.to_be_bytes().to_vec()].concat()
    }
    fn signed(value: i32) -> Vec<u8> {
        [vec![if value < 0 { 255 } else { 0 }; 28], value.to_be_bytes().to_vec()].concat()
    }
    fn log(topic: [u8; 32], pool: u8, index: u32, data: Vec<Vec<u8>>, indexed: Vec<Vec<u8>>) -> Log {
        let mut topics = vec![topic.to_vec()];
        topics.extend(indexed);
        Log {
            address: vec![pool; 20],
            block_index: index,
            ordinal: 1000 - u64::from(index),
            topics,
            data: data.concat(),
            ..Default::default()
        }
    }
    fn swap(pool: u8, index: u32) -> Log {
        log(
            SWAP,
            pool,
            index,
            vec![unsigned(1), unsigned(2), unsigned(1u128 << 96), unsigned(u128::MAX), signed(-1)],
            vec![unsigned(1), unsigned(2)],
        )
    }
    fn mint(index: u32, amount: u128) -> Log {
        log(
            MINT,
            1,
            index,
            vec![unsigned(1), unsigned(amount), unsigned(0), unsigned(0)],
            vec![unsigned(2), signed(-120), signed(120)],
        )
    }
    fn burn(index: u32, amount: u128) -> Log {
        log(
            BURN,
            1,
            index,
            vec![unsigned(amount), unsigned(0), unsigned(0)],
            vec![unsigned(2), signed(-120), signed(120)],
        )
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
    fn preserves_trailing_liquidity_changes_in_canonical_order_without_float_conversion() {
        let input = block(vec![burn(9, 7), swap(1, 2), mint(5, u128::MAX)]);
        let out = extract(&input).unwrap();
        let changes = &out.pools[0].changes;
        assert_eq!(changes.iter().map(|c| c.block_log_index).collect::<Vec<_>>(), vec![2, 5, 9]);
        let Some(pb::pool_change::Change::Swap(state)) = &changes[0].change else {
            panic!("swap missing")
        };
        assert_eq!(state.liquidity, "340282366920938463463374607431768211455");
        assert_eq!(state.sqrt_price_x96, "79228162514264337593543950336");
        assert_eq!(state.tick, -1);
        let Some(pb::pool_change::Change::Liquidity(mint)) = &changes[1].change else {
            panic!("mint missing")
        };
        assert_eq!(mint.liquidity_delta, u128::MAX.to_string());
        assert_eq!((mint.tick_lower, mint.tick_upper), (-120, 120));
        let Some(pb::pool_change::Change::Liquidity(burn)) = &changes[2].change else {
            panic!("burn missing")
        };
        assert_eq!(burn.liquidity_delta, "-7");
    }
    #[test]
    fn invalid_pool_event_is_explicit_and_does_not_hide_an_unrelated_valid_pool() {
        let mut invalid = swap(1, 0);
        invalid.data.pop();
        let mut overflow = swap(3, 2);
        overflow.data[96] = 1; // uint128 must not exceed 16 bytes
        let out = extract(&block(vec![invalid, swap(2, 1), overflow])).unwrap();
        assert!(matches!(&out.pools[0].changes[0].change,Some(pb::pool_change::Change::Invalid(v)) if v.event_name=="Swap"));
        assert!(matches!(&out.pools[1].changes[0].change, Some(pb::pool_change::Change::Swap(_))));
        assert!(matches!(&out.pools[2].changes[0].change, Some(pb::pool_change::Change::Invalid(_))));
    }
    #[test]
    fn initialize_and_zero_burn_are_preserved() {
        let init = log(INITIALIZE, 1, 0, vec![unsigned(1u128 << 96), signed(0)], vec![]);
        let out = extract(&block(vec![init, burn(1, 0)])).unwrap();
        assert!(matches!(&out.pools[0].changes[0].change,Some(pb::pool_change::Change::Initialize(v)) if v.liquidity=="0"));
        assert!(matches!(&out.pools[0].changes[1].change,Some(pb::pool_change::Change::Liquidity(v)) if v.liquidity_delta=="0"));
    }
    #[test]
    fn reverts_and_failed_transactions_never_change_state() {
        let mut input = block(vec![swap(1, 0)]);
        input.transaction_traces[0].calls = vec![
            Call {
                logs: vec![swap(1, 0)],
                ..Default::default()
            },
            Call {
                state_reverted: true,
                logs: vec![burn(1, 50)],
                ..Default::default()
            },
        ];
        input.transaction_traces.push(TransactionTrace {
            status: 2,
            receipt: Some(TransactionReceipt {
                logs: vec![mint(2, 100)],
                ..Default::default()
            }),
            ..Default::default()
        });
        let out = extract(&input).unwrap();
        assert_eq!(out.pools[0].changes.len(), 1);
        assert_eq!(out.pools[0].changes[0].block_log_index, 0);
    }
    #[test]
    fn empty_blocks_have_complete_identity_and_corrupt_positions_are_rejected() {
        let mut input = block(vec![]);
        input.header.as_mut().unwrap().timestamp.as_mut().unwrap().nanos = 987654321;
        let out = extract(&input).unwrap();
        assert!(out.pools.is_empty());
        assert_eq!(out.parent_hash, vec![2; 32]);
        assert_eq!(out.timestamp_nanos, 987654321);
        assert!(extract(&block(vec![swap(1, 0), swap(2, 0)])).is_err());
        input.transaction_traces[0].receipt = None;
        assert!(extract(&input).is_err());
    }
}
