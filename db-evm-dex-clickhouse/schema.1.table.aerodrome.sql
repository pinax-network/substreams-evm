-- Aerodrome/Velodrome Swap (Solidly fork) --
CREATE TABLE IF NOT EXISTS aerodrome_swap AS TEMPLATE_LOG
COMMENT 'Aerodrome/Velodrome Swap events';
ALTER TABLE aerodrome_swap
    -- swap event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'sender wallet address',
    ADD COLUMN IF NOT EXISTS `to`               String COMMENT 'recipient wallet address',
    ADD COLUMN IF NOT EXISTS amount0_in         UInt256 COMMENT 'Amount of token0 in',
    ADD COLUMN IF NOT EXISTS amount1_in         UInt256 COMMENT 'Amount of token1 in',
    ADD COLUMN IF NOT EXISTS amount0_out        UInt256 COMMENT 'Amount of token0 out',
    ADD COLUMN IF NOT EXISTS amount1_out        UInt256 COMMENT 'Amount of token1 out',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS stable             Bool COMMENT 'Stable or volatile pool';

-- Aerodrome/Velodrome Mint --
CREATE TABLE IF NOT EXISTS aerodrome_mint AS TEMPLATE_LOG
COMMENT 'Aerodrome/Velodrome Mint events';
ALTER TABLE aerodrome_mint
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS amount0            UInt256 COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            UInt256 COMMENT 'Amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS stable             Bool COMMENT 'Stable or volatile pool';

-- Aerodrome/Velodrome Burn --
CREATE TABLE IF NOT EXISTS aerodrome_burn AS TEMPLATE_LOG
COMMENT 'Aerodrome/Velodrome Burn events';
ALTER TABLE aerodrome_burn
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS `to`               String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS amount0            UInt256 COMMENT 'Amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            UInt256 COMMENT 'Amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS stable             Bool COMMENT 'Stable or volatile pool';

-- Aerodrome/Velodrome Sync --
CREATE TABLE IF NOT EXISTS aerodrome_sync AS TEMPLATE_LOG
COMMENT 'Aerodrome/Velodrome Sync events';
ALTER TABLE aerodrome_sync
    ADD COLUMN IF NOT EXISTS reserve0           UInt256 COMMENT 'Reserve of token0',
    ADD COLUMN IF NOT EXISTS reserve1           UInt256 COMMENT 'Reserve of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS stable             Bool COMMENT 'Stable or volatile pool';

-- Aerodrome/Velodrome Fees --
CREATE TABLE IF NOT EXISTS aerodrome_fees AS TEMPLATE_LOG
COMMENT 'Aerodrome/Velodrome Fees events';
ALTER TABLE aerodrome_fees
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS amount0            UInt256 COMMENT 'Fee amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            UInt256 COMMENT 'Fee amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS stable             Bool COMMENT 'Stable or volatile pool';

-- Aerodrome/Velodrome Claim --
CREATE TABLE IF NOT EXISTS aerodrome_claim AS TEMPLATE_LOG
COMMENT 'Aerodrome/Velodrome Claim events';
ALTER TABLE aerodrome_claim
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS amount0            UInt256 COMMENT 'Claim amount of token0',
    ADD COLUMN IF NOT EXISTS amount1            UInt256 COMMENT 'Claim amount of token1',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS stable             Bool COMMENT 'Stable or volatile pool';

-- Aerodrome/Velodrome PoolCreated --
CREATE TABLE IF NOT EXISTS aerodrome_pool_created AS TEMPLATE_LOG
COMMENT 'Aerodrome/Velodrome PoolCreated events';
ALTER TABLE aerodrome_pool_created
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS stable             Bool COMMENT 'Stable or volatile pool',
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS extra_data         String COMMENT 'Extra data';
