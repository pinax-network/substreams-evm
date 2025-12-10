-- State Pools Tokens --
-- Aggregates token pair swap data per pool
CREATE TABLE IF NOT EXISTS state_pools_tokens (
    -- chain + DEX identity
    pool                    LowCardinality(String) COMMENT 'Pool/exchange contract address',
    factory                 LowCardinality(String) COMMENT 'Factory contract address',
    protocol                LowCardinality(String) COMMENT 'DEX protocol name',

    -- token pair identity (directional)
    input_contract             LowCardinality(String) COMMENT 'Input token contract address',
    output_contract            LowCardinality(String) COMMENT 'Output token contract address',

    -- aggregated metrics
    transactions            SimpleAggregateFunction(sum, UInt64) COMMENT 'Total number of swaps for this token pair direction',

    -- indexes
    INDEX idx_pool              (pool)                              TYPE bloom_filter   GRANULARITY 1,
    INDEX idx_factory           (factory)                           TYPE set(1024)      GRANULARITY 1,
    INDEX idx_protocol          (protocol)                          TYPE set(4)         GRANULARITY 1,
    INDEX idx_input_contract    (input_contract)                    TYPE bloom_filter   GRANULARITY 1,
    INDEX idx_output_contract   (output_contract)                   TYPE bloom_filter   GRANULARITY 1,
    INDEX idx_token_pair        (input_contract, output_contract)   TYPE bloom_filter   GRANULARITY 1,
    INDEX idx_transactions      (transactions)                      TYPE minmax         GRANULARITY 1
)
ENGINE = AggregatingMergeTree
ORDER BY (
    pool, factory, protocol, input_contract, output_contract
)
COMMENT 'Aggregated token pair swap statistics per pool';

-- Materialized view to populate state_pools_tokens from swaps
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_tokens
TO state_pools_tokens
AS
SELECT
    pool,
    factory,
    protocol,
    input_contract,
    output_contract,
    count() AS transactions
FROM swaps
GROUP BY
    pool,
    factory,
    protocol,
    input_contract,
    output_contract;
