//! Host-only diagnostic for the reviewed YBC reward dependency runtime.
//!
//! This does not qualify a production layout or retain holder state. Inputs must
//! be independently initialized and bound to the reviewed token/dependency/pool
//! runtimes. Historical RPC expectations never become model inputs.
use anyhow::{ensure, Context, Result};
use primitive_types::U256;

#[doc(hidden)]
pub mod fixture;

#[derive(Clone, Debug, Default)]
pub struct Hour {
    pub no_burn: bool,
    pub total_rate: U256,
    pub user_rate: U256,
    pub reward: U256,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub raw: U256,
    pub static_rate: U256,
    pub wbnb_value: U256,
    pub lp_reward: U256,
    pub now: U256,
    pub launch_time: U256,
    pub last_cycle: U256,
    pub initial_total_rate: U256,
    pub initial_user_rate: U256,
    pub pool_balance: U256,
    pub burn_per_day: U256,
    pub token_reserve: U256,
    pub quote_reserve: U256,
    /// Consecutive hours beginning at last_cycle, including skipped hours.
    pub hours: Vec<Hour>,
}

fn add(a: U256, b: U256) -> Result<U256> {
    a.checked_add(b).context("uint256 addition overflow")
}
fn mul(a: U256, b: U256) -> Result<U256> {
    a.checked_mul(b).context("uint256 multiplication overflow")
}
fn sub(a: U256, b: U256) -> Result<U256> {
    a.checked_sub(b).context("uint256 subtraction underflow")
}

fn quote(amount: U256, input: U256, output: U256) -> Result<U256> {
    if amount.is_zero() {
        return Ok(U256::zero());
    }
    ensure!(!input.is_zero() && !output.is_zero(), "INSUFFICIENT_LIQUIDITY");
    Ok(mul(amount, output)? / input)
}

fn preview_burn(pool: U256, per_day: U256) -> Result<(U256, U256)> {
    if pool.is_zero() {
        return Ok((U256::zero(), U256::zero()));
    }
    let burned = mul(pool, per_day / 24)? / 1_000_000;
    if burned.is_zero() {
        return Ok((U256::zero(), U256::zero()));
    }
    let reward = mul(burned, 2.into())? / 3;
    let remaining = sub(pool, add(reward, reward / 2)?)?;
    Ok((reward, remaining))
}

impl State {
    /// Returns (reward, stopping_hour), preserving deployed early-exit order.
    pub fn reward(&self) -> Result<(U256, U256)> {
        if self.static_rate < U256::from(1_000_000_000u64) {
            return Ok((U256::zero(), U256::zero()));
        }
        let current = sub(self.now, self.launch_time)? / 3600;
        if current.is_zero() || self.last_cycle >= current {
            return Ok((U256::zero(), U256::zero()));
        }
        let stop = current.min(add(self.last_cycle, 240.into())?);
        let count = (stop - self.last_cycle).low_u64() as usize;
        ensure!(self.hours.len() >= count, "missing initialized hourly state");
        ensure!(
            self.hours[0].total_rate == self.initial_total_rate && self.hours[0].user_rate == self.initial_user_rate,
            "initial rate differs from its first hourly storage word"
        );
        let mut total_rate = self.initial_total_rate;
        let mut user_rate = self.initial_user_rate;
        let mut pool = self.pool_balance;
        let mut sum = U256::zero();
        for hour in self.hours.iter().take(count) {
            if hour.no_burn {
                continue;
            }
            if !hour.total_rate.is_zero() {
                total_rate = hour.total_rate;
            }
            if !hour.user_rate.is_zero() {
                user_rate = hour.user_rate;
            }
            let mut reward = hour.reward;
            if reward.is_zero() && !total_rate.is_zero() {
                (reward, pool) = preview_burn(pool, self.burn_per_day)?;
            }
            if !total_rate.is_zero() && !reward.is_zero() && !user_rate.is_zero() {
                sum = add(sum, mul(reward, user_rate)? / total_rate)?;
            }
        }
        let quoted = quote(sum, self.token_reserve, self.quote_reserve)?;
        let cap = mul(self.wbnb_value, 13.into())? / 10;
        if add(self.lp_reward, quoted)? > cap {
            if self.lp_reward > cap {
                return Ok((U256::zero(), U256::zero()));
            }
            sum = quote(sub(cap, self.lp_reward)?, self.quote_reserve, self.token_reserve)?;
        }
        Ok((sum, stop))
    }

    pub fn balance(&self) -> Result<U256> {
        add(self.raw, self.reward()?.0)
    }
}

#[cfg(test)]
mod tests;
