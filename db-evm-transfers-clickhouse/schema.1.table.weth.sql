-- WETH Deposit/Withdrawal Logs --
CREATE TABLE IF NOT EXISTS weth_deposit AS TEMPLATE_LOG
COMMENT 'WETH Deposit events';
ALTER TABLE weth_deposit
    -- event --
    ADD COLUMN IF NOT EXISTS dst        String,
    ADD COLUMN IF NOT EXISTS wad        UInt256;

-- WETH Withdrawal Logs --
CREATE TABLE IF NOT EXISTS weth_withdrawal AS TEMPLATE_LOG
COMMENT 'WETH Withdrawal events';
ALTER TABLE weth_withdrawal
    -- event --
    ADD COLUMN IF NOT EXISTS src        String,
    ADD COLUMN IF NOT EXISTS wad        UInt256;