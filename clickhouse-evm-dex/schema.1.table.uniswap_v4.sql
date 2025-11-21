-- Uniswap V4 Swap --
CREATE TABLE IF NOT EXISTS uniswap_v4_swap AS TEMPLATE_LOG
COMMENT 'Uniswap V4 Swap events';
ALTER TABLE uniswap_v4_swap
    -- swap event information --
    ADD COLUMN IF NOT EXISTS id                 String COMMENT 'Pool ID',
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'sender wallet address',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of currency0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of currency1',
    ADD COLUMN IF NOT EXISTS sqrt_price_x96     String COMMENT 'Square root price',
    ADD COLUMN IF NOT EXISTS liquidity          String COMMENT 'Liquidity',
    ADD COLUMN IF NOT EXISTS tick               Int32 COMMENT 'Tick',
    ADD COLUMN IF NOT EXISTS fee                String COMMENT 'Fee',

    -- Initialize --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS currency0          String COMMENT 'Currency0 address',
    ADD COLUMN IF NOT EXISTS currency1          String COMMENT 'Currency1 address',
    ADD COLUMN IF NOT EXISTS tick_spacing       Int32 COMMENT 'Tick spacing',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_id (id) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_tick (tick) TYPE minmax GRANULARITY 1,

    -- indexes (Initialize) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency0 (currency0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency1 (currency1) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V4 Initialize --
CREATE TABLE IF NOT EXISTS uniswap_v4_initialize AS TEMPLATE_LOG
COMMENT 'Uniswap V4 Initialize events';
ALTER TABLE uniswap_v4_initialize
    -- event information --
    ADD COLUMN IF NOT EXISTS id                 String COMMENT 'Pool ID',
    ADD COLUMN IF NOT EXISTS currency0          String COMMENT 'Currency0 address',
    ADD COLUMN IF NOT EXISTS currency1          String COMMENT 'Currency1 address',
    ADD COLUMN IF NOT EXISTS fee                UInt64 COMMENT 'Fee tier',
    ADD COLUMN IF NOT EXISTS tick_spacing       Int32 COMMENT 'Tick spacing',
    ADD COLUMN IF NOT EXISTS sqrt_price_x96     String COMMENT 'Square root price',
    ADD COLUMN IF NOT EXISTS tick               Int32 COMMENT 'Tick',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_id (id) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency0 (currency0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency1 (currency1) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_tick (tick) TYPE minmax GRANULARITY 1;

-- Uniswap V4 ModifyLiquidity --
CREATE TABLE IF NOT EXISTS uniswap_v4_modify_liquidity AS TEMPLATE_LOG
COMMENT 'Uniswap V4 ModifyLiquidity events';
ALTER TABLE uniswap_v4_modify_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS id                 String COMMENT 'Pool ID',
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS tick_lower         Int32 COMMENT 'Lower tick',
    ADD COLUMN IF NOT EXISTS tick_upper         Int32 COMMENT 'Upper tick',
    ADD COLUMN IF NOT EXISTS liquidity_delta    String COMMENT 'Liquidity delta',
    ADD COLUMN IF NOT EXISTS salt               String COMMENT 'Salt',

    -- Initialize --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS currency0          String COMMENT 'Currency0 address',
    ADD COLUMN IF NOT EXISTS currency1          String COMMENT 'Currency1 address',
    ADD COLUMN IF NOT EXISTS fee                UInt64 COMMENT 'Fee tier',
    ADD COLUMN IF NOT EXISTS tick_spacing       Int32 COMMENT 'Tick spacing',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_id (id) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_tick_lower (tick_lower) TYPE minmax GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_tick_upper (tick_upper) TYPE minmax GRANULARITY 1,

    -- indexes (Initialize) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency0 (currency0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency1 (currency1) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V4 Donate --
CREATE TABLE IF NOT EXISTS uniswap_v4_donate AS TEMPLATE_LOG
COMMENT 'Uniswap V4 Donate events';
ALTER TABLE uniswap_v4_donate
    -- event information --
    ADD COLUMN IF NOT EXISTS id                 String COMMENT 'Pool ID',
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of currency0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of currency1',

    -- Initialize --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS currency0          String COMMENT 'Currency0 address',
    ADD COLUMN IF NOT EXISTS currency1          String COMMENT 'Currency1 address',
    ADD COLUMN IF NOT EXISTS fee                UInt64 COMMENT 'Fee tier',
    ADD COLUMN IF NOT EXISTS tick_spacing       Int32 COMMENT 'Tick spacing',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_id (id) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter GRANULARITY 1,

    -- indexes (Initialize) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency0 (currency0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency1 (currency1) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V4 ProtocolFeeControllerUpdated --
CREATE TABLE IF NOT EXISTS uniswap_v4_protocol_fee_controller_updated AS TEMPLATE_LOG
COMMENT 'Uniswap V4 ProtocolFeeControllerUpdated events';
ALTER TABLE uniswap_v4_protocol_fee_controller_updated
    -- event information --
    ADD COLUMN IF NOT EXISTS protocol_fee_controller  String COMMENT 'Protocol fee controller address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_protocol_fee_controller (protocol_fee_controller) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V4 ProtocolFeeUpdated --
CREATE TABLE IF NOT EXISTS uniswap_v4_protocol_fee_updated AS TEMPLATE_LOG
COMMENT 'Uniswap V4 ProtocolFeeUpdated events';
ALTER TABLE uniswap_v4_protocol_fee_updated
    -- event information --
    ADD COLUMN IF NOT EXISTS id                 String COMMENT 'Pool ID',
    ADD COLUMN IF NOT EXISTS protocol_fee       UInt64 COMMENT 'Protocol fee',

    -- Initialize --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS currency0          String COMMENT 'Currency0 address',
    ADD COLUMN IF NOT EXISTS currency1          String COMMENT 'Currency1 address',
    ADD COLUMN IF NOT EXISTS fee                UInt64 COMMENT 'Fee tier',
    ADD COLUMN IF NOT EXISTS tick_spacing       Int32 COMMENT 'Tick spacing',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_id (id) TYPE bloom_filter GRANULARITY 1,

    -- indexes (Initialize) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency0 (currency0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_currency1 (currency1) TYPE bloom_filter GRANULARITY 1;
