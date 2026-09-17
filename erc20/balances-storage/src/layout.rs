use crate::{hash, hex_bytes, require};
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
    /// Explicitly qualified first deployment of a direct mapping or minimal proxy.
    #[serde(default)]
    pub deployment: Option<Deployment>,
    #[serde(default)]
    pub other_slots: Vec<String>,
    #[serde(default)]
    pub other_mapping_slots: Vec<String>,
    /// Reviewed non-balance mapping bases whose values span multiple words.
    #[serde(default)]
    pub other_mapping_words: BTreeMap<String, u8>,
    /// Reviewed OpenZeppelin Trace208 arrays, separate from ordinary balances.
    #[serde(default)]
    pub voting_checkpoints: Option<VotingCheckpoints>,
    /// Reviewed append-only address[] roots, with a witnessed length increment
    /// before every element write and a maximum pre-append length of u64::MAX.
    #[serde(default)]
    pub address_lists: Vec<String>,
    /// Reviewed replacement for a zero balance word. No inferred defaults.
    #[serde(default)]
    pub zero_balance: Option<ZeroBalance>,
    #[serde(default)]
    pub proxy: Option<ProxyLayout>,
    #[serde(default)]
    pub beacon_proxy: Option<BeaconProxyLayout>,
    #[serde(default)]
    pub minimal_proxy: Option<MinimalProxyLayout>,
    #[serde(default)]
    pub address_hash_balance: Option<AddressHashBalance>,
    /// Caller-proven empty-at-CREATE mapping that the pinned direct runtime
    /// cannot write. This is not inferred from absent writes or RPC samples.
    #[serde(default)]
    pub immutable_zero_mapping: bool,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointClock {
    BlockNumber,
    Timestamp,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VotingCheckpoints {
    pub clock: CheckpointClock,
    /// Direct Trace208 arrays, normally the total-supply vote history.
    #[serde(default)]
    pub slots: Vec<String>,
    /// mapping(address => Trace208), with its array at struct offset zero.
    #[serde(default)]
    pub mapping_slots: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct VerifiedVotingCheckpoints {
    pub clock: CheckpointClock,
    pub slots: BTreeSet<[u8; 32]>,
    pub mapping_slots: BTreeSet<[u8; 32]>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddressHashBalance {
    pub modulus: String,
    pub offset: String,
    pub multiplier: String,
    /// These scalar slots select holders that read the ordinary balance mapping.
    pub stored_addresses: Vec<StoredAddress>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredAddress {
    pub slot: String,
    pub address: String,
}
#[derive(Clone, Debug)]
pub struct VerifiedAddressHashBalance {
    pub modulus: substreams::scalar::BigInt,
    pub offset: substreams::scalar::BigInt,
    pub multiplier: substreams::scalar::BigInt,
    pub stored_addresses: BTreeMap<[u8; 32], Vec<u8>>,
}
impl VerifiedAddressHashBalance {
    pub fn amount(&self, address: &[u8]) -> Option<String> {
        if self.stored_addresses.values().any(|stored| stored == address) {
            return None;
        }
        Some(((substreams::scalar::BigInt::from_unsigned_bytes_be(&hash(address)) % &self.modulus + &self.offset) * &self.multiplier).to_string())
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MinimalProxyLayout {
    pub implementation: String,
    pub code_hash: String,
}
#[derive(Clone, Debug)]
pub struct VerifiedMinimalProxy {
    pub implementation: Vec<u8>,
    pub code_hash: [u8; 32],
}
impl VerifiedMinimalProxy {
    /// Canonical 45-byte ERC-1167 runtime; other forwarding patterns need review.
    pub fn runtime(&self) -> Vec<u8> {
        let mut bytes = hex::decode("363d3d373d3d3d363d73").unwrap();
        bytes.extend_from_slice(&self.implementation);
        bytes.extend(hex::decode("5af43d82803e903d91602b57fd5bf3").unwrap());
        bytes
    }
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Deployment {
    pub block: u64,
    pub block_hash: String,
}
#[derive(Clone, Debug)]
pub struct VerifiedDeployment {
    pub block: u64,
    pub block_hash: [u8; 32],
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BeaconProxyLayout {
    pub beacon_slot: String,
    pub beacon: String,
    pub beacon_code_hash: String,
    pub implementation_slot: String,
    pub implementation: String,
    pub implementation_code_hash: String,
}
#[derive(Clone, Debug)]
pub struct VerifiedBeaconProxy {
    pub beacon_slot: [u8; 32],
    pub beacon: Vec<u8>,
    pub beacon_code_hash: [u8; 32],
    pub implementation_slot: [u8; 32],
    pub implementation: Vec<u8>,
    pub implementation_code_hash: [u8; 32],
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZeroBalance {
    /// Full uint256 word, including leading zeros.
    pub value: String,
    /// Omit only when the value is an immutable runtime constant.
    #[serde(default)]
    pub storage_slot: Option<String>,
    /// Runtime-qualified holders that keep a zero word instead of the fallback.
    #[serde(default)]
    pub excluded_addresses: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct VerifiedZeroBalance {
    pub value: [u8; 32],
    pub storage_slot: Option<[u8; 32]>,
    pub excluded_addresses: BTreeSet<Vec<u8>>,
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
    pub deployment: Option<VerifiedDeployment>,
    pub other_slots: BTreeSet<[u8; 32]>,
    pub other_mapping_slots: BTreeSet<[u8; 32]>,
    pub other_mapping_words: BTreeMap<[u8; 32], u8>,
    pub voting_checkpoints: Option<VerifiedVotingCheckpoints>,
    pub address_lists: BTreeSet<[u8; 32]>,
    pub zero_balance: Option<VerifiedZeroBalance>,
    pub proxy: Option<VerifiedProxy>,
    pub beacon_proxy: Option<VerifiedBeaconProxy>,
    pub minimal_proxy: Option<VerifiedMinimalProxy>,
    pub address_hash_balance: Option<VerifiedAddressHashBalance>,
    pub immutable_zero_mapping: bool,
}
impl VerifiedLayout {
    /// A qualified value available without reading prior holder state.
    /// Call only at/after deployment for an immutable-zero mapping.
    pub fn known_amount(&self, address: &[u8]) -> Option<String> {
        if self.immutable_zero_mapping {
            Some("0".into())
        } else {
            self.address_hash_balance.as_ref().and_then(|rule| rule.amount(address))
        }
    }
    /// Input is the canonical decimal uint256 decoded from the raw storage word.
    /// Apply only after raw-word continuity checks; zero and the fallback value
    /// can represent different storage states with the same public balance.
    pub fn project_amount(&self, address: &[u8], raw: &str) -> String {
        if let Some(amount) = self.address_hash_balance.as_ref().and_then(|rule| rule.amount(address)) {
            return amount;
        }
        if raw == "0" {
            if let Some(rule) = &self.zero_balance {
                if !rule.excluded_addresses.contains(address) {
                    return substreams::scalar::BigInt::from_unsigned_bytes_be(&rule.value).to_string();
                }
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
            let beacon_proxy = layout
                .beacon_proxy
                .map(|p| -> Result<VerifiedBeaconProxy, Error> {
                    require(proxy.is_none(), "configure either a direct proxy or a beacon proxy")?;
                    let beacon = fixed(&p.beacon, 20)?;
                    let implementation = fixed(&p.implementation, 20)?;
                    require(
                        beacon.iter().any(|b| *b != 0)
                            && implementation.iter().any(|b| *b != 0)
                            && beacon != contract
                            && implementation != contract
                            && beacon != implementation,
                        "invalid beacon dependency addresses",
                    )?;
                    let beacon_slot = word(&p.beacon_slot)?;
                    require(
                        beacon_slot != balance_slot
                            && !other_slots.contains(&beacon_slot)
                            && !other_mapping_slots.contains(&beacon_slot)
                            && !other_mapping_words.contains_key(&beacon_slot),
                        "beacon slot cannot be ignored or used for balances",
                    )?;
                    Ok(VerifiedBeaconProxy {
                        beacon_slot,
                        beacon,
                        beacon_code_hash: word(&p.beacon_code_hash)?,
                        implementation_slot: word(&p.implementation_slot)?,
                        implementation,
                        implementation_code_hash: word(&p.implementation_code_hash)?,
                    })
                })
                .transpose()?;
            let zero_balance = layout
                .zero_balance
                .map(|rule| -> Result<VerifiedZeroBalance, Error> {
                    let mut excluded_addresses = BTreeSet::new();
                    for address in &rule.excluded_addresses {
                        require(excluded_addresses.insert(fixed(address, 20)?), "duplicate zero-balance excluded address")?;
                    }
                    let storage_slot = rule.storage_slot.as_deref().map(word).transpose()?;
                    if let Some(slot) = storage_slot {
                        require(
                            slot != balance_slot
                                && !other_slots.contains(&slot)
                                && !other_mapping_slots.contains(&slot)
                                && !other_mapping_words.contains_key(&slot)
                                && proxy.as_ref().is_none_or(|p| p.implementation_slot != slot)
                                && beacon_proxy.as_ref().is_none_or(|p| p.beacon_slot != slot),
                            "zero-balance dependency must be distinct and cannot be ignored",
                        )?;
                    }
                    Ok(VerifiedZeroBalance {
                        value: word(&rule.value)?,
                        storage_slot,
                        excluded_addresses,
                    })
                })
                .transpose()?;
            let deployment = layout
                .deployment
                .map(|d| -> Result<VerifiedDeployment, Error> {
                    require(d.block > 0, "deployment block must be positive")?;
                    require(
                        proxy.is_none() && beacon_proxy.is_none() && zero_balance.is_none(),
                        "deployment qualification requires a direct mapping or minimal proxy without a zero-balance rule",
                    )?;
                    let block_hash = word(&d.block_hash)?;
                    require(block_hash != [0; 32], "deployment block hash cannot be zero")?;
                    Ok(VerifiedDeployment { block: d.block, block_hash })
                })
                .transpose()?;
            let code_hash = word(&layout.code_hash)?;
            let minimal_proxy = layout
                .minimal_proxy
                .map(|p| -> Result<VerifiedMinimalProxy, Error> {
                    require(proxy.is_none() && beacon_proxy.is_none(), "configure only one proxy kind")?;
                    let implementation = fixed(&p.implementation, 20)?;
                    require(
                        implementation.iter().any(|b| *b != 0) && implementation != contract,
                        "invalid minimal proxy implementation",
                    )?;
                    let parsed = VerifiedMinimalProxy {
                        implementation,
                        code_hash: word(&p.code_hash)?,
                    };
                    require(
                        hash(&parsed.runtime()) == code_hash,
                        "minimal proxy runtime hash does not bind the canonical forwarding code and target",
                    )?;
                    Ok(parsed)
                })
                .transpose()?;
            let address_hash_balance = layout
                .address_hash_balance
                .map(|rule| -> Result<VerifiedAddressHashBalance, Error> {
                    use substreams::scalar::BigInt;
                    require(
                        zero_balance.is_none() && deployment.is_none(),
                        "address-hash balances cannot combine with a zero fallback or deployment baseline",
                    )?;
                    let modulus = BigInt::from_unsigned_bytes_be(&word(&rule.modulus)?);
                    let offset = BigInt::from_unsigned_bytes_be(&word(&rule.offset)?);
                    let multiplier = BigInt::from_unsigned_bytes_be(&word(&rule.multiplier)?);
                    let max = BigInt::from_unsigned_bytes_be(&[255; 32]);
                    require(modulus > 0, "address-hash modulus must be positive")?;
                    let largest = &modulus - BigInt::from(1) + &offset;
                    require(largest <= max && &largest * &multiplier <= max, "address-hash arithmetic exceeds uint256")?;
                    let mut stored_addresses = BTreeMap::new();
                    for dependency in rule.stored_addresses {
                        let slot = word(&dependency.slot)?;
                        require(
                            slot != balance_slot
                                && !other_slots.contains(&slot)
                                && !other_mapping_slots.contains(&slot)
                                && !other_mapping_words.contains_key(&slot)
                                && proxy.as_ref().is_none_or(|p| p.implementation_slot != slot)
                                && beacon_proxy.as_ref().is_none_or(|p| p.beacon_slot != slot),
                            "address selector slot cannot be ignored or reused",
                        )?;
                        require(
                            stored_addresses.insert(slot, fixed(&dependency.address, 20)?).is_none(),
                            "duplicate address selector slot",
                        )?;
                    }
                    Ok(VerifiedAddressHashBalance {
                        modulus,
                        offset,
                        multiplier,
                        stored_addresses,
                    })
                })
                .transpose()?;
            let voting_checkpoints = layout
                .voting_checkpoints
                .map(|rule| -> Result<VerifiedVotingCheckpoints, Error> {
                    let mut roots = BTreeSet::new();
                    for value in rule.slots.iter().chain(&rule.mapping_slots) {
                        let slot = word(value)?;
                        require(roots.insert(slot), "duplicate voting checkpoint root")?;
                        require(
                            slot != balance_slot
                                && !other_slots.contains(&slot)
                                && !other_mapping_slots.contains(&slot)
                                && !other_mapping_words.contains_key(&slot)
                                && proxy.as_ref().is_none_or(|p| p.implementation_slot != slot)
                                && beacon_proxy.as_ref().is_none_or(|p| p.beacon_slot != slot)
                                && zero_balance.as_ref().is_none_or(|p| p.storage_slot != Some(slot))
                                && address_hash_balance.as_ref().is_none_or(|p| !p.stored_addresses.contains_key(&slot)),
                            "voting checkpoint root overlaps another configured field",
                        )?;
                    }
                    require(!roots.is_empty(), "voting checkpoint roots are empty")?;
                    Ok(VerifiedVotingCheckpoints {
                        clock: rule.clock,
                        slots: rule.slots.iter().map(|s| word(s)).collect::<Result<_, _>>()?,
                        mapping_slots: rule.mapping_slots.iter().map(|s| word(s)).collect::<Result<_, _>>()?,
                    })
                })
                .transpose()?;
            let mut address_lists = BTreeSet::new();
            for value in &layout.address_lists {
                let slot = word(value)?;
                require(address_lists.insert(slot), "duplicate address-list root")?;
                require(
                    slot != balance_slot
                        && !other_slots.contains(&slot)
                        && !other_mapping_slots.contains(&slot)
                        && !other_mapping_words.contains_key(&slot)
                        && proxy.as_ref().is_none_or(|p| p.implementation_slot != slot)
                        && beacon_proxy.as_ref().is_none_or(|p| p.beacon_slot != slot)
                        && zero_balance.as_ref().is_none_or(|p| p.storage_slot != Some(slot))
                        && address_hash_balance.as_ref().is_none_or(|p| !p.stored_addresses.contains_key(&slot))
                        && voting_checkpoints
                            .as_ref()
                            .is_none_or(|p| !p.slots.contains(&slot) && !p.mapping_slots.contains(&slot)),
                    "address-list root overlaps another configured field",
                )?;
            }
            require(
                !layout.immutable_zero_mapping
                    || (deployment.is_some()
                        && proxy.is_none()
                        && beacon_proxy.is_none()
                        && minimal_proxy.is_none()
                        && zero_balance.is_none()
                        && address_hash_balance.is_none()),
                "immutable-zero mapping requires a qualified direct deployment without another balance rule",
            )?;
            Ok(VerifiedLayout {
                contract,
                balance_slot,
                code_hash,
                deployment,
                other_slots,
                other_mapping_slots,
                other_mapping_words,
                voting_checkpoints,
                address_lists,
                zero_balance,
                proxy,
                beacon_proxy,
                minimal_proxy,
                address_hash_balance,
                immutable_zero_mapping: layout.immutable_zero_mapping,
            })
        })
        .collect()
}
