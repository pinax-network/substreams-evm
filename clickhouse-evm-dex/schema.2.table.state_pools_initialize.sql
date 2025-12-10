-- Pool Initialize Events for All Supported DEX Protocols --
CREATE TABLE IF NOT EXISTS state_pools_initialize (
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
    tokens                      Array(String) COMMENT 'token addresses in the pool',
    token0                      LowCardinality(String) COMMENT 'first token address' MATERIALIZED if(length(tokens) >= 1, tokens[1], ''),
    token1                      LowCardinality(String) COMMENT 'second token address' MATERIALIZED if(length(tokens) >= 2, tokens[2], ''),
    token2                      LowCardinality(String) COMMENT 'third token address' MATERIALIZED if(length(tokens) >= 3, tokens[3], ''),
    token3                      LowCardinality(String) COMMENT 'fourth token address' MATERIALIZED if(length(tokens) >= 4, tokens[4], ''),
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
    INDEX idx_protocol             (protocol)          TYPE set(8) GRANULARITY 4,
)
ENGINE = ReplacingMergeTree
ORDER BY (pool, factory);

-- Uniswap::V2::Factory:PairCreated --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v2_pair_created
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
    address AS factory,
    pair AS pool,
    [token0, token1] AS tokens,
    'uniswap_v2' AS protocol
FROM uniswap_v2_pair_created;

-- Uniswap::V3::Factory:PoolCreated --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v3_pool_created
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
    address AS factory,
    pool,
    [token0, token1] AS tokens,
    'uniswap_v3' AS protocol
FROM uniswap_v3_pool_created;

-- Uniswap::V4::IPoolManager:Initialize --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v4_initialize
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
    address AS factory,

    -- event --
    id as pool,
    [currency0, currency1] AS tokens,
    'uniswap_v4' AS protocol
FROM uniswap_v4_initialize;

-- Uniswap::V1::Factory:NewExchange --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v1_new_exchange
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
    log_address AS factory,
    exchange AS pool,
    ['0x0000000000000000000000000000000000000000', token] AS tokens, -- ETH (represented as zero address) and token
    'uniswap_v1' AS protocol
FROM uniswap_v1_new_exchange;

-- SunPump::TokenCreate --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_create
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
    log_address AS factory,
    token_address AS pool,
    ['0x0000000000000000000000000000000000000000', token_address] AS tokens, -- TRX (represented as zero address) and token
    'sunpump' AS protocol
FROM sunpump_token_create;

-- SunPump::TokenCreateLegacy --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_create_legacy
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
    log_address AS factory,
    token_address AS pool,
    ['0x0000000000000000000000000000000000000000', token_address] AS tokens, -- TRX (represented as zero address) and token
    'sunpump' AS protocol
FROM sunpump_token_create_legacy;

-- Curve.fi::PlainPoolDeployed --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_plain_pool_deployed
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
    log_address AS factory,
    address AS pool,
    splitByChar(',', coins) AS tokens,
    'curvefi' AS protocol
FROM curvefi_plain_pool_deployed
WHERE length(splitByChar(',', coins)) >= 2
  AND arrayElement(splitByChar(',', coins), 1) != ''
  AND arrayElement(splitByChar(',', coins), 2) != '';

-- Curve.fi::MetaPoolDeployed --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_meta_pool_deployed
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
    log_address AS factory,
    address AS pool,
    [coin, base_pool] AS tokens,
    'curvefi' AS protocol
FROM curvefi_meta_pool_deployed;

-- Balancer::PoolRegistered --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_pool_registered
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
    log_address AS factory,
    pool,
    [] AS tokens, -- Balancer pools have variable tokens, populated separately
    'balancer' AS protocol
FROM balancer_pool_registered;

-- Bancor::Activation --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_bancor_activation
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
    log_address AS factory,
    anchor AS pool,
    [] AS tokens, -- Bancor converters have variable reserve tokens, populated separately
    'bancor' AS protocol
FROM bancor_activation
WHERE activated = true;
