//! Strict decoder for independently captured raw storage, never RPC balances.
use super::{Period, State};
use crate::{
    data::{items, text},
    rpc::quantity,
};
use anyhow::{ensure, Context, Result};
use primitive_types::U256;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const TOKEN: &str = "0xe18102869d32181aea317a40c5c4ce90ca591913";
pub const HELPER: &str = "0x1430c0bd0d023690f7aee3666daf265c498cc6a5";
pub const POOL_A: &str = "0xe82c917d216d15dd88ae65300d53c2a202dd41c6";
pub const POOL_B: &str = "0x16b9a82891338f9ba80e2d6970fdda79d1eb0dae";
pub const ROUTER: &str = "0x10ed43c718714eb63d5aa57b78b54704e256024e";

pub fn key(args: &[U256], slot: u64) -> U256 {
    args.iter().fold(U256::from(slot), |root, arg| {
        let mut preimage = [0; 64];
        arg.to_big_endian(&mut preimage[..32]);
        root.to_big_endian(&mut preimage[32..]);
        U256::from_big_endian(&erc20_balances_storage::hash(&preimage))
    })
}
pub fn word(value: U256) -> String {
    format!("0x{value:064x}")
}

pub fn validate_bindings(v: &Value) -> Result<()> {
    for (address, hash) in [
        (TOKEN, "0xe904e21712652b72bb09176cbfa0ef41be3cbbf56121e292cb86bacde74b9373"),
        (HELPER, "0x1c8507bc61f3c46a5d90674b6172d2384b657059d49c3f39621a9cb10c5c06e5"),
        (ROUTER, "0x69aef35de7236f9ae83edadff736f01ea40edd8919b2e73d87dea5d3255b8c3e"),
        (POOL_A, "0x60e4bcb14447615ab7c14fda2c2d70ca4191570e8841c75618e627c8f72662f8"),
        (POOL_B, "0x60e4bcb14447615ab7c14fda2c2d70ca4191570e8841c75618e627c8f72662f8"),
    ] {
        ensure!(
            items(v, "runtime_bindings")?
                .iter()
                .filter(|r| r["address"] == address && r["runtime_hash"] == hash)
                .count()
                == 1,
            "missing or unreviewed runtime binding for {address}"
        );
    }
    Ok(())
}

pub fn decode(v: &Value) -> Result<State> {
    validate_bindings(v)?;
    ensure!(v["contract"] == TOKEN, "unreviewed token");
    let holder = quantity(&v["holder"])?;
    ensure!(holder < U256::one() << 160, "invalid holder");
    let mut raw = BTreeMap::new();
    for row in items(v, "raw_state")? {
        let key = (text(&row["contract"])?.to_owned(), quantity(&row["key"])?);
        ensure!(raw.insert(key, quantity(&row["value"])?).is_none(), "duplicate raw word");
    }
    let read = |contract: &str, k: U256| {
        raw.get(&(contract.to_owned(), k))
            .copied()
            .with_context(|| format!("missing raw word {contract} {}", word(k)))
    };
    let token = |k| read(TOKEN, k);
    let address_mask = (U256::one() << 160) - 1;
    for (contract, slot, expected) in [(TOKEN, 6, HELPER), (HELPER, 1, TOKEN), (HELPER, 2, POOL_A)] {
        ensure!(
            read(contract, U256::from(slot))? & address_mask == quantity(&json!(expected))?,
            "unreviewed stored dependency address"
        );
    }
    let base = key(&[holder], 9);
    let packed = token(base)?;
    let mut user = [U256::zero(); 13];
    user[0] = packed & address_mask;
    user[1] = (packed >> 160) & U256::from(255);
    for (offset, value) in user.iter_mut().enumerate().skip(2) {
        *value = token(base + U256::from(offset - 1))?;
    }
    let pool_holder = quantity(&json!(POOL_A))?;
    let pool_base = key(&[pool_holder], 9);
    // Read all record words returned to the nested pool getter, plus raw balance.
    token(pool_base)?;
    for offset in 1..12 {
        token(pool_base + U256::from(offset))?;
    }
    let pool_balance = token(key(&[pool_holder], 0))?;
    let pool_hour_rate = token(pool_base + U256::from(4))?;
    let pool_cursors = if pool_hour_rate.is_zero() {
        None
    } else {
        let base = key(&[pool_holder], 11);
        Some([
            token(base)?,
            token(base + U256::one())?,
            token(base + U256::from(2))?,
            token(base + U256::from(3))?,
        ])
    };
    let now = quantity(&v["timestamp"])?;
    let epoch = token(29.into())?;
    // This only plans period reads. Preserve the original time inputs so the
    // model evaluates early exits before the deployed checked subtraction. An
    // underflowing clock cannot reach any period read, so initialize no periods.
    let (current_hour, current_day) = now.checked_sub(epoch).map(|elapsed| (elapsed / 3600, elapsed / 86400)).unwrap_or_default();
    let last_hour = token(key(&[holder], 11))?;
    let last_day = token(key(&[holder], 11) + U256::one())?;
    let hour_count = if user[5].is_zero() || last_hour >= current_hour {
        0
    } else {
        (current_hour - last_hour).min(168.into()).low_u64()
    };
    let day_count = if user[5].is_zero() || user[6].is_zero() || last_day >= current_day {
        0
    } else {
        let n = current_day - last_day;
        ensure!(n <= 1000.into(), "unsupported fixture daily horizon");
        n.low_u64()
    };
    let periods = |count: u64, start: U256, total: u64, user_slot: u64, reward: u64| -> Result<Vec<Period>> {
        (0..count)
            .map(|offset| {
                let index = start + U256::from(offset);
                Ok(Period {
                    index,
                    total: token(key(&[index], total))?,
                    user: token(key(&[holder, index], user_slot))?,
                    reward: token(key(&[index], reward))?,
                })
            })
            .collect()
    };
    let p1 = read(POOL_A, 8.into())?;
    let p2 = read(POOL_B, 8.into())?;
    let mask = (U256::one() << 112) - 1;
    Ok(State {
        raw: token(key(&[holder], 0))?,
        user,
        last_hour,
        last_day,
        now,
        epoch,
        amount40: token(key(&[holder], 40))?,
        last39: token(key(&[holder], 39))?,
        claimed10: token(key(&[holder], 10))?,
        reserve0_a: p1 & mask,
        reserve1_a: (p1 >> 112) & mask,
        reserve0_b: p2 & mask,
        reserve1_b: (p2 >> 112) & mask,
        pool_hour_rate,
        pool_day_rate: token(pool_base + U256::from(5))?,
        pool_cursors,
        pool_balance,
        hours: periods(hour_count, last_hour, 23, 12, 24)?,
        days: periods(day_count, last_day, 25, 13, 27)?,
    })
}
