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
    tx_from                     String,

    tx_to                       String,
    tx_nonce                    UInt64,
    tx_gas_price                UInt256,
    tx_gas_limit                UInt64,
    tx_gas_used                 UInt64,
    tx_value                    UInt256,

    -- call --
    call_caller                 String COMMENT 'Call-level caller address from shared log metadata',
    call_index                  UInt32 COMMENT 'Call index from shared log metadata',
    call_begin_ordinal          UInt64 COMMENT 'Call begin ordinal from shared log metadata',
    call_end_ordinal            UInt64 COMMENT 'Call end ordinal from shared log metadata',
    call_address                String COMMENT 'Call recipient address from shared log metadata',
    call_value                  UInt256 COMMENT 'Call value from shared log metadata',
    call_gas_consumed           UInt64 COMMENT 'Call gas consumed from shared log metadata',
    call_gas_limit              UInt64 COMMENT 'Call gas limit from shared log metadata',
    call_depth                  UInt32 COMMENT 'Call depth from shared log metadata',
    call_parent_index           UInt32 COMMENT 'Call parent index from shared log metadata',
    call_type                   LowCardinality(String) COMMENT 'Call type from shared log metadata',

    -- log --
    log_index                   UInt32, -- derived from Substreams
    log_block_index             UInt32 COMMENT 'BlockIndex represents the index of the log relative to the Block.',
    log_address                 String,
    log_ordinal                 UInt32 COMMENT 'The block global ordinal when the log was recorded.',
    log_topic0                  LowCardinality(String),

    -- swap event information --
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
    factory                     LowCardinality(String) COMMENT 'Factory contract address',
    pool                        String COMMENT 'Pool/exchange contract address',
    user                        String COMMENT 'User wallet address',
    input_contract              String COMMENT 'Input token contract address',
    input_amount                UInt256 COMMENT 'Amount of input tokens swapped',
    output_contract             String COMMENT 'Output token contract address',
    output_amount               UInt256 COMMENT 'Amount of output tokens received',

    -- contraints data validation --
    CONSTRAINT log_address_not_empty CHECK log_address != '',
    CONSTRAINT log_topic0_not_empty CHECK log_topic0 != '',
    CONSTRAINT tx_hash_not_empty CHECK tx_hash != '',
    CONSTRAINT tx_from_not_empty CHECK tx_from != '',
    CONSTRAINT factory_not_empty CHECK factory != '',
    CONSTRAINT pool_not_empty CHECK pool != '',
    CONSTRAINT user_not_empty CHECK user != '',
    CONSTRAINT input_contract_not_empty CHECK input_contract != '',
    CONSTRAINT output_contract_not_empty CHECK output_contract != '',
    CONSTRAINT input_amount_nonzero CHECK input_amount > 0,
    CONSTRAINT output_amount_nonzero CHECK output_amount > 0,

    -- materialized token pair (canonical ordering) --
    token0                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, input_contract, output_contract) COMMENT 'Lexicographically smaller token address',
    token1                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, output_contract, input_contract) COMMENT 'Lexicographically larger token address',
    amount0                     UInt256 MATERIALIZED if(input_contract <= output_contract, input_amount, output_amount) COMMENT 'Amount of token0 swapped',
    amount1                     UInt256 MATERIALIZED if(input_contract <= output_contract, output_amount, input_amount) COMMENT 'Amount of token1 swapped',

    -- INDEXES --
    INDEX idx_block_num         (block_num)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_minute            (minute)                    TYPE minmax             GRANULARITY 1,
    INDEX idx_amount0 (amount0) TYPE minmax,
    INDEX idx_amount1 (amount1) TYPE minmax,
    INDEX idx_input_amount (input_amount) TYPE minmax,
    INDEX idx_output_amount (output_amount) TYPE minmax,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_protocol_count ( SELECT protocol, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY protocol ),
    PROJECTION prj_factory_count ( SELECT factory, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY factory ),
    PROJECTION prj_pool_count ( SELECT pool, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY pool ),
    PROJECTION prj_tx_from_count ( SELECT tx_from, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY tx_from ),
    PROJECTION prj_call_caller_count ( SELECT call_caller, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY call_caller ),
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
    PROJECTION prj_log_topic0_by_minute ( SELECT log_topic0, minute GROUP BY log_topic0, minute ),
    PROJECTION prj_call_type_by_minute ( SELECT call_type, minute GROUP BY call_type, minute ),
    PROJECTION prj_protocol_by_minute ( SELECT protocol, minute, count() GROUP BY protocol, minute ),
    PROJECTION prj_tx_from_by_minute ( SELECT tx_from, minute, count() GROUP BY tx_from, minute ),
    PROJECTION prj_call_caller_by_minute ( SELECT call_caller, minute, count() GROUP BY call_caller, minute ),
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
COMMENT 'DEX swap events normalized across supported protocols';

