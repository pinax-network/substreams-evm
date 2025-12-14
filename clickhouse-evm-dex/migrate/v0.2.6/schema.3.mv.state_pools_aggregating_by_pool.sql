-- Pool activity (Transactions) --
CREATE TABLE IF NOT EXISTS state_pools_aggregating_by_pool ON CLUSTER 'tokenapis-a' (
    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- DEX identity
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
    factory              LowCardinality(String),
    pool                 String,

    -- universal --
    uaw                     AggregateFunction(uniq, String) COMMENT 'unique wallet addresses',
    transactions            SimpleAggregateFunction(sum, UInt64) COMMENT 'total number of transactions',

    -- indexes --
    INDEX idx_min_timestamp     (min_timestamp)              TYPE minmax             GRANULARITY 1,
    INDEX idx_max_timestamp     (max_timestamp)              TYPE minmax             GRANULARITY 1,
    INDEX idx_min_block_num     (min_block_num)              TYPE minmax             GRANULARITY 1,
    INDEX idx_max_block_num     (max_block_num)              TYPE minmax             GRANULARITY 1,
    INDEX idx_protocol          (protocol)                   TYPE set(8)             GRANULARITY 1,
    INDEX idx_factory           (factory)                    TYPE set(1024)          GRANULARITY 1,
    INDEX idx_transactions      (transactions)               TYPE minmax             GRANULARITY 1
)
ENGINE = ReplicatedAggregatingMergeTree
ORDER BY (pool, factory, protocol);

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_aggregating_by_pool_swaps ON CLUSTER 'tokenapis-a'
TO state_pools_aggregating_by_pool
AS
SELECT
    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num,

    -- DEX identity
    protocol, factory, pool,

    -- universal --
    uniqState(user) AS uaw,
    count() as transactions
FROM swaps
GROUP BY protocol, factory, pool;