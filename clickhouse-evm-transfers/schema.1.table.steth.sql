-- stETH TokenRebased events --
CREATE TABLE IF NOT EXISTS steth_token_rebased AS TEMPLATE_LOG
COMMENT 'stETH TokenRebased events';
ALTER TABLE steth_token_rebased
    -- event --
    ADD COLUMN IF NOT EXISTS report_timestamp       UInt256,
    ADD COLUMN IF NOT EXISTS time_elapsed           UInt256,
    ADD COLUMN IF NOT EXISTS pre_total_shares       UInt256,
    ADD COLUMN IF NOT EXISTS pre_total_ether        UInt256,
    ADD COLUMN IF NOT EXISTS post_total_shares      UInt256,
    ADD COLUMN IF NOT EXISTS post_total_ether       UInt256,
    ADD COLUMN IF NOT EXISTS shares_minted_as_fees  UInt256;

-- stETH SharesBurnt events --
CREATE TABLE IF NOT EXISTS steth_shares_burnt AS TEMPLATE_LOG
COMMENT 'stETH SharesBurnt events';
ALTER TABLE steth_shares_burnt
    -- event --
    ADD COLUMN IF NOT EXISTS account                    String,
    ADD COLUMN IF NOT EXISTS pre_rebase_token_amount    UInt256,
    ADD COLUMN IF NOT EXISTS post_rebase_token_amount   UInt256,
    ADD COLUMN IF NOT EXISTS shares_amount              UInt256;

-- stETH TransferShares events --
CREATE TABLE IF NOT EXISTS steth_transfer_shares AS TEMPLATE_LOG
COMMENT 'stETH TransferShares events';
ALTER TABLE steth_transfer_shares
    -- event --
    ADD COLUMN IF NOT EXISTS `from`       String,
    ADD COLUMN IF NOT EXISTS `to`         String,
    ADD COLUMN IF NOT EXISTS shares_value UInt256;

-- stETH ExternalSharesBurnt events --
CREATE TABLE IF NOT EXISTS steth_external_shares_burnt AS TEMPLATE_LOG
COMMENT 'stETH ExternalSharesBurnt events';
ALTER TABLE steth_external_shares_burnt
    -- event --
    ADD COLUMN IF NOT EXISTS amount_of_shares UInt256;
