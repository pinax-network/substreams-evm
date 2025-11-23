-- Swaps --
CREATE TABLE IF NOT EXISTS swaps AS TEMPLATE_LOG
COMMENT 'Swaps';
ALTER TABLE swaps
    -- swap event information --
    ADD COLUMN IF NOT EXISTS protocol           LowCardinality(String) COMMENT 'DEX protocol name',
    ADD COLUMN IF NOT EXISTS factory            LowCardinality(String) COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS pool               LowCardinality(String) COMMENT 'Pool/exchange contract address',
    ADD COLUMN IF NOT EXISTS user               String COMMENT 'User wallet address',
    ADD COLUMN IF NOT EXISTS input_contract     LowCardinality(String) COMMENT 'Input token contract address',
    ADD COLUMN IF NOT EXISTS input_amount       UInt256 COMMENT 'Amount of input tokens swapped',
    ADD COLUMN IF NOT EXISTS output_contract    LowCardinality(String) COMMENT 'Output token contract address',
    ADD COLUMN IF NOT EXISTS output_amount      UInt256 COMMENT 'Amount of output tokens received',

   -- materialized token pair (canonical ordering) --
    ADD COLUMN IF NOT EXISTS token0             LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, input_contract, output_contract) COMMENT 'Lexicographically smaller token address',
    ADD COLUMN IF NOT EXISTS token1             LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, output_contract, input_contract) COMMENT 'Lexicographically larger token address',

    -- PROJECTIONS --
    -- count() --
    ADD PROJECTION IF NOT EXISTS prj_protocol_count ( SELECT protocol, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY protocol ),
    ADD PROJECTION IF NOT EXISTS prj_factory_count ( SELECT factory, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY factory ),
    ADD PROJECTION IF NOT EXISTS prj_pool_count ( SELECT pool, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY pool ),
    ADD PROJECTION IF NOT EXISTS prj_user_count ( SELECT user, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY user ),
    ADD PROJECTION IF NOT EXISTS prj_input_contract_count ( SELECT input_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY input_contract ),
    ADD PROJECTION IF NOT EXISTS prj_output_contract_count ( SELECT output_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY output_contract ),
    ADD PROJECTION IF NOT EXISTS prj_token0_count ( SELECT token0, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY token0 ),
    ADD PROJECTION IF NOT EXISTS prj_token1_count ( SELECT token1, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY token1 ),

    -- used for `/pools` endpoint --
    ADD PROJECTION IF NOT EXISTS prj_all_count (
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

    -- minute --
    ADD PROJECTION IF NOT EXISTS prj_protocol_by_minute ( SELECT protocol, minute, count() GROUP BY protocol, minute ),
    ADD PROJECTION IF NOT EXISTS prj_factory_by_minute ( SELECT factory, minute, count() GROUP BY factory, minute ),
    ADD PROJECTION IF NOT EXISTS prj_pool_by_minute ( SELECT pool, minute, count() GROUP BY pool, minute ),
    ADD PROJECTION IF NOT EXISTS prj_user_by_minute ( SELECT user, minute, count() GROUP BY user, minute ),
    ADD PROJECTION IF NOT EXISTS prj_input_contract_by_minute ( SELECT input_contract, minute, count() GROUP BY input_contract, minute ),
    ADD PROJECTION IF NOT EXISTS prj_output_contract_by_minute ( SELECT output_contract, minute, count() GROUP BY output_contract, minute ),
    ADD PROJECTION IF NOT EXISTS prj_token0_by_minute ( SELECT token0, minute, count() GROUP BY token0, minute ),
    ADD PROJECTION IF NOT EXISTS prj_token1_by_minute ( SELECT token1, minute, count() GROUP BY token1, minute );

-- SunPump TokenPurchased: User buys tokens with TRX (TRX → Token)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_purchased
TO swaps AS
SELECT
    'sunpump' AS protocol,
    -- include everything from sunpump_token_purchased except the non-relevant fields
    * EXCEPT (
        buyer,
        trx_amount,
        token,
        token_amount,
        fee,
        token_reserve,
        creator,
        token_index
    ),

    -- mapped swap fields
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: TRX being paid
    'T0000000000000000000000000000000000000001'                                 AS input_contract,  -- TRX native asset
    trx_amount                         AS input_amount,

    -- Output side: Tokens being purchased
    token                              AS output_contract,
    token_amount                       AS output_amount

FROM sunpump_token_purchased
WHERE factory != '';  -- exclude invalid events with empty factory address


-- SunPump TokenSold: User sells tokens for TRX (Token → TRX)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_sunpump_token_sold
TO swaps AS
SELECT
    'sunpump' AS protocol,

    -- include everything from sunpump_token_sold except the non-relevant fields
    * EXCEPT (
        seller,
        token,
        token_amount,
        trx_amount,
        fee,
        creator,
        token_index
    ),

    -- mapped swap fields
    log_address                        AS pool,
    seller                             AS user,

    -- Input side: Tokens being sold
    token                              AS input_contract,
    token_amount                       AS input_amount,

    -- Output side: TRX being received
    'T0000000000000000000000000000000000000001'                                 AS output_contract,  -- TRX native asset
    trx_amount                         AS output_amount

FROM sunpump_token_sold
WHERE factory != '';  -- exclude invalid events with empty factory address

-- Uniswap V1 TokenPurchase: User buys tokens with ETH (ETH → Token)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v1_token_purchase
TO swaps AS
SELECT
    'uniswap-v1' AS protocol,

    -- include everything from uniswap_v1_token_purchase except the non-relevant fields
    * EXCEPT (
        buyer,
        eth_sold,
        tokens_bought,
        token
    ),

    -- mapped swap fields
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: ETH being sold
    '0x0000000000000000000000000000000000000000'  AS input_contract,  -- ETH native asset
    eth_sold                           AS input_amount,

    -- Output side: Tokens being bought
    token                              AS output_contract,
    tokens_bought                      AS output_amount

FROM uniswap_v1_token_purchase
WHERE factory != '';  -- exclude invalid events with empty factory address


-- Uniswap V1 EthPurchase: User buys ETH with tokens (Token → ETH)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v1_eth_purchase
TO swaps AS
SELECT
    'uniswap-v1' AS protocol,

    -- include everything from uniswap_v1_eth_purchase except the non-relevant fields
    * EXCEPT (
        buyer,
        tokens_sold,
        eth_bought,
        token
    ),

    -- mapped swap fields
    log_address                        AS pool,
    buyer                              AS user,

    -- Input side: Tokens being sold
    token                              AS input_contract,
    tokens_sold                        AS input_amount,

    -- Output side: ETH being bought
    '0x0000000000000000000000000000000000000000'  AS output_contract,  -- ETH native asset
    eth_bought                         AS output_amount

FROM uniswap_v1_eth_purchase
WHERE factory != '';  -- exclude invalid events with empty factory address


-- Uniswap V2 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v2_swap
TO swaps AS
SELECT
    'uniswap-v2' AS protocol,

    -- include everything from uniswap_v2_swap except the non-relevant fields
    * EXCEPT (
        sender,
        `to`,
        amount0_in,
        amount1_in,
        amount0_out,
        amount1_out,
        token0,
        token1
    ),

    -- mapped swap fields
    log_address                        AS pool,
    sender                             AS user,

    -- Input side
    if (amount0_in > toUInt256(0), token0, token1)      AS input_contract,
    if (amount0_in > toUInt256(0), amount0_in, amount1_in) AS input_amount,

    -- Output side
    if (amount0_in > toUInt256(0), token1, token0)      AS output_contract,
    if (amount0_in > toUInt256(0), amount1_out, amount0_out) AS output_amount

FROM uniswap_v2_swap
WHERE factory != '';  -- exclude invalid events with empty factory address


-- Uniswap V3 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v3_swap
TO swaps AS
SELECT
    'uniswap-v3' AS protocol,

    -- include everything from uniswap_v3_swap except the non-relevant fields
    * EXCEPT (
        sender,
        recipient,
        amount0,
        amount1,
        sqrt_price_x96,
        liquidity,
        tick,
        token0,
        token1,
        fee,
        tick_spacing
    ),

    -- mapped swap fields
    log_address                        AS pool,
    sender                             AS user,

    -- Input side: negative amount means input
    if (amount0 < toString(toInt256(0)), token0, token1)      AS input_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: positive amount means output
    if (amount0 < toString(toInt256(0)), token1, token0)      AS output_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v3_swap
WHERE factory != '';  -- exclude invalid events with empty factory address


-- Uniswap V4 Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_uniswap_v4_swap
TO swaps AS
SELECT
    'uniswap-v4' AS protocol,

    -- include everything from uniswap_v4_swap except the non-relevant fields
    * EXCEPT (
        id,
        sender,
        amount0,
        amount1,
        sqrt_price_x96,
        liquidity,
        tick,
        fee,
        currency0,
        currency1,
        tick_spacing
    ),

    -- mapped swap fields
    log_address                        AS pool,
    sender                             AS user,

    -- Input side: negative amount means input
    if (amount0 < toString(toInt256(0)), currency0, currency1)      AS input_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount0)), abs(toInt256(amount1))) AS input_amount,

    -- Output side: positive amount means output
    if (amount0 < toString(toInt256(0)), currency1, currency0)      AS output_contract,
    if (amount0 < toString(toInt256(0)), abs(toInt256(amount1)), abs(toInt256(amount0))) AS output_amount

