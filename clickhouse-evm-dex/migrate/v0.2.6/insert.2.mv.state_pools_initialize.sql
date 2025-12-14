-- Uniswap::V2::Factory:PairCreated --
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
    'uniswap-v2' AS protocol
FROM uniswap_v2_pair_created;

-- Uniswap::V3::Factory:PoolCreated --
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
    'uniswap-v3' AS protocol
FROM uniswap_v3_pool_created;

-- Uniswap::V4::IPoolManager:Initialize --
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
    'uniswap-v4' AS protocol
FROM uniswap_v4_initialize;

-- Uniswap::V1::Factory:NewExchange --
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
    'uniswap-v1' AS protocol
FROM uniswap_v1_new_exchange;

-- SunPump::TokenCreate --
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
INSERT INTO state_pools_initialize (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, protocol)
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
