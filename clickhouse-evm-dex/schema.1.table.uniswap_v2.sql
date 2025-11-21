-- Uniswap V2 Swap --
CREATE TABLE IF NOT EXISTS uniswap_v2_swap AS TEMPLATE_LOG
COMMENT 'Uniswap V2 Swap events';
ALTER TABLE uniswap_v2_swap
    -- swap event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'sender wallet address',
    ADD COLUMN IF NOT EXISTS to                 String COMMENT 'recipient wallet address',
    ADD COLUMN IF NOT EXISTS amount0_in         String COMMENT 'Amount of token0 in',
    ADD COLUMN IF NOT EXISTS amount1_in         String COMMENT 'Amount of token1 in',
    ADD COLUMN IF NOT EXISTS amount0_out        String COMMENT 'Amount of token0 out',
    ADD COLUMN IF NOT EXISTS amount1_out        String COMMENT 'Amount of token1 out',

    -- PairCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_to (to) TYPE bloom_filter GRANULARITY 1,

    -- indexes (PairCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V2 Mint --
CREATE TABLE IF NOT EXISTS uniswap_v2_mint AS TEMPLATE_LOG
COMMENT 'Uniswap V2 Mint events';
ALTER TABLE uniswap_v2_mint
    -- event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of token1',

    -- PairCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter GRANULARITY 1,

    -- indexes (PairCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V2 Burn --
CREATE TABLE IF NOT EXISTS uniswap_v2_burn AS TEMPLATE_LOG
COMMENT 'Uniswap V2 Burn events';
ALTER TABLE uniswap_v2_burn
    -- event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS amount0            String COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            String COMMENT 'Amount of token1',
    ADD COLUMN IF NOT EXISTS to                 String COMMENT 'Recipient address',

    -- PairCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_sender (sender) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_to (to) TYPE bloom_filter GRANULARITY 1,

    -- indexes (PairCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V2 Sync --
CREATE TABLE IF NOT EXISTS uniswap_v2_sync AS TEMPLATE_LOG
COMMENT 'Uniswap V2 Sync events';
ALTER TABLE uniswap_v2_sync
    -- event information --
    ADD COLUMN IF NOT EXISTS reserve0           String COMMENT 'Reserve of token0',
    ADD COLUMN IF NOT EXISTS reserve1           String COMMENT 'Reserve of token1',

    -- PairCreated --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',

    -- indexes (PairCreated) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter GRANULARITY 1;

-- Uniswap V2 PairCreated --
CREATE TABLE IF NOT EXISTS uniswap_v2_pair_created AS TEMPLATE_LOG
COMMENT 'Uniswap V2 PairCreated events';
ALTER TABLE uniswap_v2_pair_created
    -- event information --
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS pair               String COMMENT 'Pair contract address',
    ADD COLUMN IF NOT EXISTS extra_data         UInt64 COMMENT 'Extra data',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_token0 (token0) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_token1 (token1) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_pair (pair) TYPE bloom_filter GRANULARITY 1;
