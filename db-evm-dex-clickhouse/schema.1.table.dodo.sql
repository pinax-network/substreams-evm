-- DODO V2 OrderHistory (Swap) --
CREATE TABLE IF NOT EXISTS dodo_order_history AS TEMPLATE_LOG
COMMENT 'DODO V2 OrderHistory (swap) events';
ALTER TABLE dodo_order_history
    ADD COLUMN IF NOT EXISTS from_token         String COMMENT 'Input token address',
    ADD COLUMN IF NOT EXISTS to_token           String COMMENT 'Output token address',
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS from_amount        UInt256 COMMENT 'Input token amount',
    ADD COLUMN IF NOT EXISTS return_amount      UInt256 COMMENT 'Output token amount';