-- Flash/complex swaps that cannot be losslessly normalized to a single input/output pair --
CREATE TABLE IF NOT EXISTS swaps_flash (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    UInt32,
    tx_hash                     String,
    tx_from                     String,
    tx_to                       String,
    tx_nonce                    UInt64,
    tx_gas_price                UInt256,
    tx_gas_limit                UInt64,
    tx_gas_used                 UInt64,
    tx_value                    UInt256,

    -- call --
    call_caller                 String COMMENT 'Call-level caller address from shared log metadata',
    call_index                  UInt32 COMMENT 'Call index from shared log metadata',
    call_begin_ordinal          UInt64 COMMENT 'Call begin ordinal from shared log metadata',
    call_end_ordinal            UInt64 COMMENT 'Call end ordinal from shared log metadata',
    call_address                String COMMENT 'Call recipient address from shared log metadata',
    call_value                  UInt256 COMMENT 'Call value from shared log metadata',
    call_gas_consumed           UInt64 COMMENT 'Call gas consumed from shared log metadata',
    call_gas_limit              UInt64 COMMENT 'Call gas limit from shared log metadata',
    call_depth                  UInt32 COMMENT 'Call depth from shared log metadata',
    call_parent_index           UInt32 COMMENT 'Call parent index from shared log metadata',
    call_type                   LowCardinality(String) COMMENT 'Call type from shared log metadata',

    -- log --
    log_index                   UInt32,
    log_block_index             UInt32 COMMENT 'BlockIndex represents the index of the log relative to the Block.',
    log_address                 String,
    log_ordinal                 UInt32 COMMENT 'The block global ordinal when the log was recorded.',
    log_topic0                  LowCardinality(String),

    -- swap event information --
    protocol                    LowCardinality(String) COMMENT 'flash/complex swap protocol identifier',
    factory                     LowCardinality(String) COMMENT 'Factory contract address',
    pool                        String COMMENT 'Pool/exchange contract address',
    user                        String COMMENT 'User wallet address',

    -- raw swap legs --
    token0                      String COMMENT 'Token0 contract address',
    token1                      String COMMENT 'Token1 contract address',
    amount0_in                  UInt256 COMMENT 'Amount of token0 in',
    amount1_in                  UInt256 COMMENT 'Amount of token1 in',
    amount0_out                 UInt256 COMMENT 'Amount of token0 out',
    amount1_out                 UInt256 COMMENT 'Amount of token1 out',

    -- indexes --
    INDEX idx_block_num         (block_num)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_minute            (minute)                    TYPE minmax             GRANULARITY 1
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
)
COMMENT 'Flash/complex DEX swap events kept out of the normalized swaps table';


-- SunPump TokenPurchased: User buys tokens with TRX (TRX → Token)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_sunpump_token_purchased
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'sunpump' AS protocol,
    factory,
    token                              AS pool,
    buyer                              AS user,

    -- Input side: TRX being paid
    eth                                AS input_contract,  -- TRX native asset
    trx_amount                         AS input_amount,

    -- Output side: Tokens being purchased
    token                              AS output_contract,
    token_amount                       AS output_amount

