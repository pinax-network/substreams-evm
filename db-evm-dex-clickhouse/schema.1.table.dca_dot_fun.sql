-- DCA.fun FillOrder (Swap execution) --
CREATE TABLE IF NOT EXISTS dca_dot_fun_fill_order AS TEMPLATE_LOG
COMMENT 'DCA.fun FillOrder (swap execution) events';
ALTER TABLE dca_dot_fun_fill_order
    ADD COLUMN IF NOT EXISTS order_id           UInt256 COMMENT 'Order ID',
    ADD COLUMN IF NOT EXISTS caller             String COMMENT 'Caller address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS fill_amount        UInt256 COMMENT 'Input token amount',
    ADD COLUMN IF NOT EXISTS amount_of_token_out UInt256 COMMENT 'Output token amount',
    ADD COLUMN IF NOT EXISTS protocol_fee       UInt256 COMMENT 'Protocol fee',
    ADD COLUMN IF NOT EXISTS token_in_price     UInt256 COMMENT 'Input token price',
    ADD COLUMN IF NOT EXISTS token_out_price    UInt256 COMMENT 'Output token price',
    ADD COLUMN IF NOT EXISTS scaling_factor     UInt256 COMMENT 'Scaling factor';

-- DCA.fun CreateOrder --
CREATE TABLE IF NOT EXISTS dca_dot_fun_create_order AS TEMPLATE_LOG
COMMENT 'DCA.fun CreateOrder events';
ALTER TABLE dca_dot_fun_create_order
    ADD COLUMN IF NOT EXISTS order_id           UInt256 COMMENT 'Order ID',
    ADD COLUMN IF NOT EXISTS creator            String COMMENT 'Creator address',
    ADD COLUMN IF NOT EXISTS recipient          String COMMENT 'Recipient address',
    ADD COLUMN IF NOT EXISTS token_in           String COMMENT 'Input token address',
    ADD COLUMN IF NOT EXISTS token_out          String COMMENT 'Output token address',
    ADD COLUMN IF NOT EXISTS spend_amount       UInt256 COMMENT 'Amount per DCA execution',
    ADD COLUMN IF NOT EXISTS repeats            UInt256 COMMENT 'Number of repeats',
    ADD COLUMN IF NOT EXISTS slippage           UInt256 COMMENT 'Slippage tolerance',
    ADD COLUMN IF NOT EXISTS freq_interval      UInt256 COMMENT 'Frequency interval',
    ADD COLUMN IF NOT EXISTS scaling_interval   UInt256 COMMENT 'Scaling interval',
    ADD COLUMN IF NOT EXISTS protocol_fee       UInt256 COMMENT 'Protocol fee',
    ADD COLUMN IF NOT EXISTS vault              String COMMENT 'Vault address',
    ADD COLUMN IF NOT EXISTS stake_asset_in     Bool COMMENT 'Stake input asset',
    ADD COLUMN IF NOT EXISTS stake_asset_out    Bool COMMENT 'Stake output asset';

-- DCA.fun CancelOrder --
CREATE TABLE IF NOT EXISTS dca_dot_fun_cancel_order AS TEMPLATE_LOG
COMMENT 'DCA.fun CancelOrder events';
ALTER TABLE dca_dot_fun_cancel_order
    ADD COLUMN IF NOT EXISTS order_id           UInt256 COMMENT 'Order ID',
    ADD COLUMN IF NOT EXISTS vault              String COMMENT 'Vault address';
