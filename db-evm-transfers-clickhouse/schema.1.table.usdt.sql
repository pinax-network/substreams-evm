-- USDT Issue events --
CREATE TABLE IF NOT EXISTS usdt_issue AS TEMPLATE_LOG
COMMENT 'USDT Issue events';
ALTER TABLE usdt_issue
    -- event --
    ADD COLUMN IF NOT EXISTS amount      UInt256;

-- USDT Redeem events --
CREATE TABLE IF NOT EXISTS usdt_redeem AS TEMPLATE_LOG
COMMENT 'USDT Redeem events';
ALTER TABLE usdt_redeem
    -- event --
    ADD COLUMN IF NOT EXISTS amount      UInt256;

-- USDT DestroyedBlackFunds events --
CREATE TABLE IF NOT EXISTS usdt_destroyed_black_funds AS TEMPLATE_LOG
COMMENT 'USDT DestroyedBlackFunds events';
ALTER TABLE usdt_destroyed_black_funds
    -- event --
    ADD COLUMN IF NOT EXISTS black_listed_user  String,
    ADD COLUMN IF NOT EXISTS balance            UInt256;
