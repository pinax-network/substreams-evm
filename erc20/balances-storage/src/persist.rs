// Adapted from pinax-network/substreams-evm-state at 29251362c8651fd6d5dbe7b3d69820c29ae172c5 (Apache-2.0).
//! Persistence rules: which state-change records in an Extended
//! `sf.ethereum.type.v2.Block` actually survived execution.
//!
//! Rules (from the `type.proto` documentation, verified on BSC Firehose):
//!
//! * `SUCCEEDED` tx: every change of every call with `state_reverted == false`.
//! * `FAILED` / `REVERTED` tx: only the root call is consulted. Balance
//!   changes with reason `GAS_BUY`, `GAS_REFUND`, `REWARD_TRANSACTION_FEE`
//!   persist; the sender's transaction nonce increment persists.
//! * EIP-7702 (`TRX_TYPE_SET_CODE`): nonce + code changes of every
//!   non-discarded `authority` before root-call execution persist even when
//!   the tx fails. Reverted execution can change the same authority again.
//! * `Block.system_calls`: calls with `state_reverted == false`.
//! * `Block.balance_changes`, `Block.code_changes`: always.
//! * No-op records (`old == new`) are dropped from changes. Storage no-ops have
//!   a separate optional validation hook for invariants that forbid any write.

use std::collections::HashSet;
use substreams::errors::Error;

use substreams_ethereum::pb::eth::v2::{
    balance_change::Reason, transaction_trace::Type as TxType, BalanceChange, Block, Call, CodeChange, NonceChange, StorageChange, TransactionTrace,
    TransactionTraceStatus,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Tx,
    TxFailedPersistent,
    Tx7702,
    SystemCall,
    Block,
}

/// Provenance attached to each emitted record.
#[derive(Clone, Copy, Debug)]
pub struct Ctx<'a> {
    pub scope: Scope,
    pub tx_hash: &'a [u8],
    pub tx_index: u32,
    pub tx_status: i32,
    pub call_index: u32,
}

impl Ctx<'_> {
    fn block(scope: Scope) -> Self {
        Ctx {
            scope,
            tx_hash: &[],
            tx_index: 0,
            tx_status: 0,
            call_index: 0,
        }
    }
}

/// Receives persisted records. Implemented by the map module.
pub trait Sink {
    fn storage(&mut self, c: &StorageChange, ctx: Ctx);
    fn storage_noop(&mut self, _: &StorageChange, _: Ctx) {}
    fn balance(&mut self, c: &BalanceChange, ctx: Ctx);
    fn nonce(&mut self, c: &NonceChange, ctx: Ctx);
    fn code(&mut self, c: &CodeChange, ctx: Ctx);
}

pub fn is_failed(trx: &TransactionTrace) -> bool {
    !matches!(trx.status(), TransactionTraceStatus::Succeeded)
}

pub fn is_gas_reason(bc: &BalanceChange) -> bool {
    matches!(bc.reason(), Reason::GasBuy | Reason::GasRefund | Reason::RewardTransactionFee)
}

fn balance_is_noop(bc: &BalanceChange) -> bool {
    let old = bc.old_value.as_ref().map(|v| v.bytes.as_slice()).unwrap_or(&[]);
    let new = bc.new_value.as_ref().map(|v| v.bytes.as_slice()).unwrap_or(&[]);
    strip_zeros(old) == strip_zeros(new)
}

fn strip_zeros(b: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < b.len() && b[i] == 0 {
        i += 1;
    }
    &b[i..]
}

fn storage_is_noop(sc: &StorageChange) -> bool {
    strip_zeros(&sc.old_value) == strip_zeros(&sc.new_value)
}

/// Non-discarded EIP-7702 authorities of a SetCode transaction.
pub fn authorities(trx: &TransactionTrace) -> HashSet<Vec<u8>> {
    if trx.r#type() != TxType::TrxTypeSetCode {
        return HashSet::new();
    }
    trx.set_code_authorizations
        .iter()
        .filter(|a| !a.discarded)
        .filter_map(|a| a.authority.clone())
        .filter(|a| !a.is_empty())
        .collect()
}

