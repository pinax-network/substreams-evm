-- Curve.fi TokenExchange (Swap) --
CREATE TABLE IF NOT EXISTS curvefi_token_exchange AS TEMPLATE_LOG
COMMENT 'Curve.fi TokenExchange (swap) events';
ALTER TABLE curvefi_token_exchange
    -- event information --
    ADD COLUMN IF NOT EXISTS buyer              String COMMENT 'Buyer wallet address',
    ADD COLUMN IF NOT EXISTS sold_id            String COMMENT 'ID of token sold',
    ADD COLUMN IF NOT EXISTS tokens_sold        String COMMENT 'Amount of tokens sold',
    ADD COLUMN IF NOT EXISTS bought_id          String COMMENT 'ID of token bought',
    ADD COLUMN IF NOT EXISTS tokens_bought      String COMMENT 'Amount of tokens bought',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_buyer (buyer) TYPE bloom_filter GRANULARITY 1;

-- Curve.fi AddLiquidity --
CREATE TABLE IF NOT EXISTS curvefi_add_liquidity AS TEMPLATE_LOG
COMMENT 'Curve.fi AddLiquidity events';
ALTER TABLE curvefi_add_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      Array(String) COMMENT 'Array of token amounts added',
    ADD COLUMN IF NOT EXISTS fees               Array(String) COMMENT 'Array of fees',
    ADD COLUMN IF NOT EXISTS invariant          String COMMENT 'Invariant value',
    ADD COLUMN IF NOT EXISTS token_supply       String COMMENT 'Total token supply',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_provider (provider) TYPE bloom_filter GRANULARITY 1;

-- Curve.fi RemoveLiquidity --
CREATE TABLE IF NOT EXISTS curvefi_remove_liquidity AS TEMPLATE_LOG
COMMENT 'Curve.fi RemoveLiquidity events';
ALTER TABLE curvefi_remove_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      Array(String) COMMENT 'Array of token amounts removed',
    ADD COLUMN IF NOT EXISTS fees               Array(String) COMMENT 'Array of fees',
    ADD COLUMN IF NOT EXISTS token_supply       String COMMENT 'Total token supply',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_provider (provider) TYPE bloom_filter GRANULARITY 1;

-- Curve.fi RemoveLiquidityOne --
CREATE TABLE IF NOT EXISTS curvefi_remove_liquidity_one AS TEMPLATE_LOG
COMMENT 'Curve.fi RemoveLiquidityOne events';
ALTER TABLE curvefi_remove_liquidity_one
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amount       String COMMENT 'Token amount',
    ADD COLUMN IF NOT EXISTS coin_amount        String COMMENT 'Coin amount',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_provider (provider) TYPE bloom_filter GRANULARITY 1;

-- Curve.fi RemoveLiquidityImbalance --
CREATE TABLE IF NOT EXISTS curvefi_remove_liquidity_imbalance AS TEMPLATE_LOG
COMMENT 'Curve.fi RemoveLiquidityImbalance events';
ALTER TABLE curvefi_remove_liquidity_imbalance
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      Array(String) COMMENT 'Array of token amounts',
    ADD COLUMN IF NOT EXISTS fees               Array(String) COMMENT 'Array of fees',
    ADD COLUMN IF NOT EXISTS invariant          String COMMENT 'Invariant value',
    ADD COLUMN IF NOT EXISTS token_supply       String COMMENT 'Total token supply',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_provider (provider) TYPE bloom_filter GRANULARITY 1;
