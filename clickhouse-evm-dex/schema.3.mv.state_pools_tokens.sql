-- State Pools Tokens --
-- Aggregates token pair swap data per pool
CREATE TABLE IF NOT EXISTS state_pools_tokens (
    -- chain + DEX identity
    pool                    LowCardinality(String) COMMENT 'Pool/exchange contract address',
    factory                 LowCardinality(String) COMMENT 'Factory contract address',
    protocol                Enum8(
        'sunpump' = 1,
        'uniswap-v1' = 2,
        'uniswap-v2' = 3,
        'uniswap-v3' = 4,
        'uniswap-v4' = 5,
        'curvefi' = 6,
        'balancer' = 7,
        'bancor' = 8
    ) COMMENT 'protocol identifier',
    token                   LowCardinality(String) COMMENT 'token contract address',

    -- indexes
    INDEX idx_factory           (factory)                           TYPE set(1024)      GRANULARITY 1,
    INDEX idx_protocol          (protocol)                          TYPE set(8)         GRANULARITY 1,
    INDEX idx_token             (token)                             TYPE bloom_filter   GRANULARITY 1,
)
ENGINE = ReplacingMergeTree
ORDER BY (
    pool, factory, token, protocol
)
COMMENT 'State table aggregating token pair swap data per pool';

-- Materialized view to populate state_pools_tokens from swaps
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_tokens
TO state_pools_tokens
AS
SELECT
    pool,
    factory,
    protocol,
    input_contract AS token
FROM swaps
GROUP BY
    pool,
    factory,
    protocol,
    token;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_tokens_inverse
TO state_pools_tokens
AS
SELECT
    pool,
    factory,
    protocol,
    output_contract AS token
FROM swaps
GROUP BY
    pool,
    factory,
    protocol,
    token;
