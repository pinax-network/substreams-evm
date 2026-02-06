-- WBTC Mint events --
CREATE TABLE IF NOT EXISTS wbtc_mint AS TEMPLATE_LOG
COMMENT 'WBTC Mint events';
ALTER TABLE wbtc_mint
    -- event --
    ADD COLUMN IF NOT EXISTS `to`        String,
    ADD COLUMN IF NOT EXISTS amount      UInt256;

-- WBTC Burn events --
CREATE TABLE IF NOT EXISTS wbtc_burn AS TEMPLATE_LOG
COMMENT 'WBTC Burn events';
ALTER TABLE wbtc_burn
    -- event --
    ADD COLUMN IF NOT EXISTS burner      String,
    ADD COLUMN IF NOT EXISTS value       UInt256;
