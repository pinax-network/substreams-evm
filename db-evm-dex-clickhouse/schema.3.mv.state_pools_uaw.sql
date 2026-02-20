-- Unique Active Wallets (UAW) tables --
-- These tables use AggregatingMergeTree for efficient unique user counting
-- Instead of expensive uniqMerge operations, use: SELECT count() FROM state_pools_uaw_by_user WHERE pool = '<pool>'

-- UAW by user address --
CREATE TABLE IF NOT EXISTS state_pools_uaw_by_user (
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
    user                 String COMMENT 'unique user wallet address',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- indexes --
    INDEX idx_protocol          (protocol)                   TYPE set(8)             GRANULARITY 1,
    INDEX idx_factory           (factory)                    TYPE set(1024)          GRANULARITY 1,
    INDEX idx_min_timestamp     (min_timestamp)              TYPE minmax             GRANULARITY 1,
    INDEX idx_max_timestamp     (max_timestamp)              TYPE minmax             GRANULARITY 1
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol, user)
COMMENT 'Unique user addresses per pool for UAW calculation';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_uaw_by_user
TO state_pools_uaw_by_user
AS
SELECT
    -- DEX identity
    protocol, factory, pool, user,

    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num
FROM swaps
GROUP BY protocol, factory, pool, user;

-- UAW by tx_from address --
CREATE TABLE IF NOT EXISTS state_pools_uaw_by_tx_from (
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
    tx_from              String COMMENT 'unique transaction from address',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- indexes --
    INDEX idx_protocol          (protocol)                   TYPE set(8)             GRANULARITY 1,
    INDEX idx_factory           (factory)                    TYPE set(1024)          GRANULARITY 1,
    INDEX idx_min_timestamp     (min_timestamp)              TYPE minmax             GRANULARITY 1,
    INDEX idx_max_timestamp     (max_timestamp)              TYPE minmax             GRANULARITY 1
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol, tx_from)
COMMENT 'Unique tx_from addresses per pool for UAW calculation';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_uaw_by_tx_from
TO state_pools_uaw_by_tx_from
AS
SELECT
    -- DEX identity
    protocol, factory, pool, tx_from,

    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num
FROM swaps
GROUP BY protocol, factory, pool, tx_from;