FROM uniswap_v4_swap
WHERE factory != '';  -- exclude invalid events with empty factory address


-- Curve.fi TokenExchange (Swap)
-- Note: Curve doesn't have a clear factory/token0/token1 structure like Uniswap
-- The sold_id and bought_id are indices that need to be mapped to actual token addresses
-- For now, we'll use the pool address as both pool and factory
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_curvefi_token_exchange
TO swaps AS
SELECT
    'curvefi' AS protocol,

    -- include everything from curvefi_token_exchange except the non-relevant fields
    * EXCEPT (
        buyer,
        sold_id,
        sold_amount,
        sold_token,
        bought_id,
        bought_amount,
        bought_token,
        coins,
        deployer
    ),

    -- mapped swap fields
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
WHERE factory != '';  -- exclude invalid events with empty factory address


-- Balancer V3 Vault Swap
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_balancer_vault_swap
TO swaps AS
SELECT
    'balancer' AS protocol,

    -- include everything from balancer_vault_swap except the non-relevant fields
    * EXCEPT (
        pool,
        token_in,
        token_out,
        amount_in,
        amount_out,
        swap_fee_percentage,
        swap_fee_amount
    ),

    -- mapped swap fields
    pool                               AS pool,
    tx_from                            AS user,  -- Using tx_from as the user since there's no explicit user in the event

    -- Input side
    token_in                           AS input_contract,
    amount_in                          AS input_amount,

    -- Output side
    token_out                          AS output_contract,
    amount_out                         AS output_amount

