mod utils;

use proto::pb::native::transfers::v1 as pb;
use substreams::{
    errors::Error,
    scalar::{BigDecimal, BigInt},
};
use substreams_ethereum::pb::eth::v2::{balance_change::Reason, Block, CallType};

use crate::utils::{get_balances, get_block_reward_amount, get_gas_price, is_failed_call};

#[substreams::handlers::map]
pub fn map_events(block: Block) -> Result<pb::Events, Error> {
    let mut events = pb::Events::default();

    // EXTENDED
    // balance changes at block level
    for balance_change in &block.balance_changes {
        // Block Rewards
        if let Some(value) = get_block_reward_amount(balance_change) {
            events.block_rewards.push(pb::BlockReward {
                miner: balance_change.address.to_vec(),
                value: value.to_string(),
                reason: balance_change.reason,
            });
        }

        // Validator Withdrawals (post-Shanghai)
        if balance_change.reason() == Reason::Withdrawal {
            let (old_balance, new_balance) = get_balances(balance_change);
            let value = new_balance - old_balance;
            if value.gt(&BigInt::zero()) {
                events.withdrawals.push(pb::Withdrawal {
                    address: balance_change.address.to_vec(),
                    value: value.to_string(),
                });
            }
        }

        // Genesis balances (block 0)
        // Only include non-zero allocations as genesis balance entries
        if balance_change.reason() == Reason::GenesisBalance {
            let (_, new_balance) = get_balances(balance_change);
            if new_balance.gt(&BigInt::zero()) {
                events.genesis_balances.push(pb::GenesisBalance {
                    address: balance_change.address.to_vec(),
                    value: new_balance.to_string(),
                });
            }
        }

        // DAO hard fork transfers
        // Captures both balance adjustments (DaoAdjustBalance) and refund contract deposits (DaoRefundContract)
        // No zero-value filter needed as we capture all balance changes including decreases to zero
        if balance_change.reason() == Reason::DaoRefundContract || balance_change.reason() == Reason::DaoAdjustBalance {
            let (old_balance, new_balance) = get_balances(balance_change);
            events.dao_transfers.push(pb::DaoTransfer {
                address: balance_change.address.to_vec(),
                old_value: old_balance.to_string(),
                new_value: new_balance.to_string(),
                reason: balance_change.reason,
            });
        }
    }

    // iterate over successful transactions
    for trx in block.transactions() {
        let value = trx.clone().value.unwrap_or_default().with_decimal(0);
        let to: Option<Vec<u8>> = if trx.to.is_empty() { None } else { Some(trx.to.to_vec()) };
        let gas_price = get_gas_price(&trx.gas_price);

        let mut transaction = pb::Transaction {
            from: trx.from.to_vec(),
            to,
            hash: trx.hash.to_vec(),
            nonce: trx.nonce as u64,
            gas_price: gas_price.to_string(),
            gas_limit: trx.gas_limit as u64,
            gas_used: trx.gas_used,
            value: value.to_string(),
            calls: vec![],
        };

        // EXTENDED
        // find all value transfers from successful calls
        for call_view in trx.calls() {
            let call = call_view.as_ref();
            if is_failed_call(call) {
                continue;
            }

            // Handle SELFDESTRUCT transfers
            // When a contract self-destructs, its balance is sent to a beneficiary
            if call.suicide {
                // Find the balance change for this selfdestruct
                for balance_change in &call.balance_changes {
                    if balance_change.reason() == Reason::SuicideRefund {
                        let (old_balance, new_balance) = get_balances(balance_change);
                        let value = new_balance - old_balance;
                        if value.gt(&BigInt::zero()) {
                            events.selfdestructs.push(pb::Selfdestruct {
                                from: call.address.to_vec(),
                                to: balance_change.address.to_vec(),
                                value: value.to_string(),
                                tx_hash: trx.hash.to_vec(),
                            });
                        }
                    }
                }
            }

            // ignore calls with no value
            let value = match call.value {
                Some(ref v) => BigInt::from_unsigned_bytes_be(v.bytes.as_ref()),
                None => BigInt::zero(),
            };
            if value.le(&BigInt::zero()) {
                continue;
            }
            // TO-DO: Validate this assumption
            // Test: contract calls
            // https://etherscan.io/tx/0xe28a0ad59830ada1e96b1274e9f1aa9d5aa8bcf34bfe25271968962a7dbad803#internal
            // Test: single ETH transfer
            // https://etherscan.io/tx/0xdc2cd99c61de744a502fed484d73468c2f60cb2ad8dfc9e891886e9c619302ef

            // ignore top-level calls
            if call.depth == 0 {
                continue;
            }

            // Test: tornado cash (block 9194719)
            // https://etherscan.io/tx/0x3b4f42376dbb1224d59e541636cc3704cccb9572067d8f9758312d432adb86a6

            // A DELEGATECALL executes another contract's code but uses the calling contract's storage and balance.
            // There's no separate transfer of ETH to the contract being called.
            // The original contract's msg.sender, msg.value, and balance remain in play,
            // so you do not see an actual value transfer in the blockchain ledger for a DELEGATECALL.

            // Only CALL and CREATE type calls transfer value
            let call_type = call.call_type();
            if call_type != CallType::Call && call_type != CallType::Create {
                continue;
            }

            let pb_call_type = match call_type {
                CallType::Call => pb::CallType::Call,
                CallType::Create => pb::CallType::Create,
                _ => pb::CallType::Unspecified,
            };

            transaction.calls.push(pb::Call {
                index: call.index,
                begin_ordinal: call.begin_ordinal,
                end_ordinal: call.end_ordinal,
                caller: call.caller.to_vec(),
                address: call.address.to_vec(),
                value: value.to_string(),
                gas_consumed: call.gas_consumed,
                gas_limit: call.gas_limit,
                depth: call.depth,
                parent_index: call.parent_index,
                call_type: pb_call_type.into(),
            });
        }
        // skip transactions with no value and no calls with value
        if value.eq(&BigDecimal::zero()) && transaction.calls.is_empty() {
            continue;
        }
        events.transactions.push(transaction);
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total BlockReward events: {}", events.block_rewards.len());
    substreams::log::info!("Total Withdrawal events: {}", events.withdrawals.len());
    substreams::log::info!("Total Selfdestruct events: {}", events.selfdestructs.len());
    substreams::log::info!("Total GenesisBalance events: {}", events.genesis_balances.len());
    substreams::log::info!("Total DaoTransfer events: {}", events.dao_transfers.len());

    Ok(events)
}