FROM sunpump_token_purchased
WHERE input_amount > 0 AND output_amount > 0;

-- SunPump TokenSold: User sells tokens for TRX (Token → TRX)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_sunpump_token_sold
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'sunpump' AS protocol,
    factory,
    token                              AS pool,
    seller                             AS user,

    -- Input side: Tokens being sold
    token                              AS input_contract,
    token_amount                       AS input_amount,

    -- Output side: TRX being received
    eth                                AS output_contract,  -- TRX native asset
    trx_amount                         AS output_amount

FROM sunpump_token_sold
WHERE input_amount > 0 AND output_amount > 0;

-- Uniswap V1 TokenPurchase: User buys tokens with ETH (ETH → Token)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_uniswap_v1_token_purchase
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap_v1' AS protocol,
    factory,
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: ETH being sold
    eth  AS input_contract,  -- ETH native asset
    eth_sold                           AS input_amount,

    -- Output side: Tokens being bought
    token                              AS output_contract,
    tokens_bought                      AS output_amount

FROM uniswap_v1_token_purchase
WHERE input_amount > 0 AND output_amount > 0;


-- Uniswap V1 EthPurchase: User buys ETH with tokens (Token → ETH)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_uniswap_v1_eth_purchase
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap_v1' AS protocol,
    factory,
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: Tokens being sold
    token                              AS input_contract,
    tokens_sold                        AS input_amount,

    -- Output side: ETH being bought
    eth                                AS output_contract,  -- ETH native asset
    eth_bought                         AS output_amount

FROM uniswap_v1_eth_purchase
WHERE input_amount > 0 AND output_amount > 0;


-- Uniswap V2 Swap (not Flash Swaps)
-- https://github.com/pinax-network/substreams-evm/issues/66
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_uniswap_v2_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap_v2' AS protocol,
    factory,
    log_address  AS pool,
    sender       AS user,

    -- Input side (only one of amount0_in / amount1_in is > 0 here)
    if (amount0_in > toUInt256(0), token0, token1)      AS input_contract,
    if (amount0_in > toUInt256(0), amount0_in, amount1_in) AS input_amount,

    -- Output side (only one of amount0_out / amount1_out is > 0 here)
    if (amount0_in > toUInt256(0), token1, token0)      AS output_contract,
    if (amount0_in > toUInt256(0), amount1_out, amount0_out) AS output_amount

FROM uniswap_v2_swap
WHERE
    (
        -- token0 -> token1
        (
            amount0_in  >  toUInt256(0) AND
            amount1_in  =  toUInt256(0) AND
            amount0_out =  toUInt256(0) AND
            amount1_out >  toUInt256(0)
        )
        OR
        -- token1 -> token0
        (
            amount1_in  >  toUInt256(0) AND
            amount0_in  =  toUInt256(0) AND
            amount1_out =  toUInt256(0) AND
            amount0_out >  toUInt256(0)
        )
    ) AND
    input_amount > 0 AND output_amount > 0;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_flash_uniswap_v2_swap
TO swaps_flash AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap-v2-flash' AS protocol,
    factory,
    log_address AS pool,
    sender AS user,
    token0,
    token1,
    amount0_in,
    amount1_in,
    amount0_out,
    amount1_out

FROM uniswap_v2_swap
WHERE
    (
        amount0_in  > toUInt256(0) OR
        amount1_in  > toUInt256(0) OR
        amount0_out > toUInt256(0) OR
        amount1_out > toUInt256(0)
    )
    AND NOT
    (
        -- simple token0 -> token1
        (
            amount0_in  > toUInt256(0) AND
            amount1_in  = toUInt256(0) AND
            amount0_out = toUInt256(0) AND
            amount1_out > toUInt256(0)
        )
        OR
        -- simple token1 -> token0
        (
            amount1_in  > toUInt256(0) AND
            amount0_in  = toUInt256(0) AND
            amount1_out = toUInt256(0) AND
            amount0_out > toUInt256(0)
        )
    );

