-- Transfers including ERC-20, WETH & Native value from calls & transactions --
CREATE TABLE IF NOT EXISTS native_transfers (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    Nullable(UInt32), -- derived from Substreams
    tx_hash                     String,

    -- call --
    call_index                  Nullable(UInt32),

    -- transfer --
    `from`                      String,
    `to`                        String,
    amount                      UInt256,

    -- type --
    transfer_type               Enum8('transaction' = 1, 'call' = 2, 'block_reward' = 3),

    -- INDEXES --
    INDEX idx_amount (amount) TYPE minmax,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_transfer_type_count ( SELECT transfer_type, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY transfer_type ),
    PROJECTION prj_from_count ( SELECT `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from` ),
    PROJECTION prj_to_count ( SELECT `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `to` ),
    PROJECTION prj_from_to_count ( SELECT `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from`, `to` ),

    -- minute + timestamp --
    PROJECTION prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp GROUP BY tx_hash, minute, timestamp ),

    -- minute --
    PROJECTION prj_transfer_type_by_minute ( SELECT transfer_type, minute GROUP BY transfer_type, minute ),
    PROJECTION prj_from_by_minute ( SELECT `from`, minute GROUP BY `from`, minute ),
    PROJECTION prj_to_by_minute ( SELECT `to`, minute GROUP BY `to`, minute ),
    PROJECTION prj_from_to_by_minute ( SELECT `from`, `to`, minute GROUP BY `from`, `to`, minute ),
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
)
COMMENT 'Transfers Native value from calls & transactions';

-- MV's for Transfers --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_transactions TO native_transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- call --
    cast(NULL AS Nullable(UInt32)) AS call_index,

    -- transfer --
    tx_from as `from`,
    tx_to as `to`,
    tx_value as amount,
    'transaction' AS transfer_type
FROM transactions
WHERE amount > 0 AND `from` != `to`;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_calls TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- call --
    call_index,

    -- transfer --
    call_caller as `from`,
    call_address as `to`,
    call_value as amount,
    'call' AS transfer_type
FROM calls
WHERE amount > 0 AND `from` != `to`;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_block_rewards TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --,
    CAST(NULL AS Nullable(UInt32)) AS tx_index,
    '' AS tx_hash,

    -- call --
    cast(NULL AS Nullable(UInt32)) AS call_index,

    -- transfer (native ETH leg) --
    '' AS `from`,
    miner AS `to`,
    value AS amount,
    'block_reward' AS transfer_type
FROM block_rewards
WHERE amount > 0;