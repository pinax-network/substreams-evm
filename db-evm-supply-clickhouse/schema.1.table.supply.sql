-- ERC-20 token supply --
-- There can only be a single supply change per block for a given contract --
CREATE TABLE IF NOT EXISTS supply (
    -- block --
    block_num            UInt32,
    block_hash           String,
    timestamp            DateTime(0, 'UTC'),
    minute               UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- supply --
    contract            String COMMENT 'token contract address',
    total_supply        UInt256 COMMENT 'totalSupply()',
    max_supply          Nullable(UInt256) COMMENT 'maxSupply() or cap()',

    -- indexes --
    INDEX idx_total_supply (total_supply) TYPE minmax GRANULARITY 1,

    -- count() --
    PROJECTION prj_contract_count ( SELECT contract, min(total_supply), max(total_supply), count(), max(block_num), min(block_num), max(timestamp), min(timestamp), max(minute), min(minute) GROUP BY contract )
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (contract)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'ERC-20 token supply changes per block for a given contract';
