-- Polymarket OrderFilled --
CREATE TABLE IF NOT EXISTS polymarket_order_filled AS TEMPLATE_LOG
COMMENT 'Polymarket OrderFilled events (swap events)';
ALTER TABLE polymarket_order_filled
    -- event information --
    ADD COLUMN IF NOT EXISTS order_hash           String COMMENT 'Order hash identifier',
    ADD COLUMN IF NOT EXISTS maker                String COMMENT 'Maker address',
    ADD COLUMN IF NOT EXISTS taker                String COMMENT 'Taker address',
    ADD COLUMN IF NOT EXISTS maker_asset_id       UInt256 COMMENT 'Maker asset token ID',
    ADD COLUMN IF NOT EXISTS taker_asset_id       UInt256 COMMENT 'Taker asset token ID',
    ADD COLUMN IF NOT EXISTS maker_amount_filled  UInt256 COMMENT 'Maker amount filled',
    ADD COLUMN IF NOT EXISTS taker_amount_filled  UInt256 COMMENT 'Taker amount filled',
    ADD COLUMN IF NOT EXISTS fee                  UInt256 COMMENT 'Fee amount';

-- Polymarket FeeCharged --
CREATE TABLE IF NOT EXISTS polymarket_fee_charged AS TEMPLATE_LOG
COMMENT 'Polymarket FeeCharged events';
ALTER TABLE polymarket_fee_charged
    -- event information --
    ADD COLUMN IF NOT EXISTS receiver             String COMMENT 'Fee receiver address',
    ADD COLUMN IF NOT EXISTS token_id             UInt256 COMMENT 'Token ID',
    ADD COLUMN IF NOT EXISTS amount               UInt256 COMMENT 'Fee amount';

-- Polymarket NewAdmin --
CREATE TABLE IF NOT EXISTS polymarket_new_admin AS TEMPLATE_LOG
COMMENT 'Polymarket NewAdmin events';
ALTER TABLE polymarket_new_admin
    -- event information --
    ADD COLUMN IF NOT EXISTS new_admin_address    String COMMENT 'New admin address',
    ADD COLUMN IF NOT EXISTS admin                String COMMENT 'Admin who added the new admin';

-- Polymarket NewOperator --
CREATE TABLE IF NOT EXISTS polymarket_new_operator AS TEMPLATE_LOG
COMMENT 'Polymarket NewOperator events';
ALTER TABLE polymarket_new_operator
    -- event information --
    ADD COLUMN IF NOT EXISTS new_operator_address String COMMENT 'New operator address',
    ADD COLUMN IF NOT EXISTS admin                String COMMENT 'Admin who added the new operator';

-- Polymarket OrderCancelled --
CREATE TABLE IF NOT EXISTS polymarket_order_cancelled AS TEMPLATE_LOG
COMMENT 'Polymarket OrderCancelled events';
ALTER TABLE polymarket_order_cancelled
    -- event information --
    ADD COLUMN IF NOT EXISTS order_hash           String COMMENT 'Order hash identifier';

-- Polymarket OrdersMatched --
CREATE TABLE IF NOT EXISTS polymarket_orders_matched AS TEMPLATE_LOG
COMMENT 'Polymarket OrdersMatched events';
ALTER TABLE polymarket_orders_matched
    -- event information --
    ADD COLUMN IF NOT EXISTS taker_order_hash     String COMMENT 'Taker order hash',
    ADD COLUMN IF NOT EXISTS taker_order_maker    String COMMENT 'Taker order maker address',
    ADD COLUMN IF NOT EXISTS maker_asset_id       UInt256 COMMENT 'Maker asset token ID',
    ADD COLUMN IF NOT EXISTS taker_asset_id       UInt256 COMMENT 'Taker asset token ID',
    ADD COLUMN IF NOT EXISTS maker_amount_filled  UInt256 COMMENT 'Maker amount filled',
    ADD COLUMN IF NOT EXISTS taker_amount_filled  UInt256 COMMENT 'Taker amount filled';

-- Polymarket ProxyFactoryUpdated --
CREATE TABLE IF NOT EXISTS polymarket_proxy_factory_updated AS TEMPLATE_LOG
COMMENT 'Polymarket ProxyFactoryUpdated events';
ALTER TABLE polymarket_proxy_factory_updated
    -- event information --
    ADD COLUMN IF NOT EXISTS old_proxy_factory    String COMMENT 'Old proxy factory address',
    ADD COLUMN IF NOT EXISTS new_proxy_factory    String COMMENT 'New proxy factory address';

-- Polymarket RemovedAdmin --
CREATE TABLE IF NOT EXISTS polymarket_removed_admin AS TEMPLATE_LOG
COMMENT 'Polymarket RemovedAdmin events';
ALTER TABLE polymarket_removed_admin
    -- event information --
    ADD COLUMN IF NOT EXISTS removed_admin        String COMMENT 'Removed admin address',
    ADD COLUMN IF NOT EXISTS admin                String COMMENT 'Admin who removed the admin';

-- Polymarket RemovedOperator --
CREATE TABLE IF NOT EXISTS polymarket_removed_operator AS TEMPLATE_LOG
COMMENT 'Polymarket RemovedOperator events';
ALTER TABLE polymarket_removed_operator
    -- event information --
    ADD COLUMN IF NOT EXISTS removed_operator     String COMMENT 'Removed operator address',
    ADD COLUMN IF NOT EXISTS admin                String COMMENT 'Admin who removed the operator';

-- Polymarket SafeFactoryUpdated --
CREATE TABLE IF NOT EXISTS polymarket_safe_factory_updated AS TEMPLATE_LOG
COMMENT 'Polymarket SafeFactoryUpdated events';
ALTER TABLE polymarket_safe_factory_updated
    -- event information --
    ADD COLUMN IF NOT EXISTS old_safe_factory     String COMMENT 'Old safe factory address',
    ADD COLUMN IF NOT EXISTS new_safe_factory     String COMMENT 'New safe factory address';

-- Polymarket TokenRegistered --
CREATE TABLE IF NOT EXISTS polymarket_token_registered AS TEMPLATE_LOG
COMMENT 'Polymarket TokenRegistered events';
ALTER TABLE polymarket_token_registered
    -- event information --
    ADD COLUMN IF NOT EXISTS condition_id         String COMMENT 'Condition ID (bytes32 as hex with 0x prefix)',
    ADD COLUMN IF NOT EXISTS token0               UInt256 COMMENT 'Token0 ID',
    ADD COLUMN IF NOT EXISTS token1               UInt256 COMMENT 'Token1 ID';

-- Polymarket TradingPaused --
CREATE TABLE IF NOT EXISTS polymarket_trading_paused AS TEMPLATE_LOG
COMMENT 'Polymarket TradingPaused events';
ALTER TABLE polymarket_trading_paused
    -- event information --
    ADD COLUMN IF NOT EXISTS pauser               String COMMENT 'Address that paused trading';

-- Polymarket TradingUnpaused --
CREATE TABLE IF NOT EXISTS polymarket_trading_unpaused AS TEMPLATE_LOG
COMMENT 'Polymarket TradingUnpaused events';
ALTER TABLE polymarket_trading_unpaused
    -- event information --
    ADD COLUMN IF NOT EXISTS pauser               String COMMENT 'Address that unpaused trading';
