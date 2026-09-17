//! Validate caller-qualified OpenZeppelin Trace208 writes without RPC or state.
use crate::{eth, hash, layout::CheckpointClock, require, word, VerifiedLayout};
use std::collections::{BTreeMap, BTreeSet};
use substreams::errors::Error;

type Key = (Vec<u8>, [u8; 32]);

fn small(value: &[u8]) -> Result<u64, Error> {
    let value = word(value)?;
    require(value[..24] == [0; 24], "voting checkpoint integer exceeds u64")?;
    Ok(u64::from_be_bytes(value[24..].try_into().unwrap()))
}
fn clock_word(value: &[u8]) -> Result<u64, Error> {
    small(&word(value)?[26..])
}
fn index(key: &[u8; 32], base: &[u8; 32]) -> Option<u64> {
    // Solidity array indexing wraps modulo 2^256. Only a clock-bounded index
    // qualifies; an arbitrary slot is never accepted just because it is larger.
    let mut difference = [0; 32];
    let mut borrow = 0_i16;
    for i in (0..32).rev() {
        let n = i16::from(key[i]) - i16::from(base[i]) - borrow;
        difference[i] = n as u8;
        borrow = i16::from(n < 0);
    }
    (difference[..24] == [0; 24]).then(|| u64::from_be_bytes(difference[24..].try_into().unwrap()))
}
fn continuous(rows: &[&eth::StorageChange]) -> Result<(), Error> {
    for row in rows {
        require(row.ordinal > 0, "voting checkpoint write has no ordinal")?;
        word(&row.old_value)?;
        word(&row.new_value)?;
    }
    for pair in rows.windows(2) {
        require(
            pair[0].ordinal < pair[1].ordinal && word(&pair[0].new_value)? == word(&pair[1].old_value)?,
            "discontinuous voting checkpoint writes",
        )?;
    }
    Ok(())
}

pub fn validate(
    block: &eth::Block,
    layouts: &[VerifiedLayout],
    storage: &[eth::StorageChange],
    preimages: &BTreeMap<[u8; 32], Vec<u8>>,
) -> Result<BTreeSet<Key>, Error> {
    let mut accepted = BTreeSet::new();
    for layout in layouts {
        let Some(rule) = &layout.voting_checkpoints else { continue };
        let clock = match rule.clock {
            CheckpointClock::BlockNumber => block.number,
            CheckpointClock::Timestamp => {
                let time = block
                    .header
                    .as_ref()
                    .and_then(|h| h.timestamp.as_ref())
                    .ok_or_else(|| Error::msg("checkpoint timestamp is missing"))?;
                require(time.seconds >= 0, "negative checkpoint timestamp")?;
                time.seconds as u64
            }
        };
        require(clock > 0 && clock < (1_u64 << 48), "checkpoint clock outside positive uint48")?;
        let mut roots = rule.slots.clone();
        for (key, preimage) in preimages {
            if preimage.len() == 64 && preimage[..12] == [0; 12] && rule.mapping_slots.contains(&word(&preimage[32..])?) {
                roots.insert(*key);
            }
        }
        let mut writes = BTreeMap::<[u8; 32], Vec<&eth::StorageChange>>::new();
        for row in storage.iter().filter(|r| r.address == layout.contract) {
            writes.entry(word(&row.key)?).or_default().push(row);
        }
        for root in roots {
            let headers = writes.get(&root);
            let mut final_length = None;
            let mut append_ordinal = None;
            if let Some(headers) = headers {
                continuous(headers)?;
                for row in headers {
                    let old = small(&row.old_value)?;
                    let new = small(&row.new_value)?;
                    require(old <= clock + 1 && new <= clock + 1, "checkpoint length exceeds distinct clock keys")?;
                    require(new == old || new == old + 1, "checkpoint length must stay equal or append one")?;
                    if new > old {
                        require(append_ordinal.replace(row.ordinal).is_none(), "multiple checkpoint appends for one clock")?;
                    }
                    final_length = Some(new);
                }
                accepted.insert((layout.contract.clone(), root));
            }
            let base = hash(&root);
            let mut touched = 0;
            for (key, rows) in &writes {
                let Some(i) = index(key, &base).filter(|i| *i <= clock) else { continue };
                // The reviewed insertion rule stores one packed uint48 clock /
                // uint208 vote word per distinct, monotonically increasing key.
                // Thus index <= clock is intrinsic, not a guessed ignore range.
                continuous(rows)?;
                if let Some(length) = final_length {
                    require(length > 0 && i == length - 1, "checkpoint write is not the witnessed final element")?;
                }
                if let Some(ordinal) = append_ordinal {
                    require(
                        rows[0].ordinal > ordinal && word(&rows[0].old_value)? == [0; 32],
                        "new checkpoint must follow its length append and start empty",
                    )?;
                } else {
                    // Timestamp clocks may overwrite a checkpoint from a prior
                    // block in the same second, without any length SSTORE.
                    require(rule.clock == CheckpointClock::Timestamp, "block checkpoint is missing its length append")?;
                    require(clock_word(&rows[0].old_value)? == clock, "existing checkpoint belongs to another clock")?;
                }
                for row in rows {
                    require(clock_word(&row.new_value)? == clock, "checkpoint word has the wrong clock")?;
                }
                accepted.insert((layout.contract.clone(), *key));
                touched += 1;
            }
            require(touched <= 1, "multiple checkpoint elements changed for one clock")?;
            require(append_ordinal.is_none() || touched == 1, "checkpoint append is missing its element")?;
        }
    }
    Ok(accepted)
}
