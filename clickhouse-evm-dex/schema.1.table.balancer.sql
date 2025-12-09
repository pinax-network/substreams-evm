-- Balancer V3 Vault Swap --
CREATE TABLE IF NOT EXISTS balancer_vault_swap AS TEMPLATE_LOG
COMMENT 'Balancer V3 Vault Swap events';
ALTER TABLE balancer_vault_swap
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS token_in           String COMMENT 'Token in contract address',
    ADD COLUMN IF NOT EXISTS token_out          String COMMENT 'Token out contract address',
    ADD COLUMN IF NOT EXISTS amount_in          UInt256 COMMENT 'Amount of token in',
    ADD COLUMN IF NOT EXISTS amount_out         UInt256 COMMENT 'Amount of token out',
    ADD COLUMN IF NOT EXISTS swap_fee_percentage UInt256 COMMENT 'Swap fee percentage',
    ADD COLUMN IF NOT EXISTS swap_fee_amount    UInt256 COMMENT 'Swap fee amount',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_pool (pool) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token_in (token_in) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_token_out (token_out) TYPE bloom_filter,

    -- indexes (PoolRegistered) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

-- Balancer LiquidityAdded --
CREATE TABLE IF NOT EXISTS balancer_liquidity_added AS TEMPLATE_LOG
COMMENT 'Balancer LiquidityAdded events';
ALTER TABLE balancer_liquidity_added
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS liquidity_provider String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS kind               UInt32 COMMENT 'Type of liquidity addition',
    ADD COLUMN IF NOT EXISTS total_supply       UInt256 COMMENT 'Total supply after addition',
    ADD COLUMN IF NOT EXISTS amounts_added_raw  String COMMENT 'Comma-separated amounts added',
    ADD COLUMN IF NOT EXISTS swap_fee_amounts_raw String COMMENT 'Comma-separated swap fee amounts',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_pool (pool) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_liquidity_provider (liquidity_provider) TYPE bloom_filter,

    -- indexes (PoolRegistered) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

-- Balancer LiquidityRemoved --
CREATE TABLE IF NOT EXISTS balancer_liquidity_removed AS TEMPLATE_LOG
COMMENT 'Balancer LiquidityRemoved events';
ALTER TABLE balancer_liquidity_removed
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS liquidity_provider String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS kind               UInt32 COMMENT 'Type of liquidity removal',
    ADD COLUMN IF NOT EXISTS total_supply       UInt256 COMMENT 'Total supply after removal',
    ADD COLUMN IF NOT EXISTS amounts_removed_raw String COMMENT 'Comma-separated amounts removed',
    ADD COLUMN IF NOT EXISTS swap_fee_amounts_raw String COMMENT 'Comma-separated swap fee amounts',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_pool (pool) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_liquidity_provider (liquidity_provider) TYPE bloom_filter,

    -- indexes (PoolRegistered) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

-- Balancer PoolRegistered --
CREATE TABLE IF NOT EXISTS balancer_pool_registered AS TEMPLATE_LOG
COMMENT 'Balancer PoolRegistered events';
ALTER TABLE balancer_pool_registered
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_pool (pool) TYPE bloom_filter;

-- Balancer SwapFeePercentage --
CREATE TABLE IF NOT EXISTS balancer_swap_fee_percentage AS TEMPLATE_LOG
COMMENT 'Balancer V2 SwapFeePercentage events';
ALTER TABLE balancer_swap_fee_percentage
    -- event information --
    ADD COLUMN IF NOT EXISTS swap_fee_percentage UInt256 COMMENT 'Swap fee percentage',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',

    -- indexes (PoolRegistered) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

-- Balancer ProtocolFeePercentage --
CREATE TABLE IF NOT EXISTS balancer_protocol_fee_percentage AS TEMPLATE_LOG
COMMENT 'Balancer V2 ProtocolFeePercentage events';
ALTER TABLE balancer_protocol_fee_percentage
    -- event information --
    ADD COLUMN IF NOT EXISTS fee_type           UInt256 COMMENT 'Fee type being updated',
    ADD COLUMN IF NOT EXISTS protocol_fee_percentage UInt256 COMMENT 'Protocol fee percentage',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',

    -- indexes (PoolRegistered) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

-- Balancer AggregateSwapFeePercentage --
CREATE TABLE IF NOT EXISTS balancer_aggregate_swap_fee_percentage AS TEMPLATE_LOG
COMMENT 'Balancer V3 AggregateSwapFeePercentage events';
ALTER TABLE balancer_aggregate_swap_fee_percentage
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS aggregate_swap_fee_percentage UInt256 COMMENT 'Aggregate swap fee percentage',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_pool (pool) TYPE bloom_filter,

    -- indexes (PoolRegistered) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

