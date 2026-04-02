-- Pool fee state derived from normalized swap fee logs
CREATE TABLE IF NOT EXISTS state_pools_fees (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- transaction --
    tx_hash                     String,

    -- DEX identity --
    factory                     LowCardinality(String),
    pool                        String,
    protocol                    Enum8(
        'sunpump' = 1,
        'uniswap_v1' = 2,
        'uniswap_v2' = 3,
        'uniswap_v3' = 4,
        'uniswap_v4' = 5,
        'curvefi' = 6,
        'balancer' = 7,
        'bancor' = 8,
        'cow' = 9,
        'aerodrome' = 10,
        'dodo' = 11,
        'woofi' = 12,
        'traderjoe' = 13,
        'kyber_elastic' = 14
    ),

    -- state --
    fee                         UInt32,

    -- indexes --
    INDEX idx_block_num         (block_num)         TYPE minmax           GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)         TYPE minmax           GRANULARITY 1,
    INDEX idx_minute            (minute)            TYPE minmax           GRANULARITY 1,
    INDEX idx_factory           (factory)           TYPE set(1024)        GRANULARITY 1,
    INDEX idx_protocol          (protocol)          TYPE set(8)           GRANULARITY 1
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (pool, factory, protocol);

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_fees
TO state_pools_fees
AS
SELECT
    block_num,
    block_hash,
    timestamp,
    tx_hash,
    factory,
    pool,
    protocol,
    fee
FROM swap_fee;
