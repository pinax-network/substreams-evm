CREATE TABLE IF NOT EXISTS erc721_owners (
    -- block --
    block_num            UInt32,
    timestamp            DateTime(0, 'UTC'),

    -- owners --
    contract             String COMMENT 'contract address',
    token_id             UInt256,
    owner                String,

    -- indexes --
    INDEX idx_owner      (owner)    TYPE bloom_filter GRANULARITY 4
) ENGINE = ReplacingMergeTree(block_num)
ORDER BY (contract, token_id);

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_erc721_owners
TO erc721_owners
AS
SELECT
    -- block --
    block_num,
    timestamp,

    -- owners --
    log_address AS contract,
    token_id,
    `to` AS owner          -- current owner after this transfer
FROM erc721_transfers;