FROM balancer_vault_swap
WHERE factory != '';  -- exclude invalid events with empty factory address


-- Bancor Conversion (Swap)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_bancor_conversion
TO swaps AS
SELECT
    'bancor' AS protocol,

    -- include everything from bancor_conversion except the non-relevant fields
    * EXCEPT (
        source_token,
        target_token,
        trader,
        source_amount,
        target_amount,
        conversion_fee,
        converter_type
    ),

    -- mapped swap fields
    log_address                        AS pool,
    trader                             AS user,

    -- Input side
    source_token                       AS input_contract,
    source_amount                      AS input_amount,

    -- Output side
    target_token                       AS output_contract,
    target_amount                      AS output_amount

FROM bancor_conversion
WHERE factory != '';  -- exclude invalid events with empty factory address


-- CoW Protocol Trade (Swap)
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_cow_trade
TO swaps AS
SELECT
    'cow' AS protocol,
    '' AS factory,  -- CoW doesn't have a factory field

    -- include everything from cow_trade except the non-relevant fields
    * EXCEPT (
        owner,
        sell_token,
        buy_token,
        sell_amount,
        buy_amount,
        fee_amount,
        order_uid
    ),

    -- mapped swap fields
    log_address                        AS pool,     -- Settlement contract address
    owner                              AS user,

    -- Input side
    sell_token                         AS input_contract,
    sell_amount                        AS input_amount,

    -- Output side
    buy_token                          AS output_contract,
    buy_amount                         AS output_amount

FROM cow_trade;
