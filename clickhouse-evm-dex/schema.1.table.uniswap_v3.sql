-- Uniswap V3 Swap --
CREATE TABLE IF NOT EXISTS uniswap_v3_swap AS TEMPLATE_LOG
COMMENT 'Uniswap V3 Swap events';
ALTER TABLE uniswap_v3_swap
    -- swap event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'sender wallet address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'recipient wallet address',
    ADD COLUMN IF NOT EXISTS amount0            Int256 COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            Int256 COMMENT 'Amount of token1',
    ADD COLUMN IF NOT EXISTS sqrt_price_x96     UInt256 COMMENT 'Square root price',
    ADD COLUMN IF NOT EXISTS liquidity          UInt128 COMMENT 'Liquidity',
    ADD COLUMN IF NOT EXISTS tick               Int32 COMMENT 'Tick',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_recipient (recipient) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_tick (tick) TYPE minmax,

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 Initialize --
CREATE TABLE IF NOT EXISTS uniswap_v3_initialize AS TEMPLATE_LOG
COMMENT 'Uniswap V3 Initialize events';
ALTER TABLE uniswap_v3_initialize
    -- event information --
    ADD COLUMN IF NOT EXISTS sqrt_price_x96     UInt256 COMMENT 'Square root price',
    ADD COLUMN IF NOT EXISTS tick               Int32 COMMENT 'Tick',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_tick (tick) TYPE minmax,

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 Mint --
CREATE TABLE IF NOT EXISTS uniswap_v3_mint AS TEMPLATE_LOG
COMMENT 'Uniswap V3 Mint events';
ALTER TABLE uniswap_v3_mint
    -- event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Owner address',
    ADD COLUMN IF NOT EXISTS tick_lower         Int32 COMMENT 'Lower tick',
    ADD COLUMN IF NOT EXISTS tick_upper         Int32 COMMENT 'Upper tick',
    ADD COLUMN IF NOT EXISTS amount             UInt128 COMMENT 'Liquidity amount',
    ADD COLUMN IF NOT EXISTS amount0            UInt256 COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            UInt256 COMMENT 'Amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_owner (owner) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_tick_lower (tick_lower) TYPE minmax,
    ADD INDEX IF NOT EXISTS idx_tick_upper (tick_upper) TYPE minmax,

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 Burn --
CREATE TABLE IF NOT EXISTS uniswap_v3_burn AS TEMPLATE_LOG
COMMENT 'Uniswap V3 Burn events';
ALTER TABLE uniswap_v3_burn
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Owner address',
    ADD COLUMN IF NOT EXISTS tick_lower         Int32 COMMENT 'Lower tick',
    ADD COLUMN IF NOT EXISTS tick_upper         Int32 COMMENT 'Upper tick',
    ADD COLUMN IF NOT EXISTS amount             String COMMENT 'Liquidity amount',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_owner (owner) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_tick_lower (tick_lower) TYPE minmax,
    ADD INDEX IF NOT EXISTS idx_tick_upper (tick_upper) TYPE minmax,

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 Collect --
CREATE TABLE IF NOT EXISTS uniswap_v3_collect AS TEMPLATE_LOG
COMMENT 'Uniswap V3 Collect events';
ALTER TABLE uniswap_v3_collect
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Owner address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS tick_lower         Int32 COMMENT 'Lower tick',
    ADD COLUMN IF NOT EXISTS tick_upper         Int32 COMMENT 'Upper tick',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_owner (owner) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_recipient (recipient) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_tick_lower (tick_lower) TYPE minmax,
    ADD INDEX IF NOT EXISTS idx_tick_upper (tick_upper) TYPE minmax,

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 Flash --
CREATE TABLE IF NOT EXISTS uniswap_v3_flash AS TEMPLATE_LOG
COMMENT 'Uniswap V3 Flash events';
ALTER TABLE uniswap_v3_flash
    -- event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of token1',
    ADD COLUMN IF NOT EXISTS paid0              String COMMENT 'Amount of token0 paid',
    ADD COLUMN IF NOT EXISTS paid1              String COMMENT 'Amount of token1 paid',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_recipient (recipient) TYPE bloom_filter,

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 IncreaseObservationCardinalityNext --
CREATE TABLE IF NOT EXISTS uniswap_v3_increase_observation_cardinality_next AS TEMPLATE_LOG
COMMENT 'Uniswap V3 IncreaseObservationCardinalityNext events';
ALTER TABLE uniswap_v3_increase_observation_cardinality_next
    -- event information --
    ADD COLUMN IF NOT EXISTS observation_cardinality_next_old  UInt32 COMMENT 'Old observation cardinality',
    ADD COLUMN IF NOT EXISTS observation_cardinality_next_new  UInt32 COMMENT 'New observation cardinality',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 SetFeeProtocol --
CREATE TABLE IF NOT EXISTS uniswap_v3_set_fee_protocol AS TEMPLATE_LOG
COMMENT 'Uniswap V3 SetFeeProtocol events';
ALTER TABLE uniswap_v3_set_fee_protocol
    -- event information --
    ADD COLUMN IF NOT EXISTS fee_protocol0_old  UInt32 COMMENT 'Old fee protocol for token0',
    ADD COLUMN IF NOT EXISTS fee_protocol1_old  UInt32 COMMENT 'Old fee protocol for token1',
    ADD COLUMN IF NOT EXISTS fee_protocol0_new  UInt32 COMMENT 'New fee protocol for token0',
    ADD COLUMN IF NOT EXISTS fee_protocol1_new  UInt32 COMMENT 'New fee protocol for token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 CollectProtocol --
CREATE TABLE IF NOT EXISTS uniswap_v3_collect_protocol AS TEMPLATE_LOG
COMMENT 'Uniswap V3 CollectProtocol events';
ALTER TABLE uniswap_v3_collect_protocol
    -- event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_recipient (recipient) TYPE bloom_filter,

    -- indexes (PoolCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter;

-- Uniswap V3 PoolCreated --
CREATE TABLE IF NOT EXISTS uniswap_v3_pool_created AS TEMPLATE_LOG
COMMENT 'Uniswap V3 PoolCreated events';
ALTER TABLE uniswap_v3_pool_created
    -- event information --
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS fee                UInt64 COMMENT 'Fee tier',
    ADD COLUMN IF NOT EXISTS tick_spacing       Int32 COMMENT 'Tick spacing',
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_pool (pool) TYPE bloom_filter;

-- Uniswap V3 OwnerChanged --
CREATE TABLE IF NOT EXISTS uniswap_v3_owner_changed AS TEMPLATE_LOG
COMMENT 'Uniswap V3 OwnerChanged events';
ALTER TABLE uniswap_v3_owner_changed
    -- event information --
    ADD COLUMN IF NOT EXISTS old_owner          String COMMENT 'Old owner address',
    ADD COLUMN IF NOT EXISTS new_owner          String COMMENT 'New owner address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_old_owner (old_owner) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_new_owner (new_owner) TYPE bloom_filter;

-- Uniswap V3 FeeAmountEnabled --
CREATE TABLE IF NOT EXISTS uniswap_v3_fee_amount_enabled AS TEMPLATE_LOG
COMMENT 'Uniswap V3 FeeAmountEnabled events';
ALTER TABLE uniswap_v3_fee_amount_enabled
    -- event information --
    ADD COLUMN IF NOT EXISTS fee                UInt64 COMMENT 'Fee tier',
    ADD COLUMN IF NOT EXISTS tick_spacing       Int32 COMMENT 'Tick spacing';
