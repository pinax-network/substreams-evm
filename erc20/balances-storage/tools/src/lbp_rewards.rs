//! Experimental host-only model of LBP's runtime-bound reward getter.
//!
//! This is diagnostic arithmetic, not a qualified production layout or holder
//! store. Callers must supply checkpointed state, persisted updates and the
//! runtime's exempt addresses. No RPC value is used to repair replay state.
use anyhow::{ensure, Context, Result};
use primitive_types::U256;

mod decay;
#[doc(hidden)]
pub mod fixture;

pub fn unit() -> U256 {
    U256::from(1_000_000_000_000_000_000u64)
}

fn mul(a: U256, b: U256) -> Result<U256> {
    a.checked_mul(b).context("uint256 multiplication overflow")
}

fn add(a: U256, b: U256) -> Result<U256> {
    a.checked_add(b).context("uint256 addition overflow")
}

#[derive(Clone, Debug, Default)]
pub struct Globals {
    pub trading_opened: bool,
    pub open_time: u64,
    pub last_update: u64,
    pub reserve: U256,
    pub supply: U256,
    pub nodes: U256,
    pub static_acc: U256,
    pub node_acc: U256,
    pub accounted: U256,
}

impl Globals {
    /// Decode the verified hLBP slot 6 (bool, uint64, uint64, uint112).
    pub fn set_packed(&mut self, packed: U256) {
        self.trading_opened = !(packed & U256::from(255)).is_zero();
        self.open_time = (packed >> 8).low_u64();
        self.last_update = (packed >> 72).low_u64();
        self.reserve = packed >> 136;
    }

    pub fn preview(&self, now: u64) -> Result<(U256, U256)> {
        let mut sta = self.static_acc;
        let mut node = self.node_acc;
        if !self.trading_opened || now <= self.last_update || (self.supply.is_zero() && self.nodes.is_zero()) {
            return Ok((sta, node));
        }
        let emission = self.emission(now)?;
        let static_part = mul(emission, 50.into())? / 100;
        let node_part = mul(emission, 10.into())? / 100;
        if !self.supply.is_zero() && !static_part.is_zero() {
            sta = add(sta, mul(static_part, unit())? / self.supply)?;
        }
        if !self.nodes.is_zero() && !node_part.is_zero() {
            node = add(node, mul(node_part, unit())? / self.nodes)?;
        }
        Ok((sta, node))
    }

    fn emission(&self, now: u64) -> Result<U256> {
        ensure!(self.last_update >= self.open_time, "emission starts before opening");
        let start_day = (self.last_update - self.open_time) / 86400;
        let end_day = (now - self.open_time) / 86400;
        let end_decay = decay::pow998(end_day)?;
        let daily = |decay| -> Result<U256> { Ok(mul(mul(self.reserve, 160.into())?, decay)? / 10000 / unit()) };
        let total = if start_day == end_day {
            mul(daily(end_decay)?, (now - self.last_update).into())? / 86400
        } else {
            let start_decay = decay::pow998(start_day)?;
            // Boundaries cannot exceed `now` on this branch, including at u64::MAX.
            let next_boundary = self.open_time + (start_day + 1) * 86400;
            let end_boundary = self.open_time + end_day * 86400;
            let head = mul(daily(start_decay)?, (next_boundary - self.last_update).into())? / 86400;
            let tail = mul(daily(end_decay)?, (now - end_boundary).into())? / 86400;
            // Preserve Solidity's division order and adjacent-decay rounding.
            let next_decay = mul(start_decay, 998.into())? / 1000;
            let middle = if end_day > start_day + 1 && next_decay > end_decay {
                mul(mul(mul(self.reserve, 160.into())?, next_decay - end_decay)?, 500.into())? / 10000 / unit()
            } else {
                U256::zero()
            };
            add(add(head, middle)?, tail)?
        };
        let next_decay = mul(end_decay, 998.into())? / 1000;
        let cumulative = mul(mul(41958.into(), unit())?, unit() - next_decay)? / U256::from(2_000_000_000_000_000u64);
        let remaining = cumulative.saturating_sub(self.accounted);
        Ok(total.min(remaining))
    }
}

#[derive(Clone, Debug, Default)]
pub struct Holder {
    pub raw: U256,
    pub shares: U256,
    pub user_index: U256,
    pub user_node_index: U256,
    pub is_node: bool,
    /// The reward getter immediately returns zero for address(0) and DEAD.
    pub zero_or_dead: bool,
    /// Resolved from LBP's historical runtime, never inferred from balance data.
    pub exempt: bool,
}

impl Holder {
    pub fn pending(&self, globals: &Globals, now: u64) -> Result<U256> {
        // Preserve Solidity's early exits even when unused global arithmetic
        // would overflow or the global checkpoint is internally inconsistent.
        if self.zero_or_dead || (self.shares.is_zero() && !self.is_node) {
            return Ok(U256::zero());
        }
        let (sta, node) = globals.preview(now)?;
        let mut pending = U256::zero();
        if !self.shares.is_zero() && sta > self.user_index {
            pending = mul(self.shares, sta - self.user_index)? / unit();
        }
        if self.is_node && node > self.user_node_index {
            pending = add(pending, (node - self.user_node_index) / unit())?;
        }
        Ok(pending)
    }

    pub fn balance(&self, globals: &Globals, now: u64) -> Result<U256> {
        if self.exempt {
            return Ok(self.raw);
        }
        add(self.raw, self.pending(globals, now)?)
    }
}

#[cfg(test)]
mod tests;
