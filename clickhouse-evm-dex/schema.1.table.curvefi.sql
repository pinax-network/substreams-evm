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
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts added',
    ADD COLUMN IF NOT EXISTS fees               String COMMENT 'Comma-separated fees',
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
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts removed',
    ADD COLUMN IF NOT EXISTS fees               String COMMENT 'Comma-separated fees',
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
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts',
    ADD COLUMN IF NOT EXISTS fees               String COMMENT 'Comma-separated fees',
    ADD COLUMN IF NOT EXISTS invariant          String COMMENT 'Invariant value',
    ADD COLUMN IF NOT EXISTS token_supply       String COMMENT 'Total token supply',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_provider (provider) TYPE bloom_filter GRANULARITY 1;

-- Curve.fi Init --
CREATE TABLE IF NOT EXISTS curvefi_init AS TEMPLATE_LOG
COMMENT 'Curve.fi Init (pool creation) events';
ALTER TABLE curvefi_init
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Contract owner address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS pool_token         String COMMENT 'LP token address',
    ADD COLUMN IF NOT EXISTS a                  String COMMENT 'Amplification coefficient',
    ADD COLUMN IF NOT EXISTS fee                String COMMENT 'Exchange fee',
    ADD COLUMN IF NOT EXISTS admin_fee          String COMMENT 'Admin fee',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_owner (owner) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_pool_token (pool_token) TYPE bloom_filter GRANULARITY 1;
