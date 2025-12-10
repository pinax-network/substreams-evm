-- Swaps table to store DEX swap events from various protocols --
CREATE TABLE IF NOT EXISTS swaps (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    UInt32, -- derived from Substreams
    tx_hash                     String,

    -- log --
    log_index                   Nullable(UInt32), -- derived from Substreams
    log_address                 LowCardinality(String),
    log_ordinal                 Nullable(UInt32),
    log_topic0                  LowCardinality(String),

    -- swap event information --
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
    factory                     LowCardinality(String) COMMENT 'Factory contract address',
    pool                        LowCardinality(String) COMMENT 'Pool/exchange contract address',
    user                        String COMMENT 'User wallet address',
    input_contract              LowCardinality(String) COMMENT 'Input token contract address',
    input_amount                UInt256 COMMENT 'Amount of input tokens swapped',
    output_contract             LowCardinality(String) COMMENT 'Output token contract address',
    output_amount               UInt256 COMMENT 'Amount of output tokens received',

    -- materialized token pair (canonical ordering) --
    token0                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, input_contract, output_contract) COMMENT 'Lexicographically smaller token address',
    token1                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, output_contract, input_contract) COMMENT 'Lexicographically larger token address',
    amount0                     UInt256 MATERIALIZED if(input_contract <= output_contract, input_amount, output_amount) COMMENT 'Amount of token0 swapped',
    amount1                     UInt256 MATERIALIZED if(input_contract <= output_contract, output_amount, input_amount) COMMENT 'Amount of token1 swapped',

    -- INDEXES --
    INDEX idx_amount0 (amount0) TYPE minmax,
    INDEX idx_amount1 (amount1) TYPE minmax,
    INDEX idx_input_amount (input_amount) TYPE minmax,
    INDEX idx_output_amount (output_amount) TYPE minmax,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_protocol_count ( SELECT protocol, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY protocol ),
    PROJECTION prj_factory_count ( SELECT factory, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY factory ),
    PROJECTION prj_pool_count ( SELECT pool, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY pool ),
    PROJECTION prj_user_count ( SELECT user, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY user ),
    PROJECTION prj_input_contract_count ( SELECT input_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY input_contract ),
    PROJECTION prj_output_contract_count ( SELECT output_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY output_contract ),
    PROJECTION prj_token0_count ( SELECT token0, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY token0 ),
    PROJECTION prj_token1_count ( SELECT token1, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY token1 ),

    -- used for `/pools` endpoint --
    PROJECTION prj_all_count (
        SELECT
            protocol,
            factory,
            pool,
            count(),
            min(block_num),
            max(block_num),
            min(timestamp),
            max(timestamp),
            min(minute),
            max(minute),
            token0,
            token1
        GROUP BY protocol, factory, pool, token0, token1
    ),

    -- minute + timestamp --
    PROJECTION prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp GROUP BY tx_hash, minute, timestamp ),

    -- minute --
    PROJECTION prj_log_address_by_minute ( SELECT log_address, minute GROUP BY log_address, minute ),

    -- minute --
    PROJECTION prj_all_by_minute ( SELECT protocol, factory, pool, input_contract, output_contract, minute, count() GROUP BY protocol, factory, pool, input_contract, output_contract, minute ),
    PROJECTION prj_protocol_by_minute ( SELECT protocol, minute, count() GROUP BY protocol, minute ),
    PROJECTION prj_factory_by_minute ( SELECT factory, minute, count() GROUP BY factory, minute ),
    PROJECTION prj_pool_by_minute ( SELECT pool, minute, count() GROUP BY pool, minute ),
    PROJECTION prj_user_by_minute ( SELECT user, minute, count() GROUP BY user, minute ),
    PROJECTION prj_input_contract_by_minute ( SELECT input_contract, minute, count() GROUP BY input_contract, minute ),
    PROJECTION prj_output_contract_by_minute ( SELECT output_contract, minute, count() GROUP BY output_contract, minute ),
    PROJECTION prj_token0_by_minute ( SELECT token0, minute, count() GROUP BY token0, minute ),
    PROJECTION prj_token1_by_minute ( SELECT token1, minute, count() GROUP BY token1, minute )
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
)
COMMENT 'Transfers including ERC-20, WETH transfers';


