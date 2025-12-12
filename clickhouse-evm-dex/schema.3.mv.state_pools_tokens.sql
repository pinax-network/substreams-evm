-- State Pools Tokens --
-- Aggregates token pair swap data per pool
CREATE TABLE IF NOT EXISTS state_pools_tokens (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_hash                     String,

    -- DEX identity
    factory                 LowCardinality(String) COMMENT 'Factory contract address',
    pool                    String COMMENT 'Pool/exchange contract address',
    protocol                Enum8(
        'sunpump' = 1,
        'uniswap_v1' = 2,
        'uniswap_v2' = 3,
        'uniswap_v3' = 4,
        'uniswap_v4' = 5,
        'curvefi' = 6,
        'balancer' = 7,
        'bancor' = 8
    ) COMMENT 'protocol identifier',

    -- state --
    token                   String COMMENT 'token contract address',

    -- Projections --
    -- optimize for grouped array token --
    PROJECTION prj_group_array_distinct ( SELECT arraySort(groupArrayDistinct(token)), protocol, factory, pool GROUP BY protocol, factory, pool, token ),
    -- optimized for single token --
    PROJECTION prj_order_by_token ( SELECT token, protocol, factory, pool ORDER BY token, protocol, factory, pool ),
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (protocol, factory, pool, token)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'State table aggregating token pair swap data per pool';

-- Materialized view to populate state_pools_tokens from swaps
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_pools_tokens
TO state_pools_tokens
AS
SELECT
    -- block --
    max(block_num) AS block_num,
    anyLast(block_hash) AS block_hash,
    max(timestamp) as timestamp,
    max(minute) as minute,

    -- transaction --
    anyLast(tx_hash) AS tx_hash,

    -- dex --
    pool,
    factory,
    protocol,

    -- state --
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
    -- block --
    max(block_num) AS block_num,
    anyLast(block_hash) AS block_hash,
    max(timestamp) as timestamp,
    max(minute) as minute,

    -- transaction --
    anyLast(tx_hash) AS tx_hash,

    -- dex --
    pool,
    factory,
    protocol,

    -- state --
    output_contract AS token
FROM swaps
GROUP BY
    pool,
    factory,
    protocol,
    token;
