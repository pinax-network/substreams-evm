-- Transfers including ERC-20, WETH & Native value from calls & transactions --
CREATE TABLE IF NOT EXISTS transfers (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    UInt32, -- derived from Substreams
    tx_hash                     String,

    -- call --
    call_index                  Nullable(UInt32),

    -- log --
    log_index                   Nullable(UInt32), -- derived from Substreams
    log_address                 LowCardinality(String),
    log_ordinal                 Nullable(UInt32),

    -- transfer --
    `from`                      String,
    `to`                        String,
    amount                      UInt256,

    -- type --
    transfer_type               LowCardinality(String),

    -- INDEXES --
    INDEX idx_amount (amount) TYPE minmax,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_transfer_type_count ( SELECT transfer_type, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY transfer_type ),
    PROJECTION prj_from_count ( SELECT `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from` ),
    PROJECTION prj_to_count ( SELECT `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `to` ),
    PROJECTION prj_from_to_count ( SELECT `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from`, `to` ),
    PROJECTION prj_log_address_from_count ( SELECT log_address, `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `from` ),
    PROJECTION prj_log_address_to_count ( SELECT log_address, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `to` ),
    PROJECTION prj_log_address_to_from_count ( SELECT log_address, `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute)  GROUP BY log_address, `from`, `to` ),

    -- minute: log_address | from | to --
    PROJECTION prj_transfer_type_by_minute ( SELECT transfer_type, minute GROUP BY transfer_type, minute ),
    PROJECTION prj_from_by_minute ( SELECT `from`, minute GROUP BY `from`, minute ),
    PROJECTION prj_to_by_minute ( SELECT `to`, minute GROUP BY `to`, minute ),
    PROJECTION prj_from_to_by_minute ( SELECT `from`, `to`, minute GROUP BY `from`, `to`, minute ),
    PROJECTION prj_log_address_by_minute ( SELECT log_address, minute GROUP BY log_address, minute ),
    PROJECTION prj_log_address_from_by_minute ( SELECT log_address, `from`, minute GROUP BY log_address, `from`, minute ),
    PROJECTION prj_log_address_to_by_minute ( SELECT log_address, `to`, minute GROUP BY log_address, `to`, minute ),
    PROJECTION prj_log_address_from_to_by_minute ( SELECT log_address, `from`, `to`, minute GROUP BY log_address, `from`, `to`, minute ),
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num,
    tx_index
)
COMMENT 'Transfers including ERC-20, WETH & Native value from calls & transactions';

-- MV's for Transfers --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_transactions TO transfers AS
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

    -- log --
    cast(NULL AS Nullable(UInt32)) AS log_index,
    '' AS log_address,
    cast(NULL AS Nullable(UInt32)) AS log_ordinal,

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

    -- log --
    cast(NULL AS Nullable(UInt32)) AS log_index,
    '' AS log_address,
    cast(NULL AS Nullable(UInt32)) AS log_ordinal,

    -- transfer --
    call_caller as `from`,
    call_address as `to`,
    call_value as amount,
    'call' AS transfer_type
FROM calls
WHERE amount > 0 AND `from` != `to`;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_erc20_transfers TO transfers AS
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

    -- log --
    log_index,
    log_address,
    log_ordinal,

    -- transfer --
    `from`,
    `to`,
    amount,
    'transfer' AS transfer_type
FROM erc20_transfers
WHERE amount > 0 AND `from` != `to`;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_weth_deposit TO transfers AS
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

    -- log --
    log_index,
    log_address,  -- WETH contract
    log_ordinal,

    -- transfer (native ETH leg) --
    dst AS `from`,          -- user
    log_address AS `to`,    -- WETH contract
    wad AS amount,
    'deposit' AS transfer_type
FROM weth_deposit
WHERE amount > 0;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_weth_withdrawal TO transfers AS
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

    -- log --
    log_index,
    log_address,  -- WETH contract
    log_ordinal,

    -- transfer (native ETH leg) --
    log_address AS `from`,  -- WETH contract
    src AS `to`,            -- user
    wad AS amount,
    'withdrawal' AS transfer_type
FROM weth_withdrawal
WHERE amount > 0;


CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_block_rewards TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    0 AS tx_index,
    '' AS tx_hash,

    -- call --
    cast(NULL AS Nullable(UInt32)) AS call_index,

    -- log --
    cast(NULL AS Nullable(UInt32)) AS log_index,
    '' AS log_address,
    cast(NULL AS Nullable(UInt32)) AS log_ordinal,

    -- transfer (native ETH leg) --
    '' AS `from`,
    miner AS `to`,
    value AS amount,
    lower(reason) AS transfer_type
FROM block_rewards
WHERE amount > 0;