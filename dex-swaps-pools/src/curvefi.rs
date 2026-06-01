use crate::PoolEntry;
use substreams_abis::dex::curvefi;
use substreams_ethereum::pb::eth::v2::{CallType, Log, TransactionTrace};
use substreams_ethereum::Event;

// Direct (non-factory) CurveFi StableSwap deployments append the ABI-encoded constructor
// args as a fixed-size tail (8 static 32-byte slots) after the init bytecode.
const STABLESWAP_CONSTRUCTOR_INPUT_LEN: usize = 32 * 8;

/// CurveFi factory events identify the pool by the freshly-CREATE'd contract address (passed
/// in via `create_address`), which lives in the call traces rather than the event itself.
pub fn collect_curvefi_factory(log: &Log, create_address: Option<&[u8]>) -> Option<PoolEntry> {
    if let Some(event) = curvefi::factory::events::PlainPoolDeployed::match_and_decode(log) {
        return Some(PoolEntry { address: create_address?.to_vec(), tokens: event.coins, factory: log.address.clone() });
    }
    if let Some(event) = curvefi::factory::events::MetaPoolDeployed::match_and_decode(log) {
        return Some(PoolEntry { address: create_address?.to_vec(), tokens: vec![event.coin], factory: log.address.clone() });
    }
    if let Some(event) = curvefi::cryptoswapfactory::events::CryptoPoolDeployed::match_and_decode(log) {
        return Some(PoolEntry { address: create_address?.to_vec(), tokens: event.coins.to_vec(), factory: log.address.clone() });
    }
    None
}

/// CurveFi direct (non-factory) StableSwap deployment: the pool's `coins` come from the
/// constructor calldata tail rather than from a log. Factory has no role here, so `factory`
/// is left empty (matching the foundational store).
pub fn collect_curvefi_init(trx: &TransactionTrace) -> Option<PoolEntry> {
    if !is_contract_creation_transaction(trx) {
        return None;
    }
    let create_call = trx.calls.iter().find(|c| c.call_type == CallType::Create as i32 && c.depth == 0)?;
    let suffix = trx.input.len().checked_sub(STABLESWAP_CONSTRUCTOR_INPUT_LEN).and_then(|start| trx.input.get(start..))?;
    let constructor = curvefi::stableswap::constructor::Constructor::decode(suffix).ok()?;
    Some(PoolEntry { address: create_call.address.clone(), tokens: constructor.coins.into_iter().collect(), factory: vec![] })
}

/// Whether the transaction is a direct contract deployment (root CREATE / empty `to`).
fn is_contract_creation_transaction(trx: &TransactionTrace) -> bool {
    trx.calls.iter().any(|call| call.call_type == CallType::Create as i32 && call.depth == 0)
        || trx.to.is_empty()
        || trx.to.iter().all(|byte| *byte == 0)
}
