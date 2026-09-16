use crate::{hex_bytes, require};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use substreams::errors::Error;

/// A caller-qualified, non-proxy direct balance mapping. No default token list.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub contract: String,
    pub balance_slot: String,
    pub code_hash: String,
    #[serde(default)]
    pub other_slots: Vec<String>,
    #[serde(default)]
    pub other_mapping_slots: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct VerifiedLayout {
    pub contract: Vec<u8>,
    pub balance_slot: [u8; 32],
    pub code_hash: [u8; 32],
    pub other_slots: BTreeSet<[u8; 32]>,
    pub other_mapping_slots: BTreeSet<[u8; 32]>,
}
fn fixed(value: &str, size: usize) -> Result<Vec<u8>, Error> {
    require(value.starts_with("0x"), "layout values must be 0x-prefixed hex")?;
    let bytes = hex_bytes(value)?;
    require(bytes.len() == size, "layout value has wrong byte length")?;
    Ok(bytes)
}
fn word(value: &str) -> Result<[u8; 32], Error> {
    Ok(fixed(value, 32)?.try_into().unwrap())
}
pub fn parse(params: &str) -> Result<Vec<VerifiedLayout>, Error> {
    let layouts: Vec<Layout> = serde_json::from_str(params).map_err(|e| Error::msg(format!("invalid token layouts: {e}")))?;
    let mut contracts = BTreeSet::new();
    layouts
        .into_iter()
        .map(|layout| {
            let contract = fixed(&layout.contract, 20)?;
            require(contract.iter().any(|b| *b != 0), "zero token contract")?;
            require(contracts.insert(contract.clone()), "duplicate token layout")?;
            let balance_slot = word(&layout.balance_slot)?;
            let other_slots = layout.other_slots.iter().map(|s| word(s)).collect::<Result<BTreeSet<_>, _>>()?;
            let other_mapping_slots = layout.other_mapping_slots.iter().map(|s| word(s)).collect::<Result<BTreeSet<_>, _>>()?;
            require(
                !other_mapping_slots.contains(&balance_slot) && !other_slots.contains(&balance_slot),
                "balance slot cannot be ignored",
            )?;
            Ok(VerifiedLayout {
                contract,
                balance_slot,
                code_hash: word(&layout.code_hash)?,
                other_slots,
                other_mapping_slots,
            })
        })
        .collect()
}