fn emit_call_all(call: &Call, ctx: Ctx, out: &mut impl Sink) {
    for c in &call.storage_changes {
        if !storage_is_noop(c) {
            out.storage(c, ctx);
        } else {
            out.storage_noop(c, ctx);
        }
    }
    for c in &call.balance_changes {
        if !balance_is_noop(c) {
            out.balance(c, ctx);
        }
    }
    for c in &call.nonce_changes {
        if c.old_value != c.new_value {
            out.nonce(c, ctx);
        }
    }
    for c in &call.code_changes {
        if c.old_hash != c.new_hash {
            out.code(c, ctx);
        }
    }
}

pub fn collect_transaction(trx: &TransactionTrace, out: &mut impl Sink) -> Result<(), Error> {
    let status = trx.status() as i32;
    let base = |scope: Scope, call_index: u32| Ctx {
        scope,
        tx_hash: &trx.hash,
        tx_index: trx.index,
        tx_status: status,
        call_index,
    };

    if !is_failed(trx) {
        for call in &trx.calls {
            if call.state_reverted {
                continue;
            }
            emit_call_all(call, base(Scope::Tx, call.index), out);
        }
        return Ok(());
    }

    // Failed or reverted: only the root call carries persisted effects.
    let Some(root) = trx.calls.first() else { return Ok(()) };
    let ctx = base(Scope::TxFailedPersistent, root.index);

    for bc in &root.balance_changes {
        if is_gas_reason(bc) && !balance_is_noop(bc) {
            out.balance(bc, ctx);
        }
    }

    // Reverted execution records may have ordinal zero. Neither array order nor
    // the smallest ordinal across all accounts identifies the sender. Match the
    // transaction's actual sender and nonce increment instead.
    let sender_nonce = root
        .nonce_changes
        .iter()
        .filter(|n| n.ordinal > 0 && n.address == trx.from && n.old_value == trx.nonce && Some(n.new_value) == trx.nonce.checked_add(1))
        .min_by_key(|n| n.ordinal);
    if let Some(change) = sender_nonce {
        out.nonce(change, ctx);
    }

    // EIP-7702: authorization nonce/code changes persist for non-discarded authorities.
    let auths = authorities(trx);
    if !auths.is_empty() {
        if root.begin_ordinal == 0 {
            return Err(Error::msg("failed SetCode transaction has no authorization/execution ordinal boundary"));
        }
        let ctx7702 = base(Scope::Tx7702, root.index);
        // Geth and Reth emit the auth-list changes before the root call begins.
        // An authority can execute CREATE (nonce change), or other reverted code,
        // later in that same root call. Address membership is not persistence.
        let before_execution = |ordinal| ordinal > 0 && ordinal < root.begin_ordinal;
        for nc in &root.nonce_changes {
            if sender_nonce.is_some_and(|sender| std::ptr::eq(nc, sender)) {
                continue; // already emitted above
            }
            if before_execution(nc.ordinal) && auths.contains(&nc.address) && nc.old_value != nc.new_value {
                out.nonce(nc, ctx7702);
            }
        }
        for cc in &root.code_changes {
            if before_execution(cc.ordinal) && auths.contains(&cc.address) && cc.old_hash != cc.new_hash {
                out.code(cc, ctx7702);
            }
        }
    }
    Ok(())
}

