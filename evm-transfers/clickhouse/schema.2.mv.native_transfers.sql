-- Transfers including ERC-20, WETH & Native value from calls & transactions --
CREATE TABLE IF NOT EXISTS native_transfers (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    UInt32, -- derived from Substreams
    tx_hash                     String,
    tx_from                     String,
    tx_to                       LowCardinality(String),
    tx_nonce                    UInt64,
    tx_gas_price                UInt256,
    tx_gas_limit                UInt64,
    tx_gas_used                 UInt64,
    tx_value                    UInt256,

    -- call metadata --
    call_caller                 String,
    call_index                  UInt32,
    call_depth                  UInt32,
    call_type                   LowCardinality(String),

    -- transfer --
    `from`                      String,
    `to`                        String,
    amount                      UInt256,

    -- type --
    transfer_type               Enum8('transaction' = 1, 'call' = 2, 'block_reward' = 3, 'genesis_balance' = 4, 'dao_transfer' = 5),

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
    PROJECTION prj_to_by_minute ( SELECT `to`, minute GROUP BY `to`, minute )
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
)
COMMENT 'Transfers Native value from calls & transactions';

-- MV's for Transfers --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_native_transfers_transactions TO native_transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call metadata --
    '' AS call_caller,
    0 AS call_index,
    0 AS call_depth,
    '' AS call_type,

    -- transfer --
    tx_from as `from`,
    tx_to as `to`,
    tx_value as amount,
    'transaction' AS transfer_type
FROM transactions
WHERE amount > 0 AND `from` != `to`;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_native_transfers_calls TO native_transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call metadata --
    call_caller,
    call_index,
    call_depth,
    call_type,

    -- transfer --
    call_caller as `from`,
    call_address as `to`,
    call_value as amount,
    'call' AS transfer_type
FROM calls
WHERE amount > 0 AND `from` != `to`;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_native_transfers_block_rewards TO native_transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --,
    0 AS tx_index,
    '' AS tx_hash,
    '' AS tx_from,
    '' AS tx_to,
    0 AS tx_nonce,
    '0' AS tx_gas_price,
    0 AS tx_gas_limit,
    0 AS tx_gas_used,
    '0' AS tx_value,

    -- call metadata --
    '' AS call_caller,
    0 AS call_index,
    0 AS call_depth,
    '' AS call_type,

    -- transfer (native ETH leg) --
    '' AS `from`,
    miner AS `to`,
    value AS amount,
    'block_reward' AS transfer_type
FROM block_rewards
WHERE amount > 0;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_native_transfers_genesis_balances TO native_transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    0 AS tx_index,
    '' AS tx_hash,
    '' AS tx_from,
    '' AS tx_to,
    0 AS tx_nonce,
    '0' AS tx_gas_price,
    0 AS tx_gas_limit,
    0 AS tx_gas_used,
    '0' AS tx_value,

    -- call metadata --
    '' AS call_caller,
    0 AS call_index,
    0 AS call_depth,
    '' AS call_type,

    -- transfer (genesis allocation) --
    '' AS `from`,
    address AS `to`,
    value AS amount,
    'genesis_balance' AS transfer_type
FROM genesis_balances
WHERE value > 0;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_native_transfers_dao_transfers TO native_transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    0 AS tx_index,
    '' AS tx_hash,
    '' AS tx_from,
    '' AS tx_to,
    0 AS tx_nonce,
    '0' AS tx_gas_price,
    0 AS tx_gas_limit,
    0 AS tx_gas_used,
    '0' AS tx_value,

    -- call metadata --
    '' AS call_caller,
    0 AS call_index,
    0 AS call_depth,
    '' AS call_type,

    -- transfer (DAO hard fork) --
    address AS `from`,
    '' AS `to`,
    old_value - new_value AS amount,
    'dao_transfer' AS transfer_type
FROM dao_transfers
WHERE old_value > new_value;
