-- Pool initialization state derived from normalized initialize logs
CREATE TABLE IF NOT EXISTS state_pools_initialize (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- version: larger = "wins" => smallest block_num wins
    inv_block_num               Int64 MATERIALIZED (-toInt64(block_num)),

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

    -- indexes --
    INDEX idx_block_num         (block_num)         TYPE minmax           GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)         TYPE minmax           GRANULARITY 1,
    INDEX idx_minute            (minute)            TYPE minmax           GRANULARITY 1,
    INDEX idx_factory           (factory)           TYPE set(1024)        GRANULARITY 1,
    INDEX idx_protocol          (protocol)          TYPE set(8)           GRANULARITY 1
)
ENGINE = ReplacingMergeTree(inv_block_num)
ORDER BY (pool, factory, protocol);

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_initialize
TO state_pools_initialize
AS
SELECT
    block_num,
    block_hash,
    timestamp,
    tx_hash,
    factory,
    pool,
    protocol
FROM initialize;
