use super::{Account, State, Supply};
use crate::{data::*, rpc::quantity, survey::mapping_key};
use anyhow::{ensure, Context, Result};
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Layout {
    pub reflections: u64,
    pub tokens: u64,
    pub excluded_flag: u64,
    pub excluded_array: u64,
    pub total_reflections: u64,
    pub total_tokens: u64,
}

pub fn word(n: impl Into<U256>) -> String {
    format!("0x{:064x}", n.into())
}

pub fn array_key(slot: u64, index: usize) -> String {
    let mut slot_bytes = [0; 32];
    U256::from(slot).to_big_endian(&mut slot_bytes);
    let base = U256::from_big_endian(&erc20_balances_storage::hash(&slot_bytes));
    word(base.overflowing_add(index.into()).0)
}

pub fn storage(v: &Value) -> Result<BTreeMap<String, U256>> {
    v.as_object()
        .context("missing initialized storage")?
        .iter()
        .map(|(key, value)| Ok((binary(&serde_json::json!(key), 32)?, quantity(value)?)))
        .collect()
}

pub fn decode(layout: &Layout, holder: &str, words: &BTreeMap<String, U256>) -> Result<State> {
    let read = |key: &str| words.get(key).copied().context("missing initialized storage word");
    let account = |address: &str| -> Result<Account> {
        Ok(Account {
            address: hex::decode(binary(&serde_json::json!(address), 20)?.trim_start_matches("0x"))?
                .try_into()
                .unwrap(),
            reflections: read(&mapping_key(address, &word(layout.reflections))?)?,
            tokens: read(&mapping_key(address, &word(layout.tokens))?)?,
        })
    };
    let is_excluded = !(read(&mapping_key(holder, &word(layout.excluded_flag))?)? & U256::from(255)).is_zero();
    let holder = account(holder)?;
    if is_excluded {
        return Ok(State {
            holder,
            is_excluded,
            supply: None,
        });
    }
    let length = read(&word(layout.excluded_array))?;
    ensure!(length <= U256::from(usize::MAX), "exclusion list length exceeds host capacity");
    let length = length.as_usize();
    // Every member needs an initialized element word; do not allocate a vector
    // from an untrusted length before verifying even that minimum coverage.
    ensure!(length <= words.len(), "incomplete initialized exclusion list");
    let excluded = (0..length)
        .map(|index| {
            let value = read(&array_key(layout.excluded_array, index))?;
            let address = format!("0x{:040x}", value & ((U256::one() << 160) - 1));
            account(&address)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(State {
        holder,
        is_excluded,
        supply: Some(Supply {
            reflections: read(&word(layout.total_reflections))?,
            tokens: read(&word(layout.total_tokens))?,
            excluded_length: length,
            excluded,
        }),
    })
}
