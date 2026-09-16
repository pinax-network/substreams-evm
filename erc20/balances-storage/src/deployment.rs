//! A narrowly scoped first-CREATE exception to the runtime-change guard.
use crate::{eth, hash, layout::VerifiedLayout, persist::Scope, require, Changes};
use substreams::errors::Error;

pub(super) fn validate(block: &eth::Block, layouts: &[VerifiedLayout], raw: &Changes) -> Result<(), Error> {
    for layout in layouts {
        let Some(deployment) = &layout.deployment else { continue };
        if block.number > deployment.block {
            continue;
        }
        let codes = raw.codes.iter().filter(|r| r.change.address == layout.contract).collect::<Vec<_>>();
        let storage = raw.storage.iter().filter(|c| c.address == layout.contract).collect::<Vec<_>>();
        if block.number < deployment.block {
            require(codes.is_empty() && storage.is_empty(), "configured token changed before its deployment")?;
            continue;
        }
        require(block.hash == deployment.block_hash, "deployment block hash differs")?;
        require(codes.len() == 1, "deployment requires exactly one persisted code creation")?;
        let record = codes[0];
        let c = &record.change;
        require(record.scope == Scope::Tx, "deployment must originate in a successful transaction CREATE")?;
        let transactions = block.transaction_traces.iter().filter(|tx| tx.index == record.tx_index).collect::<Vec<_>>();
        require(transactions.len() == 1, "ambiguous deployment transaction")?;
        let calls = transactions[0].calls.iter().filter(|call| call.index == record.call_index).collect::<Vec<_>>();
        require(calls.len() == 1, "ambiguous deployment call")?;
        let call = calls[0];
        require(
            call.call_type == eth::CallType::Create as i32
                && call.address == layout.contract
                && !call.state_reverted
                && call.begin_ordinal > 0
                && call.end_ordinal > call.begin_ordinal,
            "deployment code must belong to a persisted CREATE call",
        )?;
        require(
            c.ordinal > call.begin_ordinal && c.ordinal < call.end_ordinal,
            "deployment code ordinal is outside CREATE",
        )?;
        require(
            c.old_code.is_empty()
                && c.old_hash == hash(&[])
                && !c.new_code.is_empty()
                && c.new_hash == layout.code_hash
                && hash(&c.new_code) == layout.code_hash,
            "deployment runtime does not match qualified first creation",
        )?;
        // Constructor writes precede the code-change ordinal. Later calls in the
        // same block are valid; writes before CREATE are not.
        require(storage.iter().all(|s| s.ordinal > call.begin_ordinal), "token storage changed before CREATE")?;
    }
    Ok(())
}
