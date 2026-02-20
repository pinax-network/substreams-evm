-- USDT Issue events --
CREATE TABLE IF NOT EXISTS usdt_issue AS TEMPLATE_LOG
COMMENT 'USDT Issue events';
ALTER TABLE usdt_issue
    -- event --
    ADD COLUMN IF NOT EXISTS owner        String,
    ADD COLUMN IF NOT EXISTS amount       UInt256;

-- USDT Redeem events --
CREATE TABLE IF NOT EXISTS usdt_redeem AS TEMPLATE_LOG
COMMENT 'USDT Redeem events';
ALTER TABLE usdt_redeem
    -- event --
    ADD COLUMN IF NOT EXISTS owner        String,
    ADD COLUMN IF NOT EXISTS amount       UInt256;

-- USDT DestroyedBlackFunds events --
CREATE TABLE IF NOT EXISTS usdt_destroyed_black_funds AS TEMPLATE_LOG
COMMENT 'USDT DestroyedBlackFunds events';
ALTER TABLE usdt_destroyed_black_funds
    -- event --
    ADD COLUMN IF NOT EXISTS black_listed_user  String,
    ADD COLUMN IF NOT EXISTS balance            UInt256;

-- USDT BlockPlaced events (v0.8.4) --
CREATE TABLE IF NOT EXISTS usdt_block_placed AS TEMPLATE_LOG
COMMENT 'USDT BlockPlaced events';
ALTER TABLE usdt_block_placed
    -- event --
    ADD COLUMN IF NOT EXISTS user  String;

-- USDT BlockReleased events (v0.8.4) --
CREATE TABLE IF NOT EXISTS usdt_block_released AS TEMPLATE_LOG
COMMENT 'USDT BlockReleased events';
ALTER TABLE usdt_block_released
    -- event --
    ADD COLUMN IF NOT EXISTS user  String;

-- USDT Mint events (v0.8.4) --
CREATE TABLE IF NOT EXISTS usdt_mint AS TEMPLATE_LOG
COMMENT 'USDT Mint events';
ALTER TABLE usdt_mint
    -- event --
    ADD COLUMN IF NOT EXISTS destination  String,
    ADD COLUMN IF NOT EXISTS amount       UInt256;

-- USDT DestroyedBlockedFunds events (v0.8.4) --
CREATE TABLE IF NOT EXISTS usdt_destroyed_blocked_funds AS TEMPLATE_LOG
COMMENT 'USDT DestroyedBlockedFunds events';
ALTER TABLE usdt_destroyed_blocked_funds
    -- event --
    ADD COLUMN IF NOT EXISTS blocked_user  String,
    ADD COLUMN IF NOT EXISTS balance       UInt256;

-- USDT NewPrivilegedContract events (v0.8.4) --
CREATE TABLE IF NOT EXISTS usdt_new_privileged_contract AS TEMPLATE_LOG
COMMENT 'USDT NewPrivilegedContract events';
ALTER TABLE usdt_new_privileged_contract
    -- event --
    ADD COLUMN IF NOT EXISTS contract  String;

-- USDT RemovedPrivilegedContract events (v0.8.4) --
CREATE TABLE IF NOT EXISTS usdt_removed_privileged_contract AS TEMPLATE_LOG
COMMENT 'USDT RemovedPrivilegedContract events';
ALTER TABLE usdt_removed_privileged_contract
    -- event --
    ADD COLUMN IF NOT EXISTS contract  String;

-- USDT LogSwapin events (swap_asset) --
CREATE TABLE IF NOT EXISTS usdt_log_swapin AS TEMPLATE_LOG
COMMENT 'USDT LogSwapin events';
ALTER TABLE usdt_log_swapin
    -- event --
    ADD COLUMN IF NOT EXISTS txhash   String,
    ADD COLUMN IF NOT EXISTS account  String,
    ADD COLUMN IF NOT EXISTS amount   UInt256;

-- USDT LogSwapout events (swap_asset) --
CREATE TABLE IF NOT EXISTS usdt_log_swapout AS TEMPLATE_LOG
COMMENT 'USDT LogSwapout events';
ALTER TABLE usdt_log_swapout
    -- event --
    ADD COLUMN IF NOT EXISTS account   String,
    ADD COLUMN IF NOT EXISTS bindaddr  String,
    ADD COLUMN IF NOT EXISTS amount    UInt256;

-- USDT LogChangeDCRMOwner events (swap_asset) --
CREATE TABLE IF NOT EXISTS usdt_log_change_dcrm_owner AS TEMPLATE_LOG
COMMENT 'USDT LogChangeDCRMOwner events';
ALTER TABLE usdt_log_change_dcrm_owner
    -- event --
    ADD COLUMN IF NOT EXISTS old_owner         String,
    ADD COLUMN IF NOT EXISTS new_owner         String,
    ADD COLUMN IF NOT EXISTS effective_height  UInt256;
