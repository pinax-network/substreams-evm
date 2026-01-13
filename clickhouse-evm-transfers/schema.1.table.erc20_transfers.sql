-- ERC20 Transfer Logs --
CREATE TABLE IF NOT EXISTS erc20_transfers AS TEMPLATE_LOG
COMMENT 'ERC20 Token Transfer events';
ALTER TABLE erc20_transfers
    -- transfer --
    ADD COLUMN IF NOT EXISTS `from`        String,
    ADD COLUMN IF NOT EXISTS `to`          String,
    ADD COLUMN IF NOT EXISTS amount        UInt256;

-- ERC20 Approvals Logs --
CREATE TABLE IF NOT EXISTS erc20_approvals AS TEMPLATE_LOG
COMMENT 'ERC20 Token Approvals events';
ALTER TABLE erc20_approvals
    -- Approval --
    ADD COLUMN IF NOT EXISTS owner        String,
    ADD COLUMN IF NOT EXISTS spender      String,
    ADD COLUMN IF NOT EXISTS value        UInt256;