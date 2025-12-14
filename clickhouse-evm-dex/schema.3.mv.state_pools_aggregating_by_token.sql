-- Pool activity (Transactions) --
CREATE TABLE IF NOT EXISTS state_pools_aggregating_by_token (
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
        'bancor' = 8
    ) COMMENT 'protocol identifier',
    factory              LowCardinality(String),
    pool                 String,
    token                String,

    -- universal --
    uniq_user               AggregateFunction(uniq, String) COMMENT 'unique user addresses',
    uniq_tx_from            AggregateFunction(uniq, String) COMMENT 'unique transaction from addresses',
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
    -- optimize for grouped array token --
    PROJECTION prj_group_array_distinct_token (
        SELECT
            arraySort(groupArrayDistinct(token)),
            pool,
            factory,
            protocol,
            sum(transactions),
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num)
        GROUP BY pool, factory, protocol
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (token, pool, factory, protocol)
SETTINGS deduplicate_merge_projection_mode = 'rebuild';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_aggregating_by_token_input_contract
TO state_pools_aggregating_by_token
AS
SELECT
    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num,

    -- DEX identity
    protocol, factory, pool,
    input_contract AS token,

    -- universal --
    uniqState(user) AS uniq_user,
    uniqState(tx_from) AS uniq_tx_from,
    count() as transactions
FROM swaps
GROUP BY token, protocol, factory, pool;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_aggregating_by_token_output_contract
TO state_pools_aggregating_by_token
AS
SELECT
    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num,

    -- DEX identity
    protocol, factory, pool,
    output_contract AS token,

    -- universal --
    uniqState(user) AS uniq_user,
    uniqState(tx_from) AS uniq_tx_from,
    count() as transactions
FROM swaps
GROUP BY token, protocol, factory, pool;