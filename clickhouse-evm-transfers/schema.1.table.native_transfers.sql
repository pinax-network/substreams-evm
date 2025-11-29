-- EXTENDED transaction with more gas fields
CREATE TABLE IF NOT EXISTS transactions AS TEMPLATE_TRANSACTION
COMMENT 'Extended Transactions with additional gas/fee fields';
ALTER TABLE transactions
    -- extended transaction gas/fee --
    ADD COLUMN IF NOT EXISTS tx_base_fee_per_gas        UInt64,
    ADD COLUMN IF NOT EXISTS tx_transaction_fee         UInt256,
    ADD COLUMN IF NOT EXISTS tx_burn_fee                UInt256,
    ADD COLUMN IF NOT EXISTS tx_fee_paid                UInt256;

-- Native TRX Transfer Transactions --
CREATE TABLE IF NOT EXISTS calls AS TEMPLATE_TRANSACTION
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num,
    tx_index, call_index,
    block_hash
)
COMMENT 'Calls with native value transfers';
ALTER TABLE calls
    -- call --
    ADD COLUMN IF NOT EXISTS call_index         UInt32,
    ADD COLUMN IF NOT EXISTS caller             String,
    ADD COLUMN IF NOT EXISTS address            String,
    ADD COLUMN IF NOT EXISTS value              UInt256,
    ADD COLUMN IF NOT EXISTS gas_consumed       UInt64,
    ADD COLUMN IF NOT EXISTS gas_limit          UInt64,
    ADD COLUMN IF NOT EXISTS depth              UInt32;
