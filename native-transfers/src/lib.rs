mod utils;

use proto::pb::evm::native::transfers::v1 as pb;
use substreams::{errors::Error, scalar::BigInt};
use substreams_ethereum::pb::eth::v2::{Block, CallType};

use crate::utils::{get_block_reward_amount, get_gas_price, is_failed_call, is_failed_transaction};

#[substreams::handlers::map]
pub fn map_events(block: Block) -> Result<pb::Events, Error> {
    let mut events = pb::Events::default();

    // EXTENDED
    // balance changes at block level
    for balance_change in &block.balance_changes {
        // Block Rewards as transfer
        if let Some(amount) = get_block_reward_amount(balance_change) {
            events.block_rewards.push(pb::BlockReward {
                miner: balance_change.address.to_vec(),
                amount: amount.to_string(),
            });
        }
    }

    // to compute the burned portion of transaction fee
    let header = block.header.clone().expect("header is required");
    let base_fee_per_gas = match header.base_fee_per_gas {
        Some(base_fee_per_gas) => BigInt::from_unsigned_bytes_be(&base_fee_per_gas.bytes),
        None => BigInt::zero(),
    };

    // iterate over successful transactions
    for trx in block.transaction_traces.iter() {
        let value = trx.clone().value.unwrap_or_default().with_decimal(0);
        let to: Option<Vec<u8>> = if trx.to.is_empty() { None } else { Some(trx.to.to_vec()) };
        let gas_price = get_gas_price(&trx.gas_price);
        let gas_used = BigInt::from(trx.gas_used);
        let transaction_fee = gas_price.clone() * &gas_used;
        let burn_fee = base_fee_per_gas.clone() * &gas_used;
        let fee_paid = transaction_fee.clone() - burn_fee.clone();

        let mut transaction = pb::Transaction {
            // -- transaction --
            from: trx.from.to_vec(),
            to,
            hash: trx.hash.to_vec(),
            nonce: trx.nonce as u64,
            gas_price: gas_price.to_string(),
            gas_limit: trx.gas_limit as u64,
            gas_used: trx.receipt().receipt.cumulative_gas_used,
            value: value.to_string(),
            base_fee_per_gas: base_fee_per_gas.to_string(),
            transaction_fee: transaction_fee.to_string(),
            burn_fee: burn_fee.to_string(),
            fee_paid: fee_paid.to_string(),
            calls: vec![],
            status: trx.status,
        };

        // EXTENDED
        // find all value transfers from successful calls
        if !is_failed_transaction(trx) {
            for call_view in trx.calls() {
                let call = call_view.as_ref();
                if is_failed_call(call) {
                    continue;
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

                // A DELEGATECALL executes another contract's code but uses the calling contract’s storage and balance.
                // There’s no separate transfer of ETH to the contract being called.
                // The original contract’s msg.sender, msg.value, and balance remain in play,
                // so you do not see an actual value transfer in the blockchain ledger for a DELEGATECALL.

                // only `call` type calls are considered transfers
                if call.call_type() != CallType::Call {
                    continue;
                }
                transaction.calls.push(pb::Call {
                    caller: call.caller.to_vec(),
                    address: call.address.to_vec(),
                    value: value.to_string(),
                    gas_consumed: call.gas_consumed,
                    gas_limit: call.gas_limit,
                    depth: call.depth,
                });
            }
        }
        if !transaction.calls.is_empty() {
            events.transactions.push(transaction);
        }
    }
    Ok(events)
}
