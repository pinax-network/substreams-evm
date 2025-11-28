use substreams::scalar::BigInt;
use substreams_ethereum::pb::eth::v2::{balance_change::Reason, BalanceChange, Call, TransactionTrace, TransactionTraceStatus};

pub fn is_failed_transaction(trx: &TransactionTrace) -> bool {
    let status: TransactionTraceStatus = trx.status();
    if status == TransactionTraceStatus::Reverted || status == TransactionTraceStatus::Failed {
        return true;
    }
    false
}

pub fn is_failed_call(call: &Call) -> bool {
    if call.state_reverted || call.status_failed || call.status_reverted {
        return true;
    }
    false
}

pub fn get_balances(balance_change: &BalanceChange) -> (BigInt, BigInt) {
    let old_balance = balance_change
        .old_value
        .as_ref()
        .map(|v| BigInt::from_unsigned_bytes_be(v.bytes.as_ref()))
        .unwrap_or_else(BigInt::zero);

    let new_balance = balance_change
        .new_value
        .as_ref()
        .map(|v| BigInt::from_unsigned_bytes_be(v.bytes.as_ref()))
        .unwrap_or_else(BigInt::zero);

    (old_balance, new_balance)
}

pub fn get_block_reward_amount(balance_change: &BalanceChange) -> Option<BigInt> {
    if balance_change.reason() != Reason::RewardMineBlock {
        return None;
    }

    let (old_balance, new_balance) = get_balances(balance_change);
    let value = new_balance - old_balance;
    if value.le(&BigInt::zero()) {
        return None;
    }
    Some(value)
}

pub fn get_gas_price(gas_price: &Option<substreams_ethereum::pb::eth::v2::BigInt>) -> BigInt {
    match gas_price.as_ref() {
        // valid price, 20 bytes or fewer (assumption that 20 bytes is the maximum size of a gas price)
        // https://github.com/pinax-network/substreams-evm-tokens/issues/34
        Some(data) if data.bytes.len() <= 20 => BigInt::from_unsigned_bytes_be(&data.bytes),

        // `None` **or** more than 20 bytes → treat as zero
        _ => BigInt::zero(),
    }
}
