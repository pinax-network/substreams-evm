//! Host-only model of the reviewed OG initialized hourly/daily reward paths.
//!
//! This is not a production layout or a retained-state implementation. The
//! caller must bind token, helper, router and pools to their reviewed runtimes.
//! Raw state and block time are inputs; RPC return values are expectations only.
use anyhow::{ensure, Context, Result};
use primitive_types::U256;

#[doc(hidden)]
#[path = "og_model/fixture.rs"]
pub mod fixture;

#[derive(Clone, Debug)]
pub struct Period {
    pub index: U256,
    pub total: U256,
    pub user: U256,
    pub reward: U256,
}

#[derive(Clone, Debug)]
pub struct State {
    pub raw: U256,
    /// ABI words from a87430ba(holder), decoded from mapping 9's packed record.
    pub user: [U256; 13],
    pub last_hour: U256,
    pub last_day: U256,
    pub now: U256,
    pub epoch: U256,
    pub amount40: U256,
    pub last39: U256,
    pub claimed10: U256,
    pub reserve0_a: U256,
    pub reserve1_a: U256,
    pub reserve0_b: U256,
    pub reserve1_b: U256,
    /// The nested pool balanceOf must take both helper early exits. A pool with
    /// its own reward rate requires a recursive model and is unsupported.
    pub pool_hour_rate: U256,
    pub pool_day_rate: U256,
    pub hours: Vec<Period>,
    pub days: Vec<Period>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Evaluation {
    pub hourly: U256,
    pub stopping_hour: U256,
    pub daily: U256,
    pub balance: U256,
}

fn add(a: U256, b: U256) -> Result<U256> {
    a.checked_add(b).context("uint256 addition overflow")
}
fn sub(a: U256, b: U256) -> Result<U256> {
    a.checked_sub(b).context("uint256 subtraction underflow")
}
fn mul(a: U256, b: U256) -> Result<U256> {
    a.checked_mul(b).context("uint256 multiplication overflow")
}
fn div(a: U256, b: U256) -> Result<U256> {
    ensure!(!b.is_zero(), "uint256 division by zero");
    Ok(a / b)
}
fn amount_out(amount: U256, input: U256, output: U256) -> Result<U256> {
    // Router's SafeMath also rejects overflow; its revert ABI differs from the
    // token/helper Solidity panic. This model does not claim revert ABI parity.
    ensure!(!amount.is_zero(), "router INSUFFICIENT_INPUT_AMOUNT");
    ensure!(!input.is_zero() && !output.is_zero(), "router INSUFFICIENT_LIQUIDITY");
    let amount_with_fee = mul(amount, 9975.into())?;
    div(mul(amount_with_fee, output)?, add(mul(input, 10000.into())?, amount_with_fee)?)
}

impl State {
    pub fn price(&self) -> Result<U256> {
        let first = amount_out(1_000_000_000_000_000_000u64.into(), self.reserve1_a, self.reserve0_a)?;
        amount_out(first, self.reserve1_b, self.reserve0_b)
    }

    fn current_hour(&self) -> Result<U256> {
        Ok(sub(self.now, self.epoch)? / 3600)
    }

    fn pool_early_exit(&self) -> Result<()> {
        // Hourly requires field 5 == 0. That also takes the daily early exit;
        // retaining field 6 makes the captured dependency explicit.
        ensure!(self.pool_hour_rate.is_zero(), "unmodeled recursive pool reward");
        Ok(())
    }

