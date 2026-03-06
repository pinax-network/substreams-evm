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
    address              String COMMENT 'normalized unique address across user, tx_from, and call_caller',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- projections --
    PROJECTION prj_factory_address (
        SELECT
            factory,
            address,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY factory, address
    ),
    PROJECTION prj_pool_factory_address (
        SELECT
            pool,
            factory,
            address,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY pool, factory, address
    ),
    PROJECTION prj_factory_count (
        SELECT
            factory,
            count()
        GROUP BY factory
    ),
    PROJECTION prj_pool_factory_count (
        SELECT
            pool,
            factory,
            count()
        GROUP BY pool, factory
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol, address)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'Normalized unique addresses per pool across user, tx_from, and call_caller';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_uaw
TO state_pools_uaw
AS
SELECT
    protocol,
    factory,
    pool,
    address,
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num
FROM (
    SELECT protocol, factory, pool, user AS address, timestamp, block_num
    FROM swaps
    WHERE user != ''

    UNION ALL

    SELECT protocol, factory, pool, tx_from AS address, timestamp, block_num
    FROM swaps
    WHERE tx_from != ''

    UNION ALL

    SELECT protocol, factory, pool, call_caller AS address, timestamp, block_num
    FROM swaps
    WHERE call_caller != ''
)
GROUP BY protocol, factory, pool, address;

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
    ),
    PROJECTION prj_factory_user_count (
        SELECT
            factory,
            count()
        GROUP BY factory
    ),
    PROJECTION prj_pool_factory_user_count (
        SELECT
            pool,
            factory,
            count()
        GROUP BY pool, factory
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol, user)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
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
    ),
    PROJECTION prj_factory_tx_from_count (
        SELECT
            factory,
            count()
        GROUP BY factory
    ),
    PROJECTION prj_pool_factory_tx_from_count (
        SELECT
            pool,
            factory,
            count()
        GROUP BY pool, factory
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol, tx_from)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
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

-- UAW by call_caller address --
CREATE TABLE IF NOT EXISTS state_pools_uaw_by_call_caller (
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
    call_caller               String COMMENT 'unique call_caller address',

    -- timestamp & block number --
    min_timestamp         SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp         SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num         SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num         SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- projections --
    PROJECTION prj_factory_call_caller (
        SELECT
            factory,
            call_caller,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY factory, call_caller
    ),
    PROJECTION prj_pool_factory_call_caller (
        SELECT
            pool,
            factory,
            call_caller,
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY pool, factory, call_caller
    ),
    PROJECTION prj_factory_call_caller_count (
        SELECT
            factory,
            count()
        GROUP BY factory
    ),
    PROJECTION prj_pool_factory_call_caller_count (
        SELECT
            pool,
            factory,
            count()
        GROUP BY pool, factory
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (pool, factory, protocol, call_caller)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'Unique call_caller addresses per pool for UAW calculation';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_uaw_by_call_caller
TO state_pools_uaw_by_call_caller
AS
SELECT
    protocol, factory, pool, call_caller,
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num
FROM swaps
GROUP BY protocol, factory, pool, call_caller;
