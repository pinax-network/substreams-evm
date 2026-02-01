-- EXTENDED transaction with more gas fields
CREATE TABLE IF NOT EXISTS transactions AS TEMPLATE_TRANSACTION
COMMENT 'Transactions with native value & gas/fee fields';

-- Native Transfers --
CREATE TABLE IF NOT EXISTS calls AS TEMPLATE_CALL
COMMENT 'Calls with native value transfers';

-- Block Rewards --
CREATE TABLE IF NOT EXISTS block_rewards (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- block reward --
    index                       UInt32,
    miner                       String,
    value                       UInt256,
    reason                      LowCardinality(String)
)
ENGINE = MergeTree
ORDER BY block_num;

-- Validator Withdrawals (post-Shanghai) --
CREATE TABLE IF NOT EXISTS withdrawals (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- withdrawal --
    index                       UInt32,
    address                     String,
    value                       UInt256
)
ENGINE = MergeTree
ORDER BY block_num;

-- Selfdestructs --
CREATE TABLE IF NOT EXISTS selfdestructs (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- selfdestruct --
    index                       UInt32,
    tx_hash                     String,
    from_address                String,
    to_address                  String,
    value                       UInt256
)
ENGINE = MergeTree
ORDER BY block_num;

-- Genesis Balances (block 0) --
CREATE TABLE IF NOT EXISTS genesis_balances (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- genesis balance --
    index                       UInt32,
    address                     String,
    value                       UInt256
)
ENGINE = MergeTree
ORDER BY block_num;

-- DAO Hard Fork Transfers --
CREATE TABLE IF NOT EXISTS dao_transfers (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- dao transfer --
    index                       UInt32,
    address                     String,
    old_value                   UInt256,
    new_value                   UInt256,
    reason                      LowCardinality(String)
)
ENGINE = MergeTree
ORDER BY block_num;