    fn cap(&self, reward: U256) -> Result<U256> {
        let current_hour = self.current_hour()?;
        // Keep deployed short-circuit and checked-arithmetic order.
        if !self.user[12].is_zero() {
            if sub(current_hour, self.last39)? > 168.into() {
                return Ok(U256::zero());
            }
            let required = mul(self.user[12], 5.into())?;
            let provided = mul(self.amount40, 100.into())?;
            if provided < required {
                return Ok(U256::zero());
            }
        }
        let cap = mul(self.user[3], 3.into())?;
        if cap <= self.claimed10 {
            return Ok(U256::zero());
        }
        let remaining = sub(cap, self.claimed10)?;
        let price = self.price()?;
        let unit = U256::from(1_000_000_000_000_000_000u64);
        let existing = div(mul(self.user[12], price)?, unit)?;
        if existing >= remaining {
            return Ok(U256::zero());
        }
        let valued = div(mul(reward, price)?, unit)?;
        if add(existing, valued)? > remaining {
            return div(mul(sub(remaining, existing)?, unit)?, price);
        }
        Ok(reward)
    }

    /// Selector e8e8fe04(holder): reward and stopping hour.
    pub fn hourly(&self) -> Result<(U256, U256)> {
        if self.user[5].is_zero() {
            return Ok((U256::zero(), U256::zero()));
        }
        let current = self.current_hour()?;
        if current.is_zero() || self.last_hour >= current {
            return Ok((U256::zero(), U256::zero()));
        }
        let stop = current.min(add(self.last_hour, 168.into())?);
        let count = (stop - self.last_hour).low_u64() as usize;
        ensure!(self.hours.len() >= count, "missing initialized hourly state");
        self.pool_early_exit()?;
        let mut total = self.hours[0].total;
        let mut user = if self.hours[0].user.is_zero() { self.user[5] } else { self.hours[0].user };
        let mut sum = U256::zero();
        for (offset, p) in self.hours.iter().take(count).enumerate() {
            ensure!(p.index == add(self.last_hour, offset.into())?, "nonconsecutive hourly state");
            if !p.total.is_zero() {
                total = p.total;
            }
            if !p.user.is_zero() {
                user = p.user;
            }
            ensure!(!p.reward.is_zero() || total.is_zero(), "unmodeled hourly zero-reward burn preview");
            if !total.is_zero() && !user.is_zero() && !p.reward.is_zero() {
                sum = add(sum, div(mul(p.reward, user)?, total)?)?;
            }
        }
        Ok((self.cap(sum)?, stop))
    }

    /// Selector 7ac1f9f8(holder). Zero initial daily rate is not yet decoded.
    pub fn daily(&self) -> Result<U256> {
        if self.user[5].is_zero() || self.user[6].is_zero() {
            return Ok(U256::zero());
        }
        let current = sub(self.now, self.epoch)? / 86400;
        if current.is_zero() || self.last_day >= current {
            return Ok(U256::zero());
        }
        let first = self.days.first().context("missing initialized daily state")?;
        let mut total = first.total;
        let mut user = first.user;
        ensure!(!user.is_zero(), "unmodeled initial daily zero-rate branch");
        self.pool_early_exit()?;
        let mut sum = U256::zero();
        let mut positive = 0;
        let mut index = self.last_day;
        for p in &self.days {
            if index >= current {
                break;
            }
            ensure!(p.index == index, "nonconsecutive daily state");
            if !p.total.is_zero() {
                total = p.total;
            }
            if !p.user.is_zero() {
                user = p.user;
            }
            ensure!(!p.reward.is_zero() || total.is_zero(), "unmodeled daily zero-reward burn preview");
            if !total.is_zero() && !user.is_zero() && !p.reward.is_zero() {
                sum = add(sum, div(mul(p.reward, user)?, total)?)?;
                positive += 1;
            }
            index = add(index, U256::one())?;
            if positive > 2 {
                break;
            }
        }
        ensure!(index >= current || positive > 2, "missing initialized daily state");
        self.cap(sum)
    }

    pub fn evaluate(&self) -> Result<Evaluation> {
        let (hourly, stopping_hour) = self.hourly()?;
        // The token adds each result before calling the next helper.
        let subtotal = add(self.raw, hourly)?;
        let daily = self.daily()?;
        Ok(Evaluation {
            hourly,
            stopping_hour,
            daily,
            balance: add(subtotal, daily)?,
        })
    }
}

#[cfg(test)]
#[path = "og_model/tests.rs"]
mod tests;
