//! BSC changed balances from persisted Extended state. No RPC imports.
mod discovery;
pub mod pb;
#[allow(dead_code)]
mod persist;

use pb::{Balance, BlockBalances, UnresolvedSlot};
use std::collections::{BTreeMap, BTreeSet};
use substreams::{errors::Error, scalar::BigInt};
use substreams_database_change::{pb::database::DatabaseChanges, tables::Tables};
use substreams_ethereum::pb::eth::v2 as eth;
use tiny_keccak::{Hasher, Keccak};

pub const WBNB: &str = "bb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c";
pub const WBNB_CODE_HASH: &str = "b7d84205eaaf83ce7b3940c6beaad6d22790255e34a9a2b486aa8cdfff118fe6";

fn require(ok: bool, message: &str) -> Result<(), Error> {
    if ok {
        Ok(())
    } else {
        Err(Error::msg(message.to_string()))
    }
}
fn hash(bytes: &[u8]) -> [u8; 32] {
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
fn slot(n: u8) -> [u8; 32] {
    let mut w = [0; 32];
    w[31] = n;
    w
}
fn mapping(owner: &[u8], position: u8) -> [u8; 32] {
    let mut preimage = [0; 64];
    preimage[12..32].copy_from_slice(owner);
    preimage[63] = position;
    hash(&preimage)
}
fn hex_bytes(s: &str) -> Result<Vec<u8>, Error> {
    hex::decode(s.strip_prefix("0x").unwrap_or(s)).map_err(|_| Error::msg("invalid preimage hex"))
}

#[derive(Default)]
struct Changes {
    balances: Vec<eth::BalanceChange>,
    storage: Vec<eth::StorageChange>,
    codes: Vec<eth::CodeChange>,
}
impl persist::Sink for Changes {
    fn balance(&mut self, c: &eth::BalanceChange, _: persist::Ctx) {
        self.balances.push(c.clone());
    }
    fn storage(&mut self, c: &eth::StorageChange, _: persist::Ctx) {
        if hex::encode(&c.address) == WBNB {
            self.storage.push(c.clone());
        }
    }
    fn code(&mut self, c: &eth::CodeChange, _: persist::Ctx) {
        if hex::encode(&c.address) == WBNB {
            self.codes.push(c.clone());
        }
    }
    fn nonce(&mut self, _: &eth::NonceChange, _: persist::Ctx) {}
}

fn insert(rows: &mut BTreeMap<(Vec<u8>, Vec<u8>), Balance>, contract: &[u8], address: &[u8], old: &[u8], new: &[u8], ordinal: u64) -> Result<(), Error> {
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
            Balance {
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

fn validate_block(block: &eth::Block) -> Result<u64, Error> {
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
    let timestamp = header.timestamp.as_ref().ok_or_else(|| Error::msg("missing timestamp"))?;
    require(timestamp.seconds >= 0, "negative timestamp")?;
    for tx in &block.transaction_traces {
        require((1..=3).contains(&tx.status) && !tx.calls.is_empty(), "incomplete transaction persistence data")?;
    }
    Ok(timestamp.seconds as u64)
}

/// Valid only for BSC's pinned WBNB implementation. Code identity is verified by
/// the comparison/bootstrap runner before a range; in-range code changes fail.
pub fn project(block: &eth::Block) -> Result<BlockBalances, Error> {
    let timestamp = validate_block(block)?;
    let mut changes = Changes::default();
    persist::collect_block(block, &mut changes)?;
    // Starting runtime must already be qualified. Even a change *to* the
    // expected runtime can follow writes made under a different layout earlier
    // in the block. Requalify that boundary instead of interpreting those writes.
    require(changes.codes.is_empty(), "WBNB code changed; storage adapter must be requalified")?;
    let mut preimages = BTreeMap::new();
    let mut candidates = BTreeSet::new();
    let wbnb = hex::decode(WBNB).unwrap();
    for tx in &block.transaction_traces {
        if tx.from.len() == 20 {
            candidates.insert(tx.from.clone());
        }
        if tx.to.len() == 20 {
            candidates.insert(tx.to.clone());
        }
    }
    for call in block.system_calls.iter().chain(block.transaction_traces.iter().flat_map(|tx| &tx.calls)) {
        if call.address.len() == 20 {
            candidates.insert(call.address.clone());
        }
        if call.caller.len() == 20 {
            candidates.insert(call.caller.clone());
        }
        if call.address != wbnb && !call.storage_changes.iter().any(|c| c.address == wbnb) {
            continue;
        }
        // Preimages are discovery hints, even in reverted calls; only persisted
        // writes below can become output balances. Verify every hint used.
        for (key, value) in &call.keccak_preimages {
            let key = hex_bytes(key)?;
            let value = hex_bytes(value)?;
            require(key.len() == 32 && hash(&value).as_slice() == key, "invalid Keccak preimage")?;
            preimages.insert(word(&key)?, value);
        }
        for log in &call.logs {
            if log.address == wbnb {
                for topic in log.topics.iter().skip(1) {
                    if topic.len() == 32 && topic[..12] == [0; 12] {
                        candidates.insert(topic[12..].to_vec());
                    }
                }
            }
        }
    }
    let candidates: BTreeMap<_, _> = candidates.into_iter().map(|a| (mapping(&a, 3), a)).collect();
    let mut out = BlockBalances {
        number: block.number,
        hash: block.hash.clone(),
        parent_hash: block.header.as_ref().unwrap().parent_hash.clone(),
        timestamp,
        preimage_count: preimages.len() as u64,
        wbnb_storage_changes: changes.storage.len() as u64,
        ..Default::default()
    };
    let mut rows = BTreeMap::new();
    changes.balances.sort_by_key(|c| c.ordinal);
    for c in changes.balances {
        insert(
            &mut rows,
            &[],
            &c.address,
            c.old_value.as_ref().map(|x| x.bytes.as_slice()).unwrap_or(&[]),
            c.new_value.as_ref().map(|x| x.bytes.as_slice()).unwrap_or(&[]),
            c.ordinal,
        )?;
    }
    changes.storage.sort_by_key(|c| c.ordinal);
    for c in changes.storage {
        let key = word(&c.key)?;
        let preimage = preimages.get(&key).filter(|p| p.len() == 64 && p[..12] == [0; 12]);
        let owner = preimage
            .filter(|p| p[32..] == slot(3))
            .map(|p| p[12..32].to_vec())
            .or_else(|| candidates.get(&key).cloned());
        if let Some(owner) = owner {
            insert(&mut rows, &wbnb, &owner, &c.old_value, &c.new_value, c.ordinal)?;
        } else {
            // WBNB's only other mutable mapping is allowance[owner][spender]
            // at slot 4. Require both verified preimages, not a guessed label.
            let allowance = preimage
                .and_then(|p| preimages.get(&word(&p[32..]).ok()?))
                .is_some_and(|p| p.len() == 64 && p[..12] == [0; 12] && p[32..] == slot(4));
            if !allowance {
                out.unresolved_wbnb_slots.push(UnresolvedSlot {
                    key: key.to_vec(),
                    new_value: c.new_value,
                    ordinal: c.ordinal,
                });
            }
        }
    }
    out.balances = rows.into_values().collect();
    Ok(out)
}

#[substreams::handlers::map]
pub fn map_balances(block: eth::Block) -> Result<BlockBalances, Error> {
    project(&block)
}

#[substreams::handlers::map]
pub fn map_erc20_candidates(block: eth::Block) -> Result<pb::StorageCandidates, Error> {
    discovery::project(&block)
}

pub fn database_changes(block: &BlockBalances) -> Result<DatabaseChanges, Error> {
    require(
        block.unresolved_wbnb_slots.is_empty(),
        "unresolved WBNB storage: refusing incomplete database output",
    )?;
    let mut tables = Tables::new();
    for balance in &block.balances {
        let address = format!("0x{}", hex::encode(&balance.address));
        let row = if balance.contract.is_empty() {
            tables.create_row("native_balances", &address)
        } else {
            let contract = format!("0x{}", hex::encode(&balance.contract));
            tables
                .create_row("erc20_balances", [("contract", contract.clone()), ("address", address.clone())])
                .set("contract", contract)
        };
        row.set("address", address)
            .set("balance", &balance.amount)
            .set("block_num", block.number.to_string())
            .set("block_hash", format!("0x{}", hex::encode(&block.hash)))
            .set("timestamp", block.timestamp.to_string());
    }
    // Always emit a marker, including blocks without changes.
    tables
        .create_row("blocks", [("block_num", block.number.to_string())])
        .set("block_num", block.number.to_string())
        .set("block_hash", format!("0x{}", hex::encode(&block.hash)))
        .set("timestamp", block.timestamp.to_string());
    Ok(tables.to_database_changes())
}

#[substreams::handlers::map]
pub fn db_out(block: BlockBalances) -> Result<DatabaseChanges, Error> {
    database_changes(&block)
}

#[cfg(test)]
mod tests;
