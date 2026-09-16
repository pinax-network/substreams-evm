//! A single RPC-free map with the shared ERC-20 Events output.
mod computed;
mod deployment;
#[cfg(not(target_arch = "wasm32"))]
pub mod discovery;
pub mod layout;
#[allow(dead_code)]
pub mod persist;

use layout::VerifiedLayout;
use proto::pb::evm::balances::v1 as balances_pb;
use std::collections::{BTreeMap, BTreeSet};
use substreams::{errors::Error, scalar::BigInt};
use substreams_ethereum::pb::eth::v2 as eth;
use tiny_keccak::{Hasher, Keccak};

fn require(ok: bool, message: &str) -> Result<(), Error> {
    if ok {
        Ok(())
    } else {
        Err(Error::msg(message.to_string()))
    }
}
pub fn hash(bytes: &[u8]) -> [u8; 32] {
    let mut result = [0; 32];
    let mut hasher = Keccak::v256();
    hasher.update(bytes);
    hasher.finalize(&mut result);
    result
}
fn word(bytes: &[u8]) -> Result<[u8; 32], Error> {
    require(bytes.len() <= 32, "word exceeds uint256")?;
    let mut out = [0; 32];
    out[32 - bytes.len()..].copy_from_slice(bytes);
    Ok(out)
}
fn amount(bytes: &[u8]) -> Result<String, Error> {
    Ok(BigInt::from_unsigned_bytes_be(&word(bytes)?).to_string())
}
fn mapping(owner: &[u8], position: &[u8; 32]) -> [u8; 32] {
    let mut preimage = [0; 64];
    preimage[12..32].copy_from_slice(owner);
    preimage[32..].copy_from_slice(position);
    hash(&preimage)
}
fn hex_bytes(s: &str) -> Result<Vec<u8>, Error> {
    hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(|_| Error::msg("invalid hex"))
}

#[derive(Default)]
struct Changes {
    storage: Vec<eth::StorageChange>,
    codes: Vec<CodeRecord>,
}
struct CodeRecord {
    change: eth::CodeChange,
    scope: persist::Scope,
    tx_index: u32,
    call_index: u32,
}
impl persist::Sink for Changes {
    fn balance(&mut self, _: &eth::BalanceChange, _: persist::Ctx) {}
    fn storage(&mut self, c: &eth::StorageChange, _: persist::Ctx) {
        self.storage.push(c.clone());
    }
    fn code(&mut self, c: &eth::CodeChange, ctx: persist::Ctx) {
        self.codes.push(CodeRecord {
            change: c.clone(),
            scope: ctx.scope,
            tx_index: ctx.tx_index,
            call_index: ctx.call_index,
        });
    }
    fn nonce(&mut self, _: &eth::NonceChange, _: persist::Ctx) {}
}

// Ordinary Rust intermediates, not protobufs or additional Substreams outputs.
#[derive(Debug)]
pub struct Change {
    pub contract: Vec<u8>,
    pub address: Vec<u8>,
    pub old_amount: String,
    pub amount: String,
    pub ordinal: u64,
}
fn insert(rows: &mut BTreeMap<(Vec<u8>, Vec<u8>), Change>, contract: &[u8], address: &[u8], old: &[u8], new: &[u8], ordinal: u64) -> Result<(), Error> {
    require(address.len() == 20, "invalid balance address")?;
    require(ordinal > 0, "persisted balance has no execution ordinal")?;
    let old = amount(old)?;
    let new = amount(new)?;
    let key = (contract.to_vec(), address.to_vec());
    if let Some(row) = rows.get_mut(&key) {
        require(ordinal > row.ordinal, "ambiguous balance execution order")?;
        require(old == row.amount, "discontinuous balance changes within block")?;
        row.amount = new;
        row.ordinal = ordinal;
    } else {
        rows.insert(
            key,
            Change {
                contract: contract.to_vec(),
                address: address.to_vec(),
                old_amount: old,
                amount: new,
                ordinal,
            },
        );
    }
    Ok(())
}
fn validate_block(block: &eth::Block) -> Result<(), Error> {
    require(
        block.detail_level == eth::block::DetailLevel::DetaillevelExtended as i32,
        "Extended blocks required",
    )?;
    require((3..=5).contains(&block.ver), "unsupported Extended producer version")?;
    let header = block.header.as_ref().ok_or_else(|| Error::msg("missing header"))?;
    require(
        block.hash.len() == 32 && header.parent_hash.len() == 32 && header.state_root.len() == 32,
        "invalid block identity",
    )?;
    require(header.number == block.number, "header number mismatch")?;
    for tx in &block.transaction_traces {
        require((1..=3).contains(&tx.status) && !tx.calls.is_empty(), "incomplete transaction persistence data")?;
    }
    Ok(())
}
fn mapping_has_base(mut key: [u8; 32], preimages: &BTreeMap<[u8; 32], Vec<u8>>, expected: &[u8; 32]) -> bool {
    // Follow verified nested mapping preimages; never guess an unknown slot's role.
    let mut visited = BTreeSet::new();
    while visited.insert(key) {
        let Some(preimage) = preimages.get(&key).filter(|p| p.len() == 64) else {
            return false;
        };
        let base: [u8; 32] = preimage[32..].try_into().unwrap();
        if &base == expected {
            return true;
        }
        key = base;
    }
    false
}
fn subtract_offset(mut key: [u8; 32], offset: u8) -> [u8; 32] {
    // Solidity addresses struct fields as (mapping hash + offset) modulo 2^256.
    let mut borrow = offset as u16;
    for byte in key.iter_mut().rev() {
        let (value, underflow) = byte.overflowing_sub(borrow as u8);
        *byte = value;
        borrow = (borrow >> 8) + u16::from(underflow);
    }
    key
}
fn ignored_mapping(key: [u8; 32], preimages: &BTreeMap<[u8; 32], Vec<u8>>, layout: &VerifiedLayout) -> bool {
    layout.other_mapping_slots.iter().any(|base| mapping_has_base(key, preimages, base))
        || layout
            .other_mapping_words
            .iter()
            .any(|(base, width)| (0..*width).any(|offset| mapping_has_base(subtract_offset(key, offset), preimages, base)))
}

