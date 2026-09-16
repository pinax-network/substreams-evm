use crate::{hex_bytes, require};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use substreams::errors::Error;

/// A caller-qualified direct balance mapping, optionally behind a pinned proxy.
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
    /// Reviewed non-balance mapping bases whose values span multiple words.
    #[serde(default)]
    pub other_mapping_words: BTreeMap<String, u8>,
    /// Reviewed replacement for a zero balance word. No inferred defaults.
    #[serde(default)]
    pub zero_balance: Option<ZeroBalance>,
    #[serde(default)]
    pub proxy: Option<ProxyLayout>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZeroBalance {
    /// Full uint256 word, including leading zeros.
    pub value: String,
    /// Omit only when the value is an immutable runtime constant.
    #[serde(default)]
    pub storage_slot: Option<String>,
}
#[derive(Clone, Debug)]
pub struct VerifiedZeroBalance {
    pub value: [u8; 32],
    pub storage_slot: Option<[u8; 32]>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProxyLayout {
    pub implementation_slot: String,
    pub implementation: String,
    pub code_hash: String,
}
#[derive(Clone, Debug)]
pub struct VerifiedProxy {
    pub implementation_slot: [u8; 32],
    pub implementation: Vec<u8>,
    pub code_hash: [u8; 32],
}
#[derive(Clone, Debug)]
pub struct VerifiedLayout {
    pub contract: Vec<u8>,
    pub balance_slot: [u8; 32],
    pub code_hash: [u8; 32],
    pub other_slots: BTreeSet<[u8; 32]>,
    pub other_mapping_slots: BTreeSet<[u8; 32]>,
    pub other_mapping_words: BTreeMap<[u8; 32], u8>,
    pub zero_balance: Option<VerifiedZeroBalance>,
    pub proxy: Option<VerifiedProxy>,
}
impl VerifiedLayout {
    /// Input is the canonical decimal uint256 decoded from the raw storage word.
    /// Apply only after raw-word continuity checks; zero and the fallback value
    /// can represent different storage states with the same public balance.
    pub fn project_amount(&self, raw: &str) -> String {
        if raw == "0" {
            if let Some(rule) = &self.zero_balance {
                return substreams::scalar::BigInt::from_unsigned_bytes_be(&rule.value).to_string();
            }
        }
        raw.to_owned()
    }
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
            let other_mapping_words = layout
                .other_mapping_words
                .iter()
                .map(|(base, width)| {
                    require((1..=32).contains(width), "non-balance mapping width must be 1..=32 words")?;
                    Ok((word(base)?, *width))
                })
                .collect::<Result<BTreeMap<_, _>, Error>>()?;
            require(
                !other_mapping_slots.contains(&balance_slot) && !other_slots.contains(&balance_slot) && !other_mapping_words.contains_key(&balance_slot),
                "balance slot cannot be ignored",
            )?;
            let proxy = layout
                .proxy
                .map(|p| -> Result<VerifiedProxy, Error> {
                    let implementation = fixed(&p.implementation, 20)?;
                    require(
                        implementation.iter().any(|b| *b != 0) && implementation != contract,
                        "invalid proxy implementation",
                    )?;
                    let implementation_slot = word(&p.implementation_slot)?;
                    require(
                        implementation_slot != balance_slot
                            && !other_slots.contains(&implementation_slot)
                            && !other_mapping_slots.contains(&implementation_slot),
                        "proxy implementation slot cannot be ignored or used for balances",
                    )?;
                    require(
                        !other_mapping_words.contains_key(&implementation_slot),
                        "proxy implementation slot cannot be ignored",
                    )?;
                    Ok(VerifiedProxy {
                        implementation_slot,
                        implementation,
                        code_hash: word(&p.code_hash)?,
                    })
                })
                .transpose()?;
            let zero_balance = layout
                .zero_balance
                .map(|rule| -> Result<VerifiedZeroBalance, Error> {
                    let storage_slot = rule.storage_slot.as_deref().map(word).transpose()?;
                    if let Some(slot) = storage_slot {
                        require(
                            slot != balance_slot
                                && !other_slots.contains(&slot)
                                && !other_mapping_slots.contains(&slot)
                                && !other_mapping_words.contains_key(&slot)
                                && proxy.as_ref().is_none_or(|p| p.implementation_slot != slot),
                            "zero-balance dependency must be distinct and cannot be ignored",
                        )?;
                    }
                    Ok(VerifiedZeroBalance {
                        value: word(&rule.value)?,
                        storage_slot,
                    })
                })
                .transpose()?;
            Ok(VerifiedLayout {
                contract,
                balance_slot,
                code_hash: word(&layout.code_hash)?,
                other_slots,
                other_mapping_slots,
                other_mapping_words,
                zero_balance,
                proxy,
            })
        })
        .collect()
}
