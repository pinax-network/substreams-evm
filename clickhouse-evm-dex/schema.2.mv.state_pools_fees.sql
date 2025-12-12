-- Pool Fees Tracking for Protocols with Dynamic Fees --
CREATE TABLE IF NOT EXISTS state_pools_fees (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_hash                     String,

    -- event --
    factory                     LowCardinality(String) COMMENT 'factory address',
    pool                        String COMMENT 'pool address',
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
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (protocol, factory, pool);

-- Uniswap::V1::Factory:NewExchange (Initialize) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v1_new_exchange_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v2_pair_created_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v3_pool_created_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v4_initialize_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_plain_pool_deployed_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_meta_pool_deployed_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_swap_fee_percentage
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_aggregate_swap_fee_percentage
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_bancor_conversion_fee_update
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_commit_new_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_new_fee
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_purchase_fee_set
TO state_pools_fees AS
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
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_sale_fee_set
TO state_pools_fees AS
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
