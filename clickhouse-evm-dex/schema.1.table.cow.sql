-- CoW Protocol Trade (Swap) --
CREATE TABLE IF NOT EXISTS cow_trade AS TEMPLATE_LOG
COMMENT 'CoW Protocol Trade (swap) events';
ALTER TABLE cow_trade
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Order owner wallet address',
    ADD COLUMN IF NOT EXISTS sell_token         String COMMENT 'Token being sold',
    ADD COLUMN IF NOT EXISTS buy_token          String COMMENT 'Token being bought',
    ADD COLUMN IF NOT EXISTS sell_amount        String COMMENT 'Amount of tokens sold',
    ADD COLUMN IF NOT EXISTS buy_amount         String COMMENT 'Amount of tokens bought',
    ADD COLUMN IF NOT EXISTS fee_amount         String COMMENT 'Fee amount charged',
    ADD COLUMN IF NOT EXISTS order_uid          String COMMENT 'Unique order identifier',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_owner (owner) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_sell_token (sell_token) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_buy_token (buy_token) TYPE bloom_filter GRANULARITY 1;

-- CoW Protocol Settlement --
CREATE TABLE IF NOT EXISTS cow_settlement AS TEMPLATE_LOG
COMMENT 'CoW Protocol Settlement events';
ALTER TABLE cow_settlement
    -- event information --
    ADD COLUMN IF NOT EXISTS solver             String COMMENT 'Solver address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_solver (solver) TYPE bloom_filter GRANULARITY 1;

-- CoW Protocol OrderInvalidated --
CREATE TABLE IF NOT EXISTS cow_order_invalidated AS TEMPLATE_LOG
COMMENT 'CoW Protocol OrderInvalidated events';
ALTER TABLE cow_order_invalidated
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Order owner address',
    ADD COLUMN IF NOT EXISTS order_uid          String COMMENT 'Unique order identifier',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_owner (owner) TYPE bloom_filter GRANULARITY 1;

-- CoW Protocol PreSignature --
CREATE TABLE IF NOT EXISTS cow_pre_signature AS TEMPLATE_LOG
COMMENT 'CoW Protocol PreSignature events';
ALTER TABLE cow_pre_signature
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Order owner address',
    ADD COLUMN IF NOT EXISTS order_uid          String COMMENT 'Unique order identifier',
    ADD COLUMN IF NOT EXISTS signed             UInt8 COMMENT 'Whether the order is signed',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_owner (owner) TYPE bloom_filter GRANULARITY 1;