pub fn collect_block(block: &Block, out: &mut impl Sink) -> Result<(), Error> {
    for call in &block.system_calls {
        if call.state_reverted {
            continue;
        }
        emit_call_all(
            call,
            Ctx {
                call_index: call.index,
                ..Ctx::block(Scope::SystemCall)
            },
            out,
        );
    }
    let ctx = Ctx::block(Scope::Block);
    for bc in &block.balance_changes {
        if !balance_is_noop(bc) {
            out.balance(bc, ctx);
        }
    }
    for cc in &block.code_changes {
        if cc.old_hash != cc.new_hash {
            out.code(cc, ctx);
        }
    }
    for trx in &block.transaction_traces {
        collect_transaction(trx, out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use substreams_ethereum::pb::eth::v2::{BigInt, SetCodeAuthorization};

    #[derive(Default)]
    struct Rec {
        storage: Vec<(Vec<u8>, u64, Scope)>,
        balance: Vec<(Vec<u8>, i32, Scope)>,
        nonce: Vec<(Vec<u8>, u64, Scope)>,
        code: Vec<(Vec<u8>, Scope)>,
    }
    impl Sink for Rec {
        fn storage(&mut self, c: &StorageChange, ctx: Ctx) {
            self.storage.push((c.address.clone(), c.ordinal, ctx.scope))
        }
        fn balance(&mut self, c: &BalanceChange, ctx: Ctx) {
            self.balance.push((c.address.clone(), c.reason, ctx.scope))
        }
        fn nonce(&mut self, c: &NonceChange, ctx: Ctx) {
            self.nonce.push((c.address.clone(), c.new_value, ctx.scope))
        }
        fn code(&mut self, c: &CodeChange, ctx: Ctx) {
            self.code.push((c.address.clone(), ctx.scope))
        }
    }

    fn big(v: u64) -> Option<BigInt> {
        Some(BigInt {
            bytes: v.to_be_bytes().to_vec(),
        })
    }
    fn bal(addr: u8, reason: Reason, ord: u64, old: u64, new: u64) -> BalanceChange {
        BalanceChange {
            address: vec![addr; 20],
            old_value: big(old),
            new_value: big(new),
            reason: reason as i32,
            ordinal: ord,
        }
    }
    fn nonce(addr: u8, ord: u64, old: u64) -> NonceChange {
        NonceChange {
            address: vec![addr; 20],
            old_value: old,
            new_value: old + 1,
            ordinal: ord,
        }
    }
    fn storage(addr: u8, ord: u64, old: u8, new: u8) -> StorageChange {
        StorageChange {
            address: vec![addr; 20],
            key: vec![1; 32],
            old_value: vec![old; 32],
            new_value: vec![new; 32],
            ordinal: ord,
        }
    }
    fn code(addr: u8, ord: u64) -> CodeChange {
        CodeChange {
            address: vec![addr; 20],
            old_hash: vec![],
            old_code: vec![],
            new_hash: vec![9; 32],
            new_code: vec![0xef, 1, 0],
            ordinal: ord,
        }
    }

    #[test]
    fn succeeded_tx_keeps_non_reverted_calls_only() {
        let trx = TransactionTrace {
            status: TransactionTraceStatus::Succeeded as i32,
            calls: vec![
                Call {
                    index: 0,
                    storage_changes: vec![storage(0xA, 10, 0, 1)],
                    ..Default::default()
                },
                Call {
                    index: 1,
                    state_reverted: true,
                    storage_changes: vec![storage(0xB, 11, 0, 1)],
                    ..Default::default()
                },
                Call {
                    index: 2,
                    storage_changes: vec![storage(0xC, 12, 5, 5)],
                    ..Default::default()
                }, // no-op
            ],
            ..Default::default()
        };
        let mut r = Rec::default();
        collect_transaction(&trx, &mut r).unwrap();
        assert_eq!(r.storage, vec![(vec![0xA; 20], 10, Scope::Tx)]);
    }

    #[test]
    fn failed_tx_keeps_gas_and_sender_nonce_only() {
        let trx = TransactionTrace {
            status: TransactionTraceStatus::Reverted as i32,
            from: vec![0xA; 20],
            nonce: 7,
            calls: vec![
                Call {
                    index: 0,
                    state_reverted: true,
                    balance_changes: vec![
                        bal(0xA, Reason::GasBuy, 1, 100, 90),
                        bal(0xA, Reason::Transfer, 2, 90, 80),
                        bal(0xB, Reason::Transfer, 3, 0, 10),
                        bal(0xA, Reason::GasRefund, 8, 80, 85),
                        bal(0xF, Reason::RewardTransactionFee, 9, 0, 5),
                    ],
                    nonce_changes: vec![nonce(0xA, 1, 7), nonce(0xB, 4, 0)],
                    storage_changes: vec![storage(0xB, 5, 0, 1)],
                    ..Default::default()
                },
                Call {
                    index: 1,
                    state_reverted: true,
                    storage_changes: vec![storage(0xC, 6, 0, 1)],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let mut r = Rec::default();
        collect_transaction(&trx, &mut r).unwrap();
        assert!(r.storage.is_empty());
        assert_eq!(r.balance.len(), 3);
        assert!(r
            .balance
            .iter()
            .all(|(_, reason, s)| *s == Scope::TxFailedPersistent && *reason != Reason::Transfer as i32));
        assert_eq!(r.nonce, vec![(vec![0xA; 20], 8, Scope::TxFailedPersistent)]);
    }

    #[test]
    fn failed_7702_tx_keeps_authority_nonce_and_code() {
        let trx = TransactionTrace {
            status: TransactionTraceStatus::Failed as i32,
            from: vec![0xA; 20],
            nonce: 7,
            r#type: TxType::TrxTypeSetCode as i32,
            set_code_authorizations: vec![
                SetCodeAuthorization {
                    authority: Some(vec![0xB; 20]),
                    discarded: false,
                    ..Default::default()
                },
                SetCodeAuthorization {
                    authority: Some(vec![0xC; 20]),
                    discarded: true,
                    ..Default::default()
                },
            ],
            calls: vec![Call {
                index: 0,
                begin_ordinal: 6,
                state_reverted: true,
                balance_changes: vec![bal(0xA, Reason::GasBuy, 1, 100, 90)],
                nonce_changes: vec![nonce(0xA, 1, 7), nonce(0xB, 2, 3), nonce(0xC, 3, 0)],
                code_changes: vec![code(0xB, 4), code(0xC, 5)],
                storage_changes: vec![storage(0xB, 6, 0, 1)],
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut r = Rec::default();
        collect_transaction(&trx, &mut r).unwrap();
        assert_eq!(r.nonce, vec![(vec![0xA; 20], 8, Scope::TxFailedPersistent), (vec![0xB; 20], 4, Scope::Tx7702)]);
        assert_eq!(r.code, vec![(vec![0xB; 20], Scope::Tx7702)]);
        assert!(r.storage.is_empty());
    }

    #[test]
    fn failed_sender_selection_ignores_zero_ordinals_and_other_accounts() {
        let trx = TransactionTrace {
            status: TransactionTraceStatus::Reverted as i32,
            from: vec![0xA; 20],
            nonce: 7,
            calls: vec![Call {
                state_reverted: true,
                nonce_changes: vec![nonce(0xB, 0, 0), nonce(0xC, 1, 0), nonce(0xA, 0, 8), nonce(0xA, 2, 7), nonce(0xA, 20, 8)],
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut r = Rec::default();
        collect_transaction(&trx, &mut r).unwrap();
        assert_eq!(r.nonce, vec![(vec![0xA; 20], 8, Scope::TxFailedPersistent)]);
    }

    #[test]
    fn failed_self_and_multiple_authorizations_exclude_reverted_execution() {
        // Auth-list effects precede root begin (10); the same delegated account
        // then attempts CREATE in reverted execution (20). Also include a
        // zero-ordinal reverted record, a discarded auth and explicit code clear.
        let mut clear = code(0xB, 8);
        clear.old_hash = vec![8; 32];
        clear.new_hash = vec![];
        clear.new_code = vec![];
        let trx = TransactionTrace {
            status: TransactionTraceStatus::Failed as i32,
            r#type: TxType::TrxTypeSetCode as i32,
            from: vec![0xA; 20],
            nonce: 7,
            set_code_authorizations: vec![
                SetCodeAuthorization {
                    authority: Some(vec![0xA; 20]),
                    nonce: 8,
                    ..Default::default()
                },
                SetCodeAuthorization {
                    authority: Some(vec![0xB; 20]),
                    nonce: 3,
                    ..Default::default()
                },
                SetCodeAuthorization {
                    authority: Some(vec![0xB; 20]),
                    nonce: 4,
                    ..Default::default()
                },
                SetCodeAuthorization {
                    authority: Some(vec![0xC; 20]),
                    discarded: true,
                    ..Default::default()
                },
            ],
            calls: vec![Call {
                begin_ordinal: 10,
                end_ordinal: 30,
                state_reverted: true,
                nonce_changes: vec![
                    nonce(0xA, 2, 7),
                    nonce(0xA, 3, 8),
                    nonce(0xB, 5, 3),
                    nonce(0xB, 7, 4),
                    nonce(0xC, 9, 0),
                    nonce(0xA, 20, 9),
                    nonce(0xB, 0, 5),
                ],
                code_changes: vec![code(0xA, 4), code(0xB, 6), clear, code(0xC, 9), code(0xA, 21), code(0xB, 0)],
                storage_changes: vec![storage(0xA, 25, 0, 1)],
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut r = Rec::default();
        collect_transaction(&trx, &mut r).unwrap();
        assert_eq!(
            r.nonce,
            vec![
                (vec![0xA; 20], 8, Scope::TxFailedPersistent),
                (vec![0xA; 20], 9, Scope::Tx7702),
                (vec![0xB; 20], 4, Scope::Tx7702),
                (vec![0xB; 20], 5, Scope::Tx7702)
            ]
        );
        assert_eq!(
            r.code,
            vec![(vec![0xA; 20], Scope::Tx7702), (vec![0xB; 20], Scope::Tx7702), (vec![0xB; 20], Scope::Tx7702)]
        );
        assert!(r.storage.is_empty());
    }

    #[test]
    fn ambiguous_failed_authorization_boundary_is_rejected() {
        let trx = TransactionTrace {
            status: TransactionTraceStatus::Failed as i32,
            r#type: TxType::TrxTypeSetCode as i32,
            set_code_authorizations: vec![SetCodeAuthorization {
                authority: Some(vec![0xA; 20]),
                ..Default::default()
            }],
            calls: vec![Call::default()],
            ..Default::default()
        };
        assert!(collect_transaction(&trx, &mut Rec::default()).is_err());
    }

    #[test]
    fn captured_bsc_failed_setcode_transactions_keep_sender_and_authority_nonces() {
        // Unmodified TransactionTrace messages fetched from BSC Extended v5.
        // Provenance, block/transaction hashes and checksums are next to the files.
        let samples: [(&[u8], u64, u64, i32); 2] = [
            (include_bytes!("../tests/fixtures/bsc-121114122-failed-setcode.pb"), 27164, 4308, 3),
            (include_bytes!("../tests/fixtures/bsc-121114153-failed-setcode.pb"), 27167, 4311, 2),
        ];
        let sender = hex::decode("d52573f6d4f68d8e7f8fe2ed50a1023c5f6fe82a").unwrap();
        let authority = hex::decode("417204ea716dfc4427bf9883521c820b036cdb7a").unwrap();
        for (bytes, sender_nonce, authority_nonce, status) in samples {
            let tx = TransactionTrace::decode(bytes).unwrap();
            assert_eq!(tx.status, status);
            assert_eq!(tx.r#type(), TxType::TrxTypeSetCode);
            assert!(tx.calls[0].state_reverted);
            let mut r = Rec::default();
            collect_transaction(&tx, &mut r).unwrap();
            assert_eq!(
                r.nonce,
                vec![
                    (sender.clone(), sender_nonce, Scope::TxFailedPersistent),
                    (authority.clone(), authority_nonce, Scope::Tx7702)
                ]
            );
            assert!(r.storage.is_empty() && r.code.is_empty());
            assert!(!r.balance.is_empty());
            assert!(r.balance.iter().all(|(_, reason, scope)| *scope == Scope::TxFailedPersistent
                && [Reason::GasBuy as i32, Reason::GasRefund as i32, Reason::RewardTransactionFee as i32].contains(reason)));
        }
    }

    #[test]
    fn block_level_and_system_calls() {
        let block = Block {
            balance_changes: vec![
                bal(0xF, Reason::RewardTransactionFee, 100, 0, 1),
                bal(0xE, Reason::RewardTransactionFee, 101, 5, 5),
            ],
            system_calls: vec![
                Call {
                    index: 0,
                    storage_changes: vec![storage(0x29, 2, 0, 1)],
                    ..Default::default()
                },
                Call {
                    index: 1,
                    state_reverted: true,
                    storage_changes: vec![storage(0x30, 3, 0, 1)],
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let mut r = Rec::default();
        collect_block(&block, &mut r).unwrap();
        assert_eq!(r.storage, vec![(vec![0x29; 20], 2, Scope::SystemCall)]);
        assert_eq!(r.balance, vec![(vec![0xF; 20], Reason::RewardTransactionFee as i32, Scope::Block)]);
    }
}
