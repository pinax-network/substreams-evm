-- ERC-20 & Native balances --
-- There can only be a single ERC-20 balance change per block for a given address / contract pair --
CREATE TABLE IF NOT EXISTS erc20_balances (
    -- block --
    block_num            UInt32,
    block_hash           String,
    timestamp            DateTime(0, 'UTC'),
    minute               UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- balance --
    contract            String COMMENT 'token contract address',
    address             String COMMENT 'token holder address',
    balance             UInt256 COMMENT 'token balance',

    -- indexes --
    INDEX idx_balance (balance) TYPE minmax,

    -- projections --
    PROJECTION prj_address_contract (SELECT * ORDER BY (address, contract))
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (contract, address)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'ERC-20 & Native balance changes per block for a given address / contract pair';

-- ERC-20 & Native balances --
-- There can only be a single ERC-20 balance change per block for a given address / contract pair --
CREATE TABLE IF NOT EXISTS native_balances (
    -- block --
    block_num            UInt32,
    block_hash           String,
    timestamp            DateTime(0, 'UTC'),
    minute               UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- balance --
    address             String COMMENT 'token holder address',
    balance             UInt256 COMMENT 'token balance',

    -- indexes --
    INDEX idx_balance (balance) TYPE minmax
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (address)
COMMENT 'Native balance changes per block for a given address';

