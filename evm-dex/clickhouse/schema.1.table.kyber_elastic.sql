-- KyberSwap Elastic Swap --
CREATE TABLE IF NOT EXISTS kyber_elastic_swap AS TEMPLATE_LOG
COMMENT 'KyberSwap Elastic Swap events';
ALTER TABLE kyber_elastic_swap
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS delta_qty0         Int256 COMMENT 'Delta quantity of token0 (signed: positive=received, negative=paid)',
    ADD COLUMN IF NOT EXISTS delta_qty1         Int256 COMMENT 'Delta quantity of token1 (signed: positive=received, negative=paid)',
    ADD COLUMN IF NOT EXISTS sqrt_p             UInt256 COMMENT 'Square root price (uint160)',
    ADD COLUMN IF NOT EXISTS liquidity          UInt256 COMMENT 'Liquidity (uint128)',
    ADD COLUMN IF NOT EXISTS current_tick       Int32 COMMENT 'Current tick (int24)',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS swap_fee_units     UInt32 COMMENT 'Swap fee units',
    ADD COLUMN IF NOT EXISTS tick_distance      Int32 COMMENT 'Tick distance';

-- KyberSwap Elastic Mint --
CREATE TABLE IF NOT EXISTS kyber_elastic_mint AS TEMPLATE_LOG
COMMENT 'KyberSwap Elastic Mint events';
ALTER TABLE kyber_elastic_mint
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Owner address',
    ADD COLUMN IF NOT EXISTS tick_lower         Int32 COMMENT 'Lower tick',
    ADD COLUMN IF NOT EXISTS tick_upper         Int32 COMMENT 'Upper tick',
    ADD COLUMN IF NOT EXISTS qty                UInt256 COMMENT 'Liquidity amount (uint128)',
    ADD COLUMN IF NOT EXISTS qty0               UInt256 COMMENT 'Token0 amount',
    ADD COLUMN IF NOT EXISTS qty1               UInt256 COMMENT 'Token1 amount',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS swap_fee_units     UInt32 COMMENT 'Swap fee units',
    ADD COLUMN IF NOT EXISTS tick_distance      Int32 COMMENT 'Tick distance';

-- KyberSwap Elastic Burn --
CREATE TABLE IF NOT EXISTS kyber_elastic_burn AS TEMPLATE_LOG
COMMENT 'KyberSwap Elastic Burn events';
ALTER TABLE kyber_elastic_burn
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Owner address',
    ADD COLUMN IF NOT EXISTS tick_lower         Int32 COMMENT 'Lower tick',
    ADD COLUMN IF NOT EXISTS tick_upper         Int32 COMMENT 'Upper tick',
    ADD COLUMN IF NOT EXISTS qty                UInt256 COMMENT 'Liquidity amount (uint128)',
    ADD COLUMN IF NOT EXISTS qty0               UInt256 COMMENT 'Token0 amount',
    ADD COLUMN IF NOT EXISTS qty1               UInt256 COMMENT 'Token1 amount',

    -- PoolCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS swap_fee_units     UInt32 COMMENT 'Swap fee units',
    ADD COLUMN IF NOT EXISTS tick_distance      Int32 COMMENT 'Tick distance';

-- KyberSwap Elastic PoolCreated --
CREATE TABLE IF NOT EXISTS kyber_elastic_pool_created AS TEMPLATE_LOG
COMMENT 'KyberSwap Elastic PoolCreated events';
ALTER TABLE kyber_elastic_pool_created
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'Token0 contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'Token1 contract address',
    ADD COLUMN IF NOT EXISTS swap_fee_units     UInt32 COMMENT 'Swap fee units',
    ADD COLUMN IF NOT EXISTS tick_distance      Int32 COMMENT 'Tick distance',
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address';
