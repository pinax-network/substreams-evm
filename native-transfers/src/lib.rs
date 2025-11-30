mod utils;

use proto::pb::evm::native::transfers::v1 as pb;
use substreams::{
    errors::Error,
    scalar::{BigDecimal, BigInt},
};
use substreams_ethereum::pb::eth::v2::{Block, CallType};

use crate::utils::{get_block_reward_amount, get_gas_price, is_failed_call};

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
            });
        }
        // skip transactions with no value and no calls with value
        if value.eq(&BigDecimal::zero()) && transaction.calls.is_empty() {
            continue;
        }
        events.transactions.push(transaction);
    }
    Ok(events)
}
