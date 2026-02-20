-- SAI Mint events --
CREATE TABLE IF NOT EXISTS sai_mint AS TEMPLATE_LOG
COMMENT 'SAI Mint events';
ALTER TABLE sai_mint
    -- event --
    ADD COLUMN IF NOT EXISTS guy         String,
    ADD COLUMN IF NOT EXISTS wad         UInt256;

-- SAI Burn events --
CREATE TABLE IF NOT EXISTS sai_burn AS TEMPLATE_LOG
COMMENT 'SAI Burn events';
ALTER TABLE sai_burn
    -- event --
    ADD COLUMN IF NOT EXISTS guy         String,
    ADD COLUMN IF NOT EXISTS wad         UInt256;
