-- Pools Created for All Supported DEX Protocols --
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
    fee                         Nullable(UInt32) COMMENT 'pool fee (e.g., 3000 represents 0.30%), NULL if not applicable or fee is dynamic',
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

-- Uniswap::V1::Factory:NewExchange --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v1_new_exchange
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
    log_address AS factory,
    exchange AS pool,
    '0x0000000000000000000000000000000000000000' AS token0, -- ETH (represented as zero address)
    token AS token1,
    3000 AS fee, -- default Uniswap V1 fee (0.3%)
    'uniswap_v1' AS protocol
FROM uniswap_v1_new_exchange;

-- SunPump::TokenCreate --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_create
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
    log_address AS factory,
    token_address AS pool,
    '0x0000000000000000000000000000000000000000' AS token0, -- TRX (represented as zero address)
    token_address AS token1,
    NULL AS fee, -- SunPump has dynamic fees
    'sunpump' AS protocol
FROM sunpump_token_create;

-- SunPump::TokenCreateLegacy --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_create_legacy
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
    log_address AS factory,
    token_address AS pool,
    '0x0000000000000000000000000000000000000000' AS token0, -- TRX (represented as zero address)
    token_address AS token1,
    NULL AS fee, -- SunPump has dynamic fees
    'sunpump' AS protocol
FROM sunpump_token_create_legacy;

-- Curve.fi::PlainPoolDeployed --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_plain_pool_deployed
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
    log_address AS factory,
    address AS pool,
    arrayElement(splitByChar(',', coins), 1) AS token0,
    arrayElement(splitByChar(',', coins), 2) AS token1,
    toUInt32(fee) AS fee, -- CurveFi fee is provided at pool creation
    'curvefi' AS protocol
FROM curvefi_plain_pool_deployed
WHERE length(splitByChar(',', coins)) >= 2;

-- Curve.fi::MetaPoolDeployed --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_meta_pool_deployed
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
    log_address AS factory,
    address AS pool,
    coin AS token0,
    base_pool AS token1,
    toUInt32(fee) AS fee, -- CurveFi fee is provided at pool creation
    'curvefi' AS protocol
FROM curvefi_meta_pool_deployed;

-- Balancer::PoolRegistered --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_pool_registered
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
    log_address AS factory,
    pool,
    '' AS token0, -- Balancer pools can have multiple tokens, we'll leave these empty
    '' AS token1,
    NULL AS fee, -- Balancer has dynamic fees
    'balancer' AS protocol
FROM balancer_pool_registered;

-- Bancor::Activation --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_bancor_activation
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
    log_address AS factory,
    anchor AS pool,
    '' AS token0, -- Bancor converters can have multiple reserve tokens
    '' AS token1,
    NULL AS fee, -- Bancor has dynamic fees
    'bancor' AS protocol
FROM bancor_activation
WHERE activated = true;
