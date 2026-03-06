-- Unique Active Wallets (UAW) tables --
-- These tables use AggregatingMergeTree for efficient distinct-address counting

CREATE TABLE IF NOT EXISTS state_pools_uaw (
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
        'kyber_elastic' = 14
    ) COMMENT 'protocol identifier',
    factory              LowCardinality(String),
    pool                 String,
    dimension            Enum8(
        'user' = 1,
        'tx_from' = 2,
        'caller' = 3
    ) COMMENT 'address dimension type',
    address              String COMMENT 'normalized address for the selected dimension',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- projections --
    PROJECTION prj_factory_address (
        SELECT
            dimension,
            factory,
            address,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY dimension, factory, address
    ),
    PROJECTION prj_pool_factory_address (
        SELECT
            dimension,
            pool,
            factory,
            address,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY dimension, pool, factory, address
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (dimension, pool, factory, protocol, address)
COMMENT 'Normalized unique addresses per pool for caller, user, and tx_from analytics';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_uaw
TO state_pools_uaw
AS
SELECT
    protocol,
    factory,
    pool,
    dimension,
    address,
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num
FROM (
    SELECT protocol, factory, pool, 'user' AS dimension, user AS address, timestamp, block_num
    FROM swaps

    UNION ALL

    SELECT protocol, factory, pool, 'tx_from' AS dimension, tx_from AS address, timestamp, block_num
    FROM swaps

    UNION ALL

    SELECT protocol, factory, pool, 'caller' AS dimension, caller AS address, timestamp, block_num
    FROM swaps
)
GROUP BY protocol, factory, pool, dimension, address;

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
        'kyber_elastic' = 14
    ) COMMENT 'protocol identifier',
    factory              LowCardinality(String),
    pool                 String,
    user                 String COMMENT 'unique user wallet address',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- projections --
    PROJECTION prj_factory_user (
        SELECT
            factory,
            user,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY factory, user
    ),
    PROJECTION prj_pool_factory_user (
        SELECT
            pool,
            factory,
            user,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY pool, factory, user
    )
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
        'kyber_elastic' = 14
    ) COMMENT 'protocol identifier',
    factory              LowCardinality(String),
    pool                 String,
    tx_from              String COMMENT 'unique transaction from address',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- projections --
    PROJECTION prj_factory_tx_from (
        SELECT
            factory,
            tx_from,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY factory, tx_from
    ),
    PROJECTION prj_pool_factory_tx_from (
        SELECT
            pool,
            factory,
            tx_from,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY pool, factory, tx_from
    )
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

-- UAW by caller address --
CREATE TABLE IF NOT EXISTS state_pools_uaw_by_caller (
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
        'kyber_elastic' = 14
    ) COMMENT 'protocol identifier',
    factory              LowCardinality(String),
    pool                 String,
    caller               String COMMENT 'unique caller address',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- projections --
    PROJECTION prj_factory_caller (
        SELECT
            factory,
            caller,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY factory, caller
    ),
    PROJECTION prj_pool_factory_caller (
        SELECT
            pool,
            factory,
            caller,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY pool, factory, caller
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol, caller)
COMMENT 'Unique caller addresses per pool for UAW calculation';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_uaw_by_caller
TO state_pools_uaw_by_caller
AS
SELECT
    protocol, factory, pool, caller,
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num
FROM swaps
GROUP BY protocol, factory, pool, caller;
