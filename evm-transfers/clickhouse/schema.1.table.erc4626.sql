-- ERC-4626 Deposit Logs --
CREATE TABLE IF NOT EXISTS erc4626_deposit AS TEMPLATE_LOG
COMMENT 'ERC-4626 Vault Deposit events';
ALTER TABLE erc4626_deposit
    -- event --
    ADD COLUMN IF NOT EXISTS sender     String,
    ADD COLUMN IF NOT EXISTS owner      String,
    ADD COLUMN IF NOT EXISTS assets     UInt256,
    ADD COLUMN IF NOT EXISTS shares     UInt256;

-- ERC-4626 Withdraw Logs --
CREATE TABLE IF NOT EXISTS erc4626_withdraw AS TEMPLATE_LOG
COMMENT 'ERC-4626 Vault Withdraw events';
ALTER TABLE erc4626_withdraw
    -- event --
    ADD COLUMN IF NOT EXISTS sender     String,
    ADD COLUMN IF NOT EXISTS receiver   String,
    ADD COLUMN IF NOT EXISTS owner      String,
    ADD COLUMN IF NOT EXISTS assets     UInt256,
    ADD COLUMN IF NOT EXISTS shares     UInt256;
