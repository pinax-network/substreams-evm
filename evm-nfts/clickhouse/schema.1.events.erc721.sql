-- ERC721 Transfers --
CREATE TABLE IF NOT EXISTS erc721_transfers AS TEMPLATE_LOG
COMMENT 'ERC721 Transfer events';
ALTER TABLE erc721_transfers
    -- event --
    ADD COLUMN IF NOT EXISTS operator             String DEFAULT '',
    ADD COLUMN IF NOT EXISTS `from`               String,
    ADD COLUMN IF NOT EXISTS `to`                  String,
    ADD COLUMN IF NOT EXISTS token_id             UInt256,
    ADD COLUMN IF NOT EXISTS amount               UInt256 DEFAULT 1,
    ADD COLUMN IF NOT EXISTS transfer_type        LowCardinality(String),
    ADD COLUMN IF NOT EXISTS token_standard       LowCardinality(String);

-- ERC721 Approval --
CREATE TABLE IF NOT EXISTS erc721_approvals AS TEMPLATE_LOG
COMMENT 'ERC721 Approval events';
ALTER TABLE erc721_approvals
    -- event --
    ADD COLUMN IF NOT EXISTS owner                String,
    ADD COLUMN IF NOT EXISTS approved             String,
    ADD COLUMN IF NOT EXISTS token_id             UInt256;

-- ERC721 Approval For All --
CREATE TABLE IF NOT EXISTS erc721_approvals_for_all AS TEMPLATE_LOG
COMMENT 'ERC721 Approval For All events';
ALTER TABLE erc721_approvals_for_all
    -- event --
    ADD COLUMN IF NOT EXISTS owner                String,
    ADD COLUMN IF NOT EXISTS operator             String,
    ADD COLUMN IF NOT EXISTS approved             Bool,
    ADD COLUMN IF NOT EXISTS token_standard       LowCardinality(String);
