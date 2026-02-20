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
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address';

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
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address';

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
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address';

-- Balancer PoolRegistered --
CREATE TABLE IF NOT EXISTS balancer_pool_registered AS TEMPLATE_LOG
COMMENT 'Balancer PoolRegistered events';
ALTER TABLE balancer_pool_registered
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS token_config       String COMMENT 'JSON array of token configurations',
    ADD COLUMN IF NOT EXISTS swap_fee_percentage UInt256 COMMENT 'Swap fee percentage for the pool',
    ADD COLUMN IF NOT EXISTS pause_window_end_time UInt256 COMMENT 'Timestamp when the pause window ends',

    -- role accounts --
    ADD COLUMN IF NOT EXISTS pause_manager      String COMMENT 'Pause manager address',
    ADD COLUMN IF NOT EXISTS swap_fee_manager   String COMMENT 'Swap fee manager address',
    ADD COLUMN IF NOT EXISTS pool_creator       String COMMENT 'Pool creator address',

    -- hooks config --
    ADD COLUMN IF NOT EXISTS enable_hook_adjusted_amounts Bool COMMENT 'Hook adjusted amounts flag',
    ADD COLUMN IF NOT EXISTS should_call_before_initialize Bool COMMENT 'Should call before initialize flag',
    ADD COLUMN IF NOT EXISTS should_call_after_initialize Bool COMMENT 'Should call after initialize flag',
    ADD COLUMN IF NOT EXISTS should_call_compute_dynamic_swap_fee Bool COMMENT 'Should call compute dynamic swap fee flag',
    ADD COLUMN IF NOT EXISTS should_call_before_swap Bool COMMENT 'Should call before swap flag',
    ADD COLUMN IF NOT EXISTS should_call_after_swap Bool COMMENT 'Should call after swap flag',
    ADD COLUMN IF NOT EXISTS should_call_before_add_liquidity Bool COMMENT 'Should call before add liquidity flag',
    ADD COLUMN IF NOT EXISTS should_call_after_add_liquidity Bool COMMENT 'Should call after add liquidity flag',
    ADD COLUMN IF NOT EXISTS should_call_before_remove_liquidity Bool COMMENT 'Should call before remove liquidity flag',
    ADD COLUMN IF NOT EXISTS should_call_after_remove_liquidity Bool COMMENT 'Should call after remove liquidity flag',
    ADD COLUMN IF NOT EXISTS hooks_address      String COMMENT 'Hooks contract address',

    -- liquidity management --
    ADD COLUMN IF NOT EXISTS disable_unbalanced_liquidity Bool COMMENT 'Disable unbalanced liquidity flag',
    ADD COLUMN IF NOT EXISTS enable_add_liquidity_custom Bool COMMENT 'Enable add liquidity custom flag',
    ADD COLUMN IF NOT EXISTS enable_remove_liquidity_custom Bool COMMENT 'Enable remove liquidity custom flag',
    ADD COLUMN IF NOT EXISTS enable_donation    Bool COMMENT 'Enable donation flag';

-- Balancer SwapFeePercentage --
CREATE TABLE IF NOT EXISTS balancer_swap_fee_percentage AS TEMPLATE_LOG
COMMENT 'Balancer V2 SwapFeePercentage events';
ALTER TABLE balancer_swap_fee_percentage
    -- event information --
    ADD COLUMN IF NOT EXISTS swap_fee_percentage    UInt256 COMMENT 'Swap fee percentage',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory                String COMMENT 'Factory contract address';

-- Balancer ProtocolFeePercentage --
CREATE TABLE IF NOT EXISTS balancer_protocol_fee_percentage AS TEMPLATE_LOG
COMMENT 'Balancer V2 ProtocolFeePercentage events';
ALTER TABLE balancer_protocol_fee_percentage
    -- event information --
    ADD COLUMN IF NOT EXISTS fee_type           UInt256 COMMENT 'Fee type being updated',
    ADD COLUMN IF NOT EXISTS protocol_fee_percentage UInt256 COMMENT 'Protocol fee percentage';

-- Balancer AggregateSwapFeePercentage --
CREATE TABLE IF NOT EXISTS balancer_aggregate_swap_fee_percentage AS TEMPLATE_LOG
COMMENT 'Balancer V3 AggregateSwapFeePercentage events';
ALTER TABLE balancer_aggregate_swap_fee_percentage
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS aggregate_swap_fee_percentage UInt256 COMMENT 'Aggregate swap fee percentage',

    -- PoolRegistered (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address';

