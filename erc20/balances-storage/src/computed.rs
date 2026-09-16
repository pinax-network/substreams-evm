//! Observed holders for reviewed address-derived getters, without storage writes.
use crate::{eth, layout::VerifiedLayout, require};
use std::collections::BTreeSet;
use substreams::errors::Error;
use substreams_abis::{standard::erc20::events, tokens::erc20::usdc::fiattoken_v2_2::events::OwnershipTransferred};
use substreams_ethereum::Event;

type TokenHolder = (Vec<u8>, Vec<u8>);

pub(super) fn holders(block: &eth::Block, layouts: &[VerifiedLayout]) -> Result<BTreeSet<TokenHolder>, Error> {
    let configured = layouts
        .iter()
        .filter(|l| l.address_hash_balance.is_some())
        .map(|l| l.contract.as_slice())
        .collect::<BTreeSet<_>>();
    let mut holders = BTreeSet::new();
    if configured.is_empty() {
        return Ok(holders);
    }
    // Same successful-transaction and non-reverted-call iterators and ABI
    // decoders as the RPC reference's ERC20 transfers/tokens modules.
    for tx in block.transactions() {
        for (log, _) in tx.logs_with_calls() {
            if !configured.contains(log.address.as_slice()) {
                continue;
            }
            let participants = if let Some(e) = events::Transfer::match_and_decode(log) {
                vec![e.from, e.to]
            } else if let Some(e) = events::Approval::match_and_decode(log) {
                vec![e.owner, e.spender]
            } else if let Some(e) = OwnershipTransferred::match_and_decode(log) {
                vec![e.previous_owner, e.new_owner]
            } else {
                return Err(Error::msg("unreviewed or malformed event for address-hash balance layout"));
            };
            for address in participants.iter().chain([&tx.from, &log.address]) {
                require(address.len() == 20, "invalid computed holder address")?;
                if address.iter().any(|b| *b != 0) {
                    holders.insert((log.address.clone(), address.clone()));
                }
            }
        }
    }
    Ok(holders)
}
