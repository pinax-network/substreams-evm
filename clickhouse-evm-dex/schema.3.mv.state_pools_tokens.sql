-- State Pools Tokens --
-- Aggregates token pair swap data per pool
CREATE TABLE IF NOT EXISTS state_pools_tokens (
    -- chain + DEX identity
    factory                 LowCardinality(String) COMMENT 'Factory contract address',
    pool                    String COMMENT 'Pool/exchange contract address',
    token                   String COMMENT 'token contract address',
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

    -- Projections --
    -- Search By Array Tokens --
    PROJECTION prj_group_token ( SELECT arraySort(groupArrayDistinct(token)), protocol, factory, pool GROUP BY protocol, factory, pool, token ),
)
ENGINE = ReplacingMergeTree
-- optimized for single token search --
ORDER BY (
    token, protocol, factory, pool
)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
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
