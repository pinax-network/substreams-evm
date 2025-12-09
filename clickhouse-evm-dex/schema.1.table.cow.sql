-- CoW Protocol Trade (Swap) --
CREATE TABLE IF NOT EXISTS cow_trade AS TEMPLATE_LOG
COMMENT 'CoW Protocol Trade (swap) events';
ALTER TABLE cow_trade
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Order owner wallet address',
    ADD COLUMN IF NOT EXISTS sell_token         String COMMENT 'Token being sold',
    ADD COLUMN IF NOT EXISTS buy_token          String COMMENT 'Token being bought',
    ADD COLUMN IF NOT EXISTS sell_amount        UInt256 COMMENT 'Amount of tokens sold',
    ADD COLUMN IF NOT EXISTS buy_amount         UInt256 COMMENT 'Amount of tokens bought',
    ADD COLUMN IF NOT EXISTS fee_amount         UInt256 COMMENT 'Fee amount charged',
    ADD COLUMN IF NOT EXISTS order_uid          String COMMENT 'Unique order identifier';

-- CoW Protocol Settlement --
CREATE TABLE IF NOT EXISTS cow_settlement AS TEMPLATE_LOG
COMMENT 'CoW Protocol Settlement events';
ALTER TABLE cow_settlement
    -- event information --
    ADD COLUMN IF NOT EXISTS solver             String COMMENT 'Solver address';

-- CoW Protocol OrderInvalidated --
CREATE TABLE IF NOT EXISTS cow_order_invalidated AS TEMPLATE_LOG
COMMENT 'CoW Protocol OrderInvalidated events';
ALTER TABLE cow_order_invalidated
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Order owner address',
    ADD COLUMN IF NOT EXISTS order_uid          String COMMENT 'Unique order identifier';

-- CoW Protocol PreSignature --
CREATE TABLE IF NOT EXISTS cow_pre_signature AS TEMPLATE_LOG
COMMENT 'CoW Protocol PreSignature events';
ALTER TABLE cow_pre_signature
    -- event information --
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Order owner address',
    ADD COLUMN IF NOT EXISTS order_uid          String COMMENT 'Unique order identifier',
    ADD COLUMN IF NOT EXISTS signed             UInt8 COMMENT 'Whether the order is signed';