-- Pool activity (Transactions) --
CREATE TABLE IF NOT EXISTS state_pools_aggregating_by_pool (
    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- DEX identity
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
        'kyber_elastic' = 14,
        'dca_dot_fun' = 15
    ) COMMENT 'protocol identifier',
    factory              LowCardinality(String),
    pool                 String,

    -- universal --
    transactions            SimpleAggregateFunction(sum, UInt64) COMMENT 'total number of transactions',

    -- indexes --
    INDEX idx_min_timestamp     (min_timestamp)              TYPE minmax             GRANULARITY 1,
    INDEX idx_max_timestamp     (max_timestamp)              TYPE minmax             GRANULARITY 1,
    INDEX idx_min_block_num     (min_block_num)              TYPE minmax             GRANULARITY 1,
    INDEX idx_max_block_num     (max_block_num)              TYPE minmax             GRANULARITY 1,
    INDEX idx_protocol          (protocol)                   TYPE set(8)             GRANULARITY 1,
    INDEX idx_factory           (factory)                    TYPE set(1024)          GRANULARITY 1,
    INDEX idx_transactions      (transactions)               TYPE minmax             GRANULARITY 1,

    -- projections --
    -- optimize for universal summary --
    PROJECTION prj_group_by_pool (
        SELECT
            -- timestamp & block number --
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num),

            -- DEX identity --
            pool,
            factory,
            protocol,

            -- universal --
            sum(transactions)
        GROUP BY pool, factory, protocol
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'Aggregating pools optimize for universal summary';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_aggregating_by_pool_swaps
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
    count() as transactions
FROM swaps
GROUP BY protocol, factory, pool;