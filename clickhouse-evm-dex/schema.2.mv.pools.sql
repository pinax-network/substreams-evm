-- Pools Created for Uniswap V2 & V3 --
CREATE TABLE IF NOT EXISTS pools (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_hash                     String,

    -- event --
    factory                     LowCardinality(String) COMMENT 'factory address',
    pool                        LowCardinality(String) COMMENT 'pool address',
    token0                      LowCardinality(String) COMMENT 'token0 address',
    token1                      LowCardinality(String) COMMENT 'token1 address',
    fee                         UInt32 COMMENT 'pool fee (e.g., 3000 represents 0.30%)',
    protocol                    Enum8(
        'sunpump' = 1,
        'uniswap-v1' = 2,
        'uniswap-v2' = 3,
        'uniswap-v3' = 4,
        'uniswap-v4' = 5,
        'curvefi' = 6,
        'balancer' = 7,
        'bancor' = 8
    ) COMMENT 'protocol identifier',

    -- indexes --
    INDEX idx_tx_hash              (tx_hash)           TYPE bloom_filter GRANULARITY 4,
    INDEX idx_factory              (factory)           TYPE set(64) GRANULARITY 4,
    INDEX idx_token0               (token0)            TYPE set(64) GRANULARITY 4,
    INDEX idx_token1               (token1)            TYPE set(64) GRANULARITY 4,
    INDEX idx_fee                  (fee)               TYPE minmax GRANULARITY 4,
    INDEX idx_protocol             (protocol)          TYPE set(8) GRANULARITY 4,
)
ENGINE = ReplacingMergeTree
ORDER BY (pool, factory);

-- Uniswap::V2::Factory:PairCreated --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v2_pair_created
TO pools AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    address AS factory,
    pair AS pool,
    token0,
    token1,
    3000 AS fee, -- default Uniswap V2 fee
    'uniswap_v2' AS protocol
FROM uniswap_v2_pair_created;

-- Uniswap::V3::Factory:PoolCreated --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v3_pool_created
TO pools AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    address AS factory,
    pool,
    token0,
    token1,
    fee,
    'uniswap_v3' AS protocol
FROM uniswap_v3_pool_created;

-- Uniswap::V4::IPoolManager:Initialize --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v4_initialize
TO pools AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- log --
    address AS factory,

    -- event --
    id as pool,
    currency0 as token0,
    currency1 as token1,
    fee,
    'uniswap_v4' AS protocol
FROM uniswap_v4_initialize;
