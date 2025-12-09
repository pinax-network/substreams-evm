-- Pool Fees Tracking for Protocols with Dynamic Fees --
CREATE TABLE IF NOT EXISTS pools_fees (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_hash                     String,

    -- event --
    pool                        LowCardinality(String) COMMENT 'pool address',
    fee                         UInt32 COMMENT 'updated pool fee (e.g., 3000 represents 0.30%)',
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
    INDEX idx_pool                 (pool)              TYPE set(64) GRANULARITY 4,
    INDEX idx_protocol             (protocol)          TYPE set(8) GRANULARITY 4,
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (pool, protocol);

-- Balancer::SwapFeePercentage (V2) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_swap_fee_percentage
TO pools_fees AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    log_address AS pool,
    toUInt32(toFloat64(swap_fee_percentage) / 10000000000000000) AS fee, -- Convert from basis points (1e18) to basis points (1e4)
    'balancer' AS protocol
FROM balancer_swap_fee_percentage;

-- Balancer::AggregateSwapFeePercentage (V3) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_aggregate_swap_fee_percentage
TO pools_fees AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    pool,
    toUInt32(toFloat64(aggregate_swap_fee_percentage) / 10000000000000000) AS fee, -- Convert from basis points (1e18) to basis points (1e4)
    'balancer' AS protocol
FROM balancer_aggregate_swap_fee_percentage;

-- Bancor::ConversionFeeUpdate --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_bancor_conversion_fee_update
TO pools_fees AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    log_address AS pool,
    new_fee AS fee,
    'bancor' AS protocol
FROM bancor_conversion_fee_update;

-- Curve.fi::CommitNewFee --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_commit_new_fee
TO pools_fees AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    log_address AS pool,
    toUInt32(fee) AS fee,
    'curvefi' AS protocol
FROM curvefi_commit_new_fee;

-- Curve.fi::NewFee --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_new_fee
TO pools_fees AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    log_address AS pool,
    toUInt32(fee) AS fee,
    'curvefi' AS protocol
FROM curvefi_new_fee;

-- SunPump::PurchaseFeeSet --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_purchase_fee_set
TO pools_fees AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    log_address AS pool,
    toUInt32(new_fee) AS fee,
    'sunpump' AS protocol
FROM sunpump_purchase_fee_set;

-- SunPump::SaleFeeSet --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_sale_fee_set
TO pools_fees AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_hash,

    -- event --
    log_address AS pool,
    toUInt32(new_fee) AS fee,
    'sunpump' AS protocol
FROM sunpump_sale_fee_set;
