-- Pool Initialize Events for All Supported DEX Protocols --
CREATE TABLE IF NOT EXISTS state_pools_initialize ON CLUSTER 'tokenapis-a' (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- version: larger = "wins" => smallest block_num wins
    inv_block_num Int64 MATERIALIZED (-toInt64(block_num)),

    -- transaction --
    tx_hash                     String,

    -- DEX identity
    factory                 LowCardinality(String) COMMENT 'Factory contract address',
    pool                    String COMMENT 'Pool/exchange contract address',
    protocol                Enum8(
        'sunpump' = 1,
        'uniswap_v1' = 2,
        'uniswap_v2' = 3,
        'uniswap_v3' = 4,
        'uniswap_v4' = 5,
        'curvefi' = 6,
        'balancer' = 7,
        'bancor' = 8
    ) COMMENT 'protocol identifier',

    -- indexes --
    INDEX idx_block_num         (block_num)         TYPE minmax           GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)         TYPE minmax           GRANULARITY 1,
    INDEX idx_minute            (minute)            TYPE minmax           GRANULARITY 1,
    INDEX idx_factory           (factory)           TYPE set(1024)        GRANULARITY 1,
    INDEX idx_protocol          (protocol)          TYPE set(8)           GRANULARITY 1
)
ENGINE = ReplicatedReplacingMergeTree(inv_block_num)
ORDER BY (pool, factory, protocol);

-- Uniswap::V2::Factory:PairCreated --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_uniswap_v2_pair_created ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    pair AS pool,
    'uniswap_v2' AS protocol
FROM uniswap_v2_pair_created;

-- Uniswap::V3::Factory:PoolCreated --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_uniswap_v3_pool_created ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    pool,
    'uniswap_v3' AS protocol
FROM uniswap_v3_pool_created;

-- Uniswap::V4::IPoolManager:Initialize --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_uniswap_v4_initialize ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- log --
    factory,

    -- event --
    id as pool,
    'uniswap_v4' AS protocol
FROM uniswap_v4_initialize;

-- Uniswap::V1::Factory:NewExchange --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_uniswap_v1_new_exchange ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    exchange AS pool,
    'uniswap_v1' AS protocol
FROM uniswap_v1_new_exchange;

-- SunPump::TokenCreate --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_sunpump_token_create ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    token_address AS pool,
    'sunpump' AS protocol
FROM sunpump_token_create;

-- SunPump::TokenCreateLegacy --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_sunpump_token_create_legacy ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    token_address AS pool,
    'sunpump' AS protocol
FROM sunpump_token_create_legacy;

-- Curve.fi::PlainPoolDeployed --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_curvefi_plain_pool_deployed ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    address AS pool,
    'curvefi' AS protocol
FROM curvefi_plain_pool_deployed;

-- Curve.fi::MetaPoolDeployed --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_curvefi_meta_pool_deployed ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    address AS pool,
    'curvefi' AS protocol
FROM curvefi_meta_pool_deployed;

-- Balancer::PoolRegistered --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_balancer_pool_registered ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    pool,
    'balancer' AS protocol
FROM balancer_pool_registered;

-- Bancor::Activation --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize_bancor_activation ON CLUSTER 'tokenapis-a'
TO state_pools_initialize AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    factory,
    converter AS pool,
    'bancor' AS protocol
FROM bancor_new_converter;
