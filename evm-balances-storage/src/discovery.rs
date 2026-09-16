//! Experimental inventory across every contract emitting ERC-20-shaped Transfer
//! logs in a block. Mapping candidates are never fed into the balances sink.
use super::*;
use pb::{MappingCandidate, StorageCandidates, TokenActivity};

const TRANSFER: &str = "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

#[derive(Default)]
struct RawChanges {
    storage: Vec<eth::StorageChange>,
    code: BTreeSet<Vec<u8>>,
}
impl persist::Sink for RawChanges {
    fn storage(&mut self, c: &eth::StorageChange, _: persist::Ctx) {
        self.storage.push(c.clone());
    }
    fn code(&mut self, c: &eth::CodeChange, _: persist::Ctx) {
        self.code.insert(c.address.clone());
    }
    fn balance(&mut self, _: &eth::BalanceChange, _: persist::Ctx) {}
    fn nonce(&mut self, _: &eth::NonceChange, _: persist::Ctx) {}
}

pub fn project(block: &eth::Block) -> Result<StorageCandidates, Error> {
    validate_block(block)?;
    let mut holders = BTreeMap::<Vec<u8>, BTreeSet<Vec<u8>>>::new();
    let persisted_calls = block.system_calls.iter().filter(|c| !c.state_reverted).chain(
        block
            .transaction_traces
            .iter()
            .filter(|tx| tx.status == 1)
            .flat_map(|tx| tx.calls.iter().filter(|c| !c.state_reverted)),
    );
    for call in persisted_calls {
        for log in &call.logs {
            if log.address.len() == 20
                && log.topics.len() == 3
                && log.data.len() == 32
                && hex::encode(&log.topics[0]) == TRANSFER
                && log.topics[1..].iter().all(|t| t.len() == 32 && t[..12] == [0; 12])
            {
                let seen = holders.entry(log.address.clone()).or_default();
                for topic in &log.topics[1..] {
                    seen.insert(topic[12..].to_vec());
                }
            }
        }
    }
    let mut tokens: BTreeMap<_, _> = holders
        .into_iter()
        .map(|(contract, holders)| {
            (
                contract.clone(),
                TokenActivity {
                    contract,
                    transfer_holders: holders.into_iter().collect(),
                    ..Default::default()
                },
            )
        })
        .collect();
    let mut raw = RawChanges::default();
    persist::collect_block(block, &mut raw)?;
    for (contract, token) in &mut tokens {
        token.code_changed = raw.code.contains(contract);
    }
    let mut preimages = BTreeMap::new();
    for call in block.system_calls.iter().chain(block.transaction_traces.iter().flat_map(|tx| &tx.calls)) {
        if !tokens.contains_key(&call.address) && !call.storage_changes.iter().any(|s| tokens.contains_key(&s.address)) {
            continue;
        }
        for (key, value) in &call.keccak_preimages {
            let key = hex_bytes(key)?;
            let value = hex_bytes(value)?;
            require(key.len() == 32 && hash(&value).as_slice() == key, "invalid discovery preimage")?;
            preimages.insert(key, value);
        }
    }
    let mut candidates = BTreeMap::<(Vec<u8>, Vec<u8>), MappingCandidate>::new();
    raw.storage.sort_by_key(|s| s.ordinal);
    for s in raw.storage {
        let Some(token) = tokens.get_mut(&s.address) else {
            continue;
        };
        token.storage_changes += 1;
        let key = word(&s.key)?.to_vec();
        let Some(p) = preimages.get(&key).filter(|p| p.len() == 64 && p[..12] == [0; 12]) else {
            token.unclassified_storage_changes += 1;
            continue;
        };
        require(s.ordinal > 0, "discovery write has no execution ordinal")?;
        let before = amount(&s.old_value)?;
        let after = amount(&s.new_value)?;
        let row = candidates.entry((s.address.clone(), key.clone())).or_insert_with(|| MappingCandidate {
            contract: s.address,
            address: p[12..32].to_vec(),
            mapping_slot: p[32..].to_vec(),
            storage_key: key,
            old_amount: before.clone(),
            amount: before.clone(),
            ordinal: 0,
        });
        require(s.ordinal > row.ordinal && row.amount == before, "discontinuous discovery writes")?;
        row.amount = after;
        row.ordinal = s.ordinal;
    }
    Ok(StorageCandidates {
        number: block.number,
        hash: block.hash.clone(),
        parent_hash: block.header.as_ref().unwrap().parent_hash.clone(),
        candidates: candidates.into_values().collect(),
        tokens: tokens.into_values().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    #[test]
    fn captured_block_discovers_multiple_contracts_without_claiming_balances() {
        let b = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122260950.pb").as_slice()).unwrap();
        let out = project(&b).unwrap();
        assert!(out.tokens.len() > 5);
        assert!(out.candidates.iter().any(|c| hex::encode(&c.contract) != WBNB));
        assert!(out.candidates.iter().any(|c| c.mapping_slot != slot(3)));
        assert!(out.candidates.iter().all(|c| c.storage_key.len() == 32 && c.mapping_slot.len() == 32));
    }
    #[test]
    fn failed_logs_do_not_create_supported_tokens() {
        let mut b = eth::Block::decode(include_bytes!("../tests/fixtures/bsc-122260950.pb").as_slice()).unwrap();
        for tx in &mut b.transaction_traces {
            for call in &mut tx.calls {
                call.state_reverted = true;
            }
        }
        b.system_calls.clear();
        assert!(project(&b).unwrap().tokens.is_empty());
    }
}
