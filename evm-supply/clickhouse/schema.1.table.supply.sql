-- ERC-20 token supply --
-- There can only be a single supply change per block for a given contract --
CREATE TABLE IF NOT EXISTS total_supply (
    -- block --
    block_num            UInt32,
    block_hash           String,
    timestamp            DateTime(0, 'UTC'),
    minute               UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- supply --
    contract            String COMMENT 'token contract address',
    amount              UInt256 COMMENT 'totalSupply()',

    -- indexes --
    INDEX idx_total_supply (amount) TYPE minmax GRANULARITY 1,

    -- count() --
    PROJECTION prj_contract_count ( SELECT contract, min(amount), max(amount), count(), max(block_num), min(block_num), max(timestamp), min(timestamp), max(minute), min(minute) GROUP BY contract )
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (contract)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'ERC-20 token supply changes per block for a given contract';
