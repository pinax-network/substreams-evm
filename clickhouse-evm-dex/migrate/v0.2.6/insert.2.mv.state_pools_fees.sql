-- Uniswap::V1::Factory:NewExchange (Initialize) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    3000 AS fee, -- default Uniswap V1 fee (0.3%)
    'uniswap-v1' AS protocol
FROM uniswap_v1_new_exchange;

-- Uniswap::V2::Factory:PairCreated (Initialize) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    pair AS pool,
    3000 AS fee, -- default Uniswap V2 fee (0.3%)
    'uniswap-v2' AS protocol
FROM uniswap_v2_pair_created;

-- Uniswap::V3::Factory:PoolCreated (Initialize) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    fee,
    'uniswap-v3' AS protocol
FROM uniswap_v3_pool_created;

-- Uniswap::V4::IPoolManager:Initialize (Initialize) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    id AS pool,
    fee,
    'uniswap-v4' AS protocol
FROM uniswap_v4_initialize;

-- Curve.fi::PlainPoolDeployed (Initialize) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    toUInt32(fee) AS fee,
    'curvefi' AS protocol
FROM curvefi_plain_pool_deployed;

-- Curve.fi::MetaPoolDeployed (Initialize) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    toUInt32(fee) AS fee,
    'curvefi' AS protocol
FROM curvefi_meta_pool_deployed;

-- Balancer::SwapFeePercentage (V2) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    log_address AS pool,
    toUInt32(toFloat64(swap_fee_percentage) / 100000000000000) AS fee, -- Convert from 1e18 fixed-point to basis points (1e4): divide by 1e14
    'balancer' AS protocol
FROM balancer_swap_fee_percentage;

-- Balancer::AggregateSwapFeePercentage (V3) --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    toUInt32(toFloat64(aggregate_swap_fee_percentage) / 100000000000000) AS fee, -- Convert from 1e18 fixed-point to basis points (1e4): divide by 1e14
    'balancer' AS protocol
FROM balancer_aggregate_swap_fee_percentage;

-- Bancor::ConversionFeeUpdate --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    log_address AS pool,
    new_fee AS fee,
    'bancor' AS protocol
FROM bancor_conversion_fee_update;

-- Curve.fi::CommitNewFee --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    log_address AS pool,
    toUInt32(fee) AS fee,
    'curvefi' AS protocol
FROM curvefi_commit_new_fee;

-- Curve.fi::NewFee --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    log_address AS pool,
    toUInt32(fee) AS fee,
    'curvefi' AS protocol
FROM curvefi_new_fee;

-- SunPump::PurchaseFeeSet --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    log_address AS pool,
    toUInt32(new_fee) AS fee,
    'sunpump' AS protocol
FROM sunpump_purchase_fee_set;

-- SunPump::SaleFeeSet --
INSERT INTO state_pools_fees (block_num, block_hash, timestamp, minute, tx_hash, factory, pool, fee, protocol)
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
    log_address AS pool,
    toUInt32(new_fee) AS fee,
    'sunpump' AS protocol
FROM sunpump_sale_fee_set;
