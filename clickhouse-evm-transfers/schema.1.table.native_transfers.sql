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
    coinbase                    String,
    value                       UInt256,

    -- INDEXES --
    INDEX idx_value (value)     TYPE minmax,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_coinbase_count ( SELECT coinbase, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY coinbase ),

    -- minute --
    PROJECTION prj_coinbase_by_minute ( SELECT coinbase, minute, count() GROUP BY coinbase, minute ),
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
);