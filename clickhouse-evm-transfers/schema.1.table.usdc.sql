-- USDC Mint events --
CREATE TABLE IF NOT EXISTS usdc_mint AS TEMPLATE_LOG
COMMENT 'USDC Mint events';
ALTER TABLE usdc_mint
    -- event --
    ADD COLUMN IF NOT EXISTS minter      String,
    ADD COLUMN IF NOT EXISTS `to`        String,
    ADD COLUMN IF NOT EXISTS amount      UInt256;

-- USDC Burn events --
CREATE TABLE IF NOT EXISTS usdc_burn AS TEMPLATE_LOG
COMMENT 'USDC Burn events';
ALTER TABLE usdc_burn
    -- event --
    ADD COLUMN IF NOT EXISTS burner      String,
    ADD COLUMN IF NOT EXISTS amount      UInt256;
