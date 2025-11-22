-- Bancor Conversion (Swap) --
CREATE TABLE IF NOT EXISTS bancor_conversion AS TEMPLATE_LOG
COMMENT 'Bancor Conversion (swap) events';

ALTER TABLE bancor_conversion
    -- event information --
    ADD COLUMN IF NOT EXISTS source_token       String COMMENT 'Source token contract address',
    ADD COLUMN IF NOT EXISTS target_token       String COMMENT 'Target token contract address',
    ADD COLUMN IF NOT EXISTS trader             String COMMENT 'Trader wallet address',
    ADD COLUMN IF NOT EXISTS source_amount      String COMMENT 'Amount of source tokens',
    ADD COLUMN IF NOT EXISTS target_amount      String COMMENT 'Amount of target tokens',
    ADD COLUMN IF NOT EXISTS conversion_fee     String COMMENT 'Conversion fee',

    -- Activation (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS converter_type     Enum8(
        'Unknown'       = 0,
        'LiquidToken'   = 1,
        'LiquidityPool' = 2,
        'FeeConverter'  = 3,
        'StablePool'    = 4
    ) COMMENT 'Converter type',
    ADD COLUMN IF NOT EXISTS anchor             String COMMENT 'Converter anchor address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_source_token (source_token) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_target_token (target_token) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_trader (trader) TYPE bloom_filter GRANULARITY 1,

    -- indexes (Activation) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1;


-- Bancor LiquidityAdded --
CREATE TABLE IF NOT EXISTS bancor_liquidity_added AS TEMPLATE_LOG
COMMENT 'Bancor LiquidityAdded events';

ALTER TABLE bancor_liquidity_added
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS reserve_token      String COMMENT 'Reserve token contract address',
    ADD COLUMN IF NOT EXISTS amount             String COMMENT 'Amount of tokens added',
    ADD COLUMN IF NOT EXISTS new_balance        String COMMENT 'New reserve balance',
    ADD COLUMN IF NOT EXISTS new_supply         String COMMENT 'New pool token supply',

    -- Activation (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS converter_type     Enum8(
        'Unknown'       = 0,
        'LiquidToken'   = 1,
        'LiquidityPool' = 2,
        'FeeConverter'  = 3,
        'StablePool'    = 4
    ) COMMENT 'Converter type',
    ADD COLUMN IF NOT EXISTS anchor             String COMMENT 'Converter anchor address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_provider (provider) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_reserve_token (reserve_token) TYPE bloom_filter GRANULARITY 1,

    -- indexes (Activation) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1;


-- Bancor LiquidityRemoved --
CREATE TABLE IF NOT EXISTS bancor_liquidity_removed AS TEMPLATE_LOG
COMMENT 'Bancor LiquidityRemoved events';

ALTER TABLE bancor_liquidity_removed
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS reserve_token      String COMMENT 'Reserve token contract address',
    ADD COLUMN IF NOT EXISTS amount             String COMMENT 'Amount of tokens removed',
    ADD COLUMN IF NOT EXISTS new_balance        String COMMENT 'New reserve balance',
    ADD COLUMN IF NOT EXISTS new_supply         String COMMENT 'New pool token supply',

    -- Activation (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS converter_type     Enum8(
        'Unknown'       = 0,
        'LiquidToken'   = 1,
        'LiquidityPool' = 2,
        'FeeConverter'  = 3,
        'StablePool'    = 4
    ) COMMENT 'Converter type',
    ADD COLUMN IF NOT EXISTS anchor             String COMMENT 'Converter anchor address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_provider (provider) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_reserve_token (reserve_token) TYPE bloom_filter GRANULARITY 1,

    -- indexes (Activation) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1;


-- Bancor TokenRateUpdate --
CREATE TABLE IF NOT EXISTS bancor_token_rate_update AS TEMPLATE_LOG
COMMENT 'Bancor TokenRateUpdate events';

ALTER TABLE bancor_token_rate_update
    -- event information --
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'First token contract address',
    ADD COLUMN IF NOT EXISTS token2             String COMMENT 'Second token contract address',
    ADD COLUMN IF NOT EXISTS rate_n             String COMMENT 'Rate numerator',
    ADD COLUMN IF NOT EXISTS rate_d             String COMMENT 'Rate denominator',

    -- Activation (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS converter_type     Enum8(
        'Unknown'       = 0,
        'LiquidToken'   = 1,
        'LiquidityPool' = 2,
        'FeeConverter'  = 3,
        'StablePool'    = 4
    ) COMMENT 'Converter type',
    ADD COLUMN IF NOT EXISTS anchor             String COMMENT 'Converter anchor address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token2 (token2) TYPE bloom_filter GRANULARITY 1,

    -- indexes (Activation) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1;


-- Bancor Activation --
CREATE TABLE IF NOT EXISTS bancor_activation AS TEMPLATE_LOG
COMMENT 'Bancor Activation events';

ALTER TABLE bancor_activation
    ADD COLUMN IF NOT EXISTS converter_type Enum8(
        'Unknown'       = 0,
        'LiquidToken'   = 1,
        'LiquidityPool' = 2,
        'FeeConverter'  = 3,
        'StablePool'    = 4
    ) COMMENT 'Converter type',
    ADD COLUMN IF NOT EXISTS anchor       String COMMENT 'Converter anchor address',
    ADD COLUMN IF NOT EXISTS activated    UInt8  COMMENT 'True if the converter was activated';
