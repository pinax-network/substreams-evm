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
    miner                       String,
    value                       UInt256,
    reason                      LowCardinality(String)
)
ENGINE = MergeTree
ORDER BY block_num;