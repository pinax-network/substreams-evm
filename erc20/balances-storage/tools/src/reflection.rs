//! Host-only model for source-reviewed reflection getters.
//!
//! Inputs come from independently initialized storage, never from balanceOf.
//! This does not qualify a production layout or maintain state across blocks.
use anyhow::{ensure, Context, Result};
use primitive_types::U256;
use std::collections::BTreeMap;

#[doc(hidden)]
pub mod fixture;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub address: [u8; 20],
    pub reflections: U256,
    pub tokens: U256,
}

#[derive(Clone, Debug)]
pub struct Supply {
    pub reflections: U256,
    pub tokens: U256,
    pub excluded_length: usize,
    /// Complete on-chain array order, including duplicates if present.
    pub excluded: Vec<Account>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub holder: Account,
    pub is_excluded: bool,
    pub supply: Option<Supply>,
}

fn div(a: U256, b: U256) -> Result<U256> {
    a.checked_div(b).context("uint256 division by zero")
}

impl Supply {
    fn validate(&self, holder: &Account) -> Result<()> {
        ensure!(self.excluded_length == self.excluded.len(), "incomplete initialized exclusion list");
        let mut known = BTreeMap::from([(holder.address, holder)]);
        for account in &self.excluded {
            if let Some(previous) = known.insert(account.address, account) {
                ensure!(previous == account, "inconsistent initialized account storage");
            }
        }
        Ok(())
    }

    fn rate(&self) -> Result<U256> {
        ensure!(self.excluded_length == self.excluded.len(), "incomplete initialized exclusion list");
        let mut reflections = self.reflections;
        let mut tokens = self.tokens;
        for excluded in &self.excluded {
            // Deployed getters return the global supplies immediately when a
            // subtraction would underflow; later entries are not evaluated.
            if excluded.reflections > reflections || excluded.tokens > tokens {
                return div(self.reflections, self.tokens);
            }
            reflections -= excluded.reflections;
            tokens -= excluded.tokens;
        }
        if reflections < div(self.reflections, self.tokens)? {
            return div(self.reflections, self.tokens);
        }
        div(reflections, tokens)
    }
}

impl State {
    pub fn balance(&self) -> Result<U256> {
        // Excluded holders bypass even invalid global supplies.
        if self.is_excluded {
            return Ok(self.holder.tokens);
        }
        let supply = self.supply.as_ref().context("missing initialized reflection supply")?;
        ensure!(self.holder.reflections <= supply.reflections, "amount exceeds total reflections");
        supply.validate(&self.holder)?;
        div(self.holder.reflections, supply.rate()?)
    }
}

#[cfg(test)]
mod tests;