-- Uniswap V3 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_uniswap_v3_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap_v3' AS protocol,
    factory,
    log_address  AS pool,
    sender       AS user,

    -- Input side: positive amount0 means token0 entered the pool (trader pays token0)
    if (toInt256(amount0) > toInt256(0), token0, token1)      AS input_contract,
    if (toInt256(amount0) > toInt256(0), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: the token with a negative amount left the pool (trader receives it); amount0>0 → token1 out, amount0<0 → token0 out
    if (toInt256(amount0) > toInt256(0), token1, token0)      AS output_contract,
    if (toInt256(amount0) > toInt256(0), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v3_swap
WHERE input_amount > 0 AND output_amount > 0;

-- Uniswap V4 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_uniswap_v4_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap_v4' AS protocol,
    factory,
    id           AS pool,
    sender       AS user,

    -- Input side: negative amount means input
    if (toInt256(amount0) < toInt256(0), currency0, currency1)      AS input_contract,
    if (toInt256(amount0) < toInt256(0), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: positive amount means output
    if (toInt256(amount0) < toInt256(0), currency1, currency0)      AS output_contract,
    if (toInt256(amount0) < toInt256(0), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v4_swap
WHERE input_amount > 0 AND output_amount > 0;


-- Curve.fi TokenExchange (Swap)
-- Note: Curve doesn't have a clear factory/token0/token1 structure like Uniswap
-- The sold_id and bought_id are indices that need to be mapped to actual token addresses
-- For now, we'll use the pool address as both pool and factory
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_curvefi_token_exchange
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'curvefi' AS protocol,
    factory,
    log_address                        AS pool,
    buyer                              AS user,

    -- Note: sold_id and bought_id are token indices, not addresses
    -- In a full implementation, these would be resolved via store lookups
    -- For now, we use the indices as placeholders
    sold_token  AS input_contract,
    sold_amount AS input_amount,

    bought_token AS output_contract,
    bought_amount AS output_amount
FROM curvefi_token_exchange
WHERE input_amount > 0 AND output_amount > 0;

-- Balancer V3 Vault Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_balancer_vault_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'balancer' AS protocol,
    factory,
    pool       AS pool,
    tx_from    AS user,  -- Using tx_from as the user since there's no explicit user in the event

    -- Input side
    token_in                           AS input_contract,
    amount_in                          AS input_amount,

    -- Output side
    token_out                          AS output_contract,
    amount_out                         AS output_amount

FROM balancer_vault_swap
WHERE input_amount > 0 AND output_amount > 0;

-- Bancor Conversion (Swap)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_bancor_conversion
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'bancor' AS protocol,
    factory,
    log_address                        AS pool,
    trader                             AS user,

    -- Input side
    source_token                       AS input_contract,
    source_amount                      AS input_amount,

    -- Output side
    target_token                       AS output_contract,
    target_amount                      AS output_amount

FROM bancor_conversion
WHERE input_amount > 0 AND output_amount > 0;

-- CoW Protocol Trade (Swap)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_cow_trade
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'cow' AS protocol,
    log_address                        AS factory,
    log_address                        AS pool,
    owner                              AS user,

    -- Input side
    sell_token                         AS input_contract,
    sell_amount                        AS input_amount,

    -- Output side
    buy_token                          AS output_contract,
    buy_amount                         AS output_amount

FROM cow_trade
WHERE input_amount > 0 AND output_amount > 0;

-- Aerodrome Swap (Solidly fork, like Uniswap V2)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_aerodrome_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'aerodrome' AS protocol,
    factory,
    log_address  AS pool,
    sender       AS user,

    -- Input side (only one of amount0_in / amount1_in is > 0 here)
    if (amount0_in > toUInt256(0), token0, token1)      AS input_contract,
    if (amount0_in > toUInt256(0), amount0_in, amount1_in) AS input_amount,

    -- Output side (only one of amount0_out / amount1_out is > 0 here)
    if (amount0_in > toUInt256(0), token1, token0)      AS output_contract,
    if (amount0_in > toUInt256(0), amount1_out, amount0_out) AS output_amount

FROM aerodrome_swap
WHERE
    -- token0 -> token1
    (
        amount0_in  >  toUInt256(0) AND
        amount1_in  =  toUInt256(0) AND
        amount0_out =  toUInt256(0) AND
        amount1_out >  toUInt256(0)
    )
    OR
    -- token1 -> token0
    (
        amount1_in  >  toUInt256(0) AND
        amount0_in  =  toUInt256(0) AND
        amount1_out =  toUInt256(0) AND
        amount0_out >  toUInt256(0)
    ) AND
    input_amount > 0 AND output_amount > 0;

-- DODO OrderHistory (Swap)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_dodo_order_history
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'dodo' AS protocol,
    log_address                        AS factory,
    log_address                        AS pool,
    sender                             AS user,

    -- Input side
    from_token                         AS input_contract,
    from_amount                        AS input_amount,

    -- Output side
    to_token                           AS output_contract,
    return_amount                      AS output_amount

FROM dodo_order_history
WHERE input_amount > 0 AND output_amount > 0;

-- WOOFi WooSwap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_woofi_woo_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'woofi' AS protocol,
    log_address                        AS factory,
    log_address                        AS pool,
    `from`                             AS user,

    -- Input side
    from_token                         AS input_contract,
    from_amount                        AS input_amount,

    -- Output side
    to_token                           AS output_contract,
    to_amount                          AS output_amount

FROM woofi_woo_swap
WHERE input_amount > 0 AND output_amount > 0;

-- Trader Joe V2 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_traderjoe_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'traderjoe' AS protocol,
    factory,
    log_address  AS pool,
    sender       AS user,

    -- Input side
    if (amount_in_x > toUInt256(0), token0, token1)         AS input_contract,
    if (amount_in_x > toUInt256(0), amount_in_x, amount_in_y) AS input_amount,

    -- Output side
    if (amount_in_x > toUInt256(0), token1, token0)         AS output_contract,
    if (amount_in_x > toUInt256(0), amount_out_y, amount_out_x) AS output_amount

FROM traderjoe_swap
WHERE
    -- Only one direction: either X->Y or Y->X, not both
    NOT (amount_in_x > toUInt256(0) AND amount_in_y > toUInt256(0))
    AND input_amount > 0 AND output_amount > 0;

-- KyberSwap Elastic Swap (signed amounts like Uniswap V3)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_swaps_kyber_elastic_swap
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
    tx_from,
    tx_to,
    tx_nonce,
    tx_gas_price,
    tx_gas_limit,
    tx_gas_used,
    tx_value,

    -- call --
    call_caller,
    call_index,
    call_begin_ordinal,
    call_end_ordinal,
    call_address,
    call_value,
    call_gas_consumed,
    call_gas_limit,
    call_depth,
    call_parent_index,
    call_type,

    -- log --
    log_index,
    log_block_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'kyber_elastic' AS protocol,
    factory,
    log_address  AS pool,
    sender       AS user,

    -- Input side: negative amount means input
    if (toInt256(delta_qty0) < toInt256(0), token0, token1)      AS input_contract,
    if (toInt256(delta_qty0) < toInt256(0), abs(toInt256(delta_qty0)), abs(toInt256(delta_qty1))) AS input_amount,

    -- Output side: positive amount means output
    if (toInt256(delta_qty0) < toInt256(0), token1, token0)      AS output_contract,
    if (toInt256(delta_qty0) < toInt256(0), abs(toInt256(delta_qty1)), abs(toInt256(delta_qty0))) AS output_amount

FROM kyber_elastic_swap
WHERE input_amount > 0 AND output_amount > 0;
