//! Validate caller-qualified address-list appends and removals from persisted writes.
use crate::{eth, hash, require, word, VerifiedLayout};
use std::collections::{BTreeMap, BTreeSet};
use substreams::errors::Error;

type Key = (Vec<u8>, [u8; 32]);

fn plus(mut word: [u8; 32], n: u64) -> [u8; 32] {
    let mut carry = u128::from(n);
    for byte in word.iter_mut().rev() {
        carry += u128::from(*byte);
        *byte = carry as u8;
        carry >>= 8;
    }
    word
}

// EVM array positions use uint256 arithmetic, including wraparound.
fn index(key: &[u8; 32], base: &[u8; 32]) -> Option<u64> {
    let mut distance = [0; 32];
    let mut borrow = 0i16;
    for i in (0..32).rev() {
        let value = i16::from(key[i]) - i16::from(base[i]) - borrow;
        distance[i] = value as u8;
        borrow = i16::from(value < 0);
    }
    (distance[..24] == [0; 24]).then(|| u64::from_be_bytes(distance[24..].try_into().unwrap()))
}

pub fn validate(layouts: &[VerifiedLayout], storage: &[eth::StorageChange], noops: &[eth::StorageChange]) -> Result<BTreeSet<Key>, Error> {
    let mut accepted = BTreeSet::new();
    for layout in layouts.iter().filter(|l| !l.address_lists.is_empty()) {
        let mut writes = BTreeMap::<[u8; 32], Vec<&eth::StorageChange>>::new();
        for row in storage.iter().chain(noops).filter(|r| r.address == layout.contract) {
            writes.entry(word(&row.key)?).or_default().push(row);
        }
        for rows in writes.values_mut() {
            rows.sort_by_key(|r| r.ordinal);
        }
        for root in &layout.address_lists {
            let Some(headers) = writes.get(root) else { continue };
            let base = hash(root);
            let mut witnessed = BTreeSet::new();
            let mut ordinals = headers.iter().map(|r| r.ordinal).collect::<BTreeSet<_>>();
            for (i, header) in headers.iter().enumerate() {
                require(header.ordinal > 0, "address-list length write has no ordinal")?;
                let old = word(&header.old_value)?;
                let new = word(&header.new_value)?;
                let previous = i.checked_sub(1).map_or(0, |n| headers[n].ordinal);
                if i > 0 {
                    require(
                        headers[i - 1].ordinal < header.ordinal && word(&headers[i - 1].new_value)? == old,
                        "discontinuous address-list length writes",
                    )?;
                }
                if new == plus(old, 1) {
                    // The last valid append grows u64::MAX to 2^64, not zero.
                    require(old[..24] == [0; 24], "address-list pre-append length exceeds u64")?;
                    let key = plus(base, u64::from_be_bytes(old[24..].try_into().unwrap()));
                    let row = writes
                        .get(&key)
                        .and_then(|rows| rows.iter().find(|r| r.ordinal > header.ordinal))
                        .ok_or_else(|| Error::msg("address-list append is missing its element"))?;
                    require(
                        headers.get(i + 1).is_none_or(|next| row.ordinal < next.ordinal),
                        "address-list element must follow its append before the next length write",
                    )?;
                    require(word(&row.old_value)? == [0; 32], "new address-list element must start empty")?;
                    require(word(&row.new_value)?[..12] == [0; 12], "address-list element exceeds an address")?;
                    require(ordinals.insert(row.ordinal), "address-list element witness reused")?;
                    witnessed.insert((key, row.ordinal));
                } else {
                    require(new[..24] == [0; 24] && old == plus(new, 1), "address-list length must append or remove one")?;
                    let tail = u64::from_be_bytes(new[24..].try_into().unwrap());
                    let key = plus(base, tail);
                    let row = writes
                        .get(&key)
                        .and_then(|rows| rows.iter().rev().find(|r| r.ordinal < header.ordinal && r.ordinal > previous))
                        .ok_or_else(|| Error::msg("address-list removal is missing its cleared tail"))?;
                    let address = word(&row.old_value)?;
                    require(
                        address[..12] == [0; 12] && word(&row.new_value)? == [0; 32],
                        "address-list removal must clear a canonical address",
                    )?;
                    require(ordinals.insert(row.ordinal), "address-list element witness reused")?;
                    witnessed.insert((key, row.ordinal));
                    // swap-and-pop may first copy the former tail to one earlier
                    // element. A tail pop has no such copy. Other writes remain
                    // unresolved; a length witness is not a blanket array range.
                    let mut moved = 0;
                    for (destination, rows) in &writes {
                        if index(destination, &base).is_none_or(|n| n >= tail) {
                            continue;
                        }
                        for copy in rows.iter().filter(|r| r.ordinal > previous && r.ordinal < row.ordinal) {
                            require(
                                word(&copy.old_value)?[..12] == [0; 12] && word(&copy.new_value)? == address,
                                "address-list move must copy the removed tail",
                            )?;
                            require(ordinals.insert(copy.ordinal), "address-list element witness reused")?;
                            witnessed.insert((*destination, copy.ordinal));
                            moved += 1;
                        }
                    }
                    require(moved <= 1, "address-list removal moves more than one element")?;
                }
            }
            for key in witnessed.iter().map(|(key, _)| key).collect::<BTreeSet<_>>() {
                let rows = &writes[key];
                for (i, row) in rows.iter().enumerate() {
                    require(witnessed.contains(&(*key, row.ordinal)), "unwitnessed address-list element write")?;
                    if i > 0 {
                        require(
                            rows[i - 1].ordinal < row.ordinal && word(&rows[i - 1].new_value)? == word(&row.old_value)?,
                            "discontinuous address-list element writes",
                        )?;
                    }
                }
                accepted.insert((layout.contract.clone(), *key));
            }
            accepted.insert((layout.contract.clone(), *root));
        }
    }
    Ok(accepted)
}
