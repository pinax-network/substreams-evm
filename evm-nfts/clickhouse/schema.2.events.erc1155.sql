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

-- Remove TTL (full history required) --
ALTER TABLE erc1155_transfers REMOVE TTL;

-- Projections --
-- count() --
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_transfer_type_count ( SELECT transfer_type, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY transfer_type );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_log_address ( SELECT log_address, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_from_count ( SELECT `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from` );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_to_count ( SELECT `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `to` );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_from_to_count ( SELECT `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from`, `to` );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_log_address_from_count ( SELECT log_address, `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `from` );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_log_address_to_count ( SELECT log_address, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `to` );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_log_address_to_from_count ( SELECT log_address, `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `from`, `to` );
-- minute + timestamp --
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp GROUP BY tx_hash, minute, timestamp );
-- minute --
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_transfer_type_by_minute ( SELECT transfer_type, minute GROUP BY transfer_type, minute );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_from_by_minute ( SELECT `from`, minute GROUP BY `from`, minute );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_to_by_minute ( SELECT `to`, minute GROUP BY `to`, minute );
ALTER TABLE erc1155_transfers ADD PROJECTION IF NOT EXISTS prj_log_address_by_minute ( SELECT log_address, minute GROUP BY log_address, minute );

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