-- SunPump TokenPurchased: User buys tokens with TRX (TRX → Token)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_purchased
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'sunpump' AS protocol,
    factory,
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: TRX being paid
    ''                                 AS input_contract,  -- TRX native asset
    trx_amount                         AS input_amount,

    -- Output side: Tokens being purchased
    token                              AS output_contract,
    token_amount                       AS output_amount

FROM sunpump_token_purchased;

-- SunPump TokenSold: User sells tokens for TRX (Token → TRX)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_sold
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'sunpump' AS protocol,
    log_address                        AS pool,
    seller                             AS user,

    -- Input side: Tokens being sold
    token                              AS input_contract,
    token_amount                       AS input_amount,

    -- Output side: TRX being received
    ''                                 AS output_contract,  -- TRX native asset
    trx_amount                         AS output_amount

FROM sunpump_token_sold;

-- Uniswap V1 TokenPurchase: User buys tokens with ETH (ETH → Token)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v1_token_purchase
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap-v1' AS protocol,
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: ETH being sold
    ''  AS input_contract,  -- ETH native asset
    eth_sold                           AS input_amount,

    -- Output side: Tokens being bought
    token                              AS output_contract,
    tokens_bought                      AS output_amount

FROM uniswap_v1_token_purchase;


-- Uniswap V1 EthPurchase: User buys ETH with tokens (Token → ETH)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v1_eth_purchase
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap-v1' AS protocol,
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: Tokens being sold
    token                              AS input_contract,
    tokens_sold                        AS input_amount,

    -- Output side: ETH being bought
    ''                                 AS output_contract,  -- ETH native asset
    eth_bought                         AS output_amount

FROM uniswap_v1_eth_purchase;


-- Uniswap V2 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v2_swap
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap-v2' AS protocol,
    log_address  AS pool,
    sender       AS user,

    -- Input side
    if (amount0_in > toUInt256(0), token0, token1)      AS input_contract,
    if (amount0_in > toUInt256(0), amount0_in, amount1_in) AS input_amount,

    -- Output side
    if (amount0_in > toUInt256(0), token1, token0)      AS output_contract,
    if (amount0_in > toUInt256(0), amount1_out, amount0_out) AS output_amount

FROM uniswap_v2_swap;


-- Uniswap V3 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v3_swap
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap-v3' AS protocol,
    log_address  AS pool,
    sender       AS user,

    -- Input side: negative amount means input
    if (amount0 < toString(toInt256(0)), token0, token1)      AS input_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: positive amount means output
    if (amount0 < toString(toInt256(0)), token1, token0)      AS output_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v3_swap;


-- Uniswap V4 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v4_swap
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap-v4' AS protocol,
    id           AS pool,
    sender       AS user,

    -- Input side: negative amount means input
    if (amount0 < toString(toInt256(0)), currency0, currency1)      AS input_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: positive amount means output
    if (amount0 < toString(toInt256(0)), currency1, currency0)      AS output_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v4_swap;


-- Curve.fi TokenExchange (Swap)
-- Note: Curve doesn't have a clear factory/token0/token1 structure like Uniswap
-- The sold_id and bought_id are indices that need to be mapped to actual token addresses
-- For now, we'll use the pool address as both pool and factory
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_token_exchange
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'curvefi' AS protocol,
    log_address                        AS pool,
    buyer                              AS user,

    -- Note: sold_id and bought_id are token indices, not addresses
    -- In a full implementation, these would be resolved via store lookups
    -- For now, we use the indices as placeholders
    sold_token  AS input_contract,
    sold_amount AS input_amount,

    bought_token AS output_contract,
    bought_amount AS output_amount
FROM curvefi_token_exchange;

-- Balancer V3 Vault Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_vault_swap
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'balancer' AS protocol,
    pool       AS pool,
    tx_from    AS user,  -- Using tx_from as the user since there's no explicit user in the event

    -- Input side
    token_in                           AS input_contract,
    amount_in                          AS input_amount,

    -- Output side
    token_out                          AS output_contract,
    amount_out                         AS output_amount

FROM balancer_vault_swap;

-- Bancor Conversion (Swap)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_bancor_conversion
TO swaps AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'bancor' AS protocol,
    log_address                        AS pool,
    trader                             AS user,

    -- Input side
    source_token                       AS input_contract,
    source_amount                      AS input_amount,

    -- Output side
    target_token                       AS output_contract,
    target_amount                      AS output_amount

FROM bancor_conversion;