/// Layout semantics and the starting runtime must be qualified by the caller.
/// Code changes are rejected except explicitly qualified first deployments.
/// The audit runner checks code_hash at both boundaries; the stateless mapper
/// cannot infer preexisting runtime identity from a block with no code change.
pub fn changes(block: &eth::Block, layouts: &[VerifiedLayout]) -> Result<Vec<Change>, Error> {
    validate_block(block)?;
    let configured: BTreeMap<_, _> = layouts.iter().map(|l| (l.contract.clone(), l)).collect();
    let mut raw = Changes::default();
    persist::collect_block(block, &mut raw)?;
    deployment::validate(block, layouts, &raw)?;
    require(
        !raw.codes.iter().any(|record| {
            let c = &record.change;
            configured
                .get(&c.address)
                .is_some_and(|l| l.deployment.as_ref().is_none_or(|d| d.block != block.number))
                || layouts.iter().any(|l| {
                    l.proxy.as_ref().is_some_and(|p| p.implementation == c.address)
                        || l.minimal_proxy.as_ref().is_some_and(|p| p.implementation == c.address)
                        || l.beacon_proxy.as_ref().is_some_and(|p| p.beacon == c.address || p.implementation == c.address)
                })
        }),
        "configured token, beacon or implementation code changed; requalify layout",
    )?;
    let mut preimages = BTreeMap::new();
    let mut candidates = BTreeSet::new();
    for tx in &block.transaction_traces {
        for address in [&tx.from, &tx.to] {
            if address.len() == 20 {
                candidates.insert(address.clone());
            }
        }
    }
    for call in block.system_calls.iter().chain(block.transaction_traces.iter().flat_map(|tx| &tx.calls)) {
        for address in [&call.address, &call.caller] {
            if address.len() == 20 {
                candidates.insert(address.clone());
            }
        }
        if !configured.contains_key(&call.address) && !call.storage_changes.iter().any(|c| configured.contains_key(&c.address)) {
            continue;
        }
        // Reverted preimages are discovery hints only; only persisted writes emit balances.
        for (key, value) in &call.keccak_preimages {
            let key = hex_bytes(key)?;
            let value = hex_bytes(value)?;
            require(key.len() == 32 && hash(&value).as_slice() == key, "invalid Keccak preimage")?;
            preimages.insert(word(&key)?, value);
        }
        for log in &call.logs {
            if configured.contains_key(&log.address) {
                for topic in log.topics.iter().skip(1) {
                    if topic.len() == 32 && topic[..12] == [0; 12] {
                        candidates.insert(topic[12..].to_vec());
                    }
                }
            }
        }
    }
    let candidates: BTreeMap<_, BTreeMap<_, _>> = layouts
        .iter()
        .map(|l| (l.balance_slot, candidates.iter().map(|a| (mapping(a, &l.balance_slot), a.clone())).collect()))
        .collect();
    let mut rows = BTreeMap::new();
    let mut beacon_slots = BTreeMap::<Vec<u8>, BTreeSet<[u8; 32]>>::new();
    for beacon in layouts.iter().filter_map(|l| l.beacon_proxy.as_ref()) {
        beacon_slots.entry(beacon.beacon.clone()).or_default().insert(beacon.implementation_slot);
    }
    raw.storage.sort_by_key(|c| c.ordinal);
    let mut deployment_keys = BTreeSet::new();
    for c in raw.storage {
        if !configured.contains_key(&c.address) && !beacon_slots.contains_key(&c.address) {
            continue;
        }
        let key = word(&c.key)?;
        require(
            !beacon_slots.get(&c.address).is_some_and(|slots| slots.contains(&key)),
            "beacon implementation changed; requalify layout",
        )?;
        let Some(layout) = configured.get(&c.address) else {
            continue;
        };
        if layout.deployment.as_ref().is_some_and(|d| d.block == block.number) && deployment_keys.insert((c.address.clone(), key)) {
            require(word(&c.old_value)? == [0; 32], "initial deployment storage must start at zero")?;
        }
        require(
            layout.beacon_proxy.as_ref().is_none_or(|p| p.beacon_slot != key),
            "proxy beacon changed; requalify layout",
        )?;
        require(
            layout.proxy.as_ref().is_none_or(|p| p.implementation_slot != key),
            "proxy implementation slot changed; requalify layout",
        )?;
        require(
            layout.zero_balance.as_ref().and_then(|r| r.storage_slot) != Some(key),
            "zero-balance dependency changed; requalify and rebuild dependent holder state",
        )?;
        if let Some(address) = layout.address_hash_balance.as_ref().and_then(|rule| rule.stored_addresses.get(&key)) {
            require(
                &word(&c.old_value)?[12..] == address && &word(&c.new_value)?[12..] == address,
                "computed balance address selector changed; requalify and rebuild dependent holder state",
            )?;
            // The getter masks the low 160 bits. Reviewed packed flags above
            // the address do not change which holders use stored balances.
            continue;
        }
        let owner = preimages
            .get(&key)
            .filter(|p| p.len() == 64 && p[..12] == [0; 12] && p[32..] == layout.balance_slot)
            .map(|p| p[12..32].to_vec())
            .or_else(|| candidates[&layout.balance_slot].get(&key).cloned());
        if let Some(owner) = owner {
            insert(&mut rows, &c.address, &owner, &c.old_value, &c.new_value, c.ordinal)?;
        } else if !layout.other_slots.contains(&key) && !ignored_mapping(key, &preimages, layout) {
            return Err(Error::msg(format!(
                "unresolved storage for configured token 0x{} at key 0x{}; refusing incomplete events",
                hex::encode(&c.address),
                hex::encode(key)
            )));
        }
    }
    Ok(rows.into_values().collect())
}
pub fn project(block: &eth::Block, layouts: &[VerifiedLayout]) -> Result<balances_pb::Events, Error> {
    let configured: BTreeMap<_, _> = layouts.iter().map(|l| (l.contract.as_slice(), l)).collect();
    let mut balances = changes(block, layouts)?
        .into_iter()
        // The RPC reference's common::is_valid_evm_address excludes null.
        .filter(|b| b.address.iter().any(|byte| *byte != 0))
        .map(|b| {
            (
                (b.contract.clone(), b.address.clone()),
                balances_pb::Balance {
                    amount: configured[b.contract.as_slice()].project_amount(&b.address, &b.amount),
                    contract: Some(b.contract),
                    address: b.address,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (contract, address) in computed::holders(block, layouts)? {
        if let Some(amount) = configured[contract.as_slice()]
            .address_hash_balance
            .as_ref()
            .and_then(|rule| rule.amount(&address))
        {
            balances.insert(
                (contract.clone(), address.clone()),
                balances_pb::Balance {
                    contract: Some(contract),
                    address,
                    amount,
                },
            );
        }
    }
    Ok(balances_pb::Events {
        balances: balances.into_values().collect(),
    })
}
// The SDK macro generates raw-pointer parameter decoding and discards function
// attributes. Keep its ABI-specific lint exception scoped to this wrapper.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
mod handler {
    use super::*;
    #[substreams::handlers::map]
    fn map_events(params: String, block: eth::Block) -> Result<balances_pb::Events, Error> {
        project(&block, &layout::parse(&params)?)
    }
}
#[cfg(test)]
mod computed_tests;
#[cfg(test)]
mod tests;
