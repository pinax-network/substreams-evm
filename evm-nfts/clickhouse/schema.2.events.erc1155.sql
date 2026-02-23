-- ERC1155 Transfer Single & Batch --
CREATE TABLE IF NOT EXISTS erc1155_transfers AS TEMPLATE_LOG
COMMENT 'ERC1155 Transfer events';
ALTER TABLE erc1155_transfers
    -- event --
    ADD COLUMN IF NOT EXISTS operator             String DEFAULT '',
    ADD COLUMN IF NOT EXISTS `from`               String,
    ADD COLUMN IF NOT EXISTS `to`                  String,
    ADD COLUMN IF NOT EXISTS token_id             UInt256,
    ADD COLUMN IF NOT EXISTS amount               UInt256 DEFAULT 1,
    ADD COLUMN IF NOT EXISTS transfer_type        LowCardinality(String),
    ADD COLUMN IF NOT EXISTS token_standard       LowCardinality(String);

-- ERC1155 Approval For All --
CREATE TABLE IF NOT EXISTS erc1155_approvals_for_all AS TEMPLATE_LOG
COMMENT 'ERC1155 Approval For All events';
ALTER TABLE erc1155_approvals_for_all
    -- event --
    ADD COLUMN IF NOT EXISTS owner                String,
    ADD COLUMN IF NOT EXISTS operator             String,
    ADD COLUMN IF NOT EXISTS approved             Bool,
    ADD COLUMN IF NOT EXISTS token_standard       LowCardinality(String);

-- ERC1155 Token Metadata --
CREATE TABLE IF NOT EXISTS erc1155_metadata_by_token as erc721_metadata_by_token
ENGINE = ReplacingMergeTree(block_num)
PRIMARY KEY (contract, token_id)
ORDER BY (contract, token_id);

CREATE TABLE IF NOT EXISTS erc1155_metadata_by_contract as erc721_metadata_by_contract
ENGINE = ReplacingMergeTree(block_num)
PRIMARY KEY (contract)
ORDER BY (contract);
