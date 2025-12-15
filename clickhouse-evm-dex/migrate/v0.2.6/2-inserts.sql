-- SunPump TokenPurchased: User buys tokens with TRX (TRX → Token)
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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
    eth                                AS input_contract,  -- TRX native asset
    trx_amount                         AS input_amount,

    -- Output side: Tokens being purchased
    token                              AS output_contract,
    token_amount                       AS output_amount

FROM sunpump_token_purchased
WHERE input_amount > 0 AND output_amount > 0;

-- SunPump TokenSold: User sells tokens for TRX (Token → TRX)
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'sunpump' AS protocol,
    factory,
    log_address                        AS pool,
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
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
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
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
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
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
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

-- Uniswap V3 Swap
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap_v3' AS protocol,
    factory,
    log_address  AS pool,
    sender       AS user,

    -- Input side: negative amount means input
    if (amount0 < toString(toInt256(0)), token0, token1)      AS input_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: positive amount means output
    if (amount0 < toString(toInt256(0)), token1, token0)      AS output_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v3_swap
WHERE input_amount > 0 AND output_amount > 0;

-- Uniswap V4 Swap
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- swap --
    'uniswap_v4' AS protocol,
    factory,
    id           AS pool,
    sender       AS user,

    -- Input side: negative amount means input
    if (amount0 < toString(toInt256(0)), currency0, currency1)      AS input_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: positive amount means output
    if (amount0 < toString(toInt256(0)), currency1, currency0)      AS output_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v4_swap
WHERE input_amount > 0 AND output_amount > 0;


-- Curve.fi TokenExchange (Swap)
-- Note: Curve doesn't have a clear factory/token0/token1 structure like Uniswap
-- The sold_id and bought_id are indices that need to be mapped to actual token addresses
-- For now, we'll use the pool address as both pool and factory
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
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
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
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
INSERT INTO swaps (
    block_num,
    block_hash,
    timestamp,
    minute,
    tx_index,
    tx_hash,
    tx_from,
    log_index,
    log_address,
    log_ordinal,
    log_topic0,
    protocol,
    factory,
    pool,
    user,
    input_contract,
    input_amount,
    output_contract,
    output_amount
)
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

    -- log --
    log_index,
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
