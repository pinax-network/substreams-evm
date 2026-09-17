//! Validate caller-qualified append-only address lists from persisted writes.
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
            for (i, header) in headers.iter().enumerate() {
                require(header.ordinal > 0, "address-list append has no ordinal")?;
                let old = word(&header.old_value)?;
                let new = word(&header.new_value)?;
                require(old[..24] == [0; 24], "address-list pre-append length exceeds u64")?;
                // The last valid append grows u64::MAX to 2^64, not to zero.
                require(new == plus(old, 1), "address-list length must append one")?;
                if i > 0 {
                    require(
                        headers[i - 1].ordinal < header.ordinal && word(&headers[i - 1].new_value)? == old,
                        "discontinuous address-list length writes",
                    )?;
                }
                let index = u64::from_be_bytes(old[24..].try_into().unwrap());
                let key = plus(hash(root), index);
                let rows = writes.get(&key).ok_or_else(|| Error::msg("address-list append is missing its element"))?;
                require(rows.len() == 1, "address-list element must be written exactly once")?;
                let row = rows[0];
                require(
                    row.ordinal > header.ordinal && headers.get(i + 1).is_none_or(|next| row.ordinal < next.ordinal),
                    "address-list element must follow its append before the next append",
                )?;
                require(word(&row.old_value)? == [0; 32], "new address-list element must start empty")?;
                require(word(&row.new_value)?[..12] == [0; 12], "address-list element exceeds an address")?;
                accepted.insert((layout.contract.clone(), key));
            }
            accepted.insert((layout.contract.clone(), *root));
        }
    }
    Ok(accepted)
}
