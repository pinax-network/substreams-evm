-- WOOFi WooSwap --
CREATE TABLE IF NOT EXISTS woofi_woo_swap AS TEMPLATE_LOG
COMMENT 'WOOFi WooSwap events';
ALTER TABLE woofi_woo_swap
    ADD COLUMN IF NOT EXISTS from_token         String COMMENT 'Input token address',
    ADD COLUMN IF NOT EXISTS to_token           String COMMENT 'Output token address',
    ADD COLUMN IF NOT EXISTS from_amount        UInt256 COMMENT 'Input token amount',
    ADD COLUMN IF NOT EXISTS to_amount          UInt256 COMMENT 'Output token amount',
    ADD COLUMN IF NOT EXISTS `from`             String COMMENT 'From address',
    ADD COLUMN IF NOT EXISTS `to`               String COMMENT 'To address',
    ADD COLUMN IF NOT EXISTS rebate_to          String COMMENT 'Rebate recipient address',
    ADD COLUMN IF NOT EXISTS swap_vol           UInt256 COMMENT 'Swap volume',
    ADD COLUMN IF NOT EXISTS swap_fee           UInt256 COMMENT 'Swap fee';
