-- Pool activity (Transactions) --
CREATE TABLE IF NOT EXISTS state_pool_activity_summary (
    -- DEX identity
    protocol                    Enum8(
        'sunpump' = 1,
        'uniswap_v1' = 2,
        'uniswap_v2' = 3,
        'uniswap_v3' = 4,
        'uniswap_v4' = 5,
        'curvefi' = 6,
        'balancer' = 7,
        'bancor' = 8
    ) COMMENT 'protocol identifier',
    factory              LowCardinality(String),
    pool                 String,

    -- summing --
    transactions         UInt64,

    -- indexes --
    INDEX idx_protocol          (protocol)                  TYPE set(8)             GRANULARITY 1,
    INDEX idx_factory           (factory)                   TYPE set(1024)          GRANULARITY 1,
    INDEX idx_transactions      (transactions)              TYPE minmax             GRANULARITY 1
)
ENGINE = SummingMergeTree
ORDER BY (pool, factory, protocol);

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pool_activity_summary
TO state_pool_activity_summary
AS
SELECT
    protocol, factory, pool,

    -- summing --
    count() as transactions
FROM swaps
GROUP BY protocol, factory, pool;