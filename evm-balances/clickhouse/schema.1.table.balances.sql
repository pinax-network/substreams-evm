-- ERC-20 & Native balances --
-- There can only be a single ERC-20 balance change per block for a given address / contract pair --
CREATE TABLE IF NOT EXISTS erc20_balances (
    -- block --
    block_num            UInt32,
    block_hash           String EPHEMERAL,
    timestamp            DateTime(0, 'UTC'),
    minute               UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- balance --
    contract            String COMMENT 'token contract address',
    address             String COMMENT 'token holder address',
    balance             UInt256 COMMENT 'token balance',
    is_deleted          UInt8 MATERIALIZED balance = 0,

    -- indexes --
    INDEX idx_balance (balance) TYPE minmax GRANULARITY 1,

    -- count() --
    PROJECTION prj_contract_count ( SELECT contract, min(balance), max(balance), count(), max(block_num), min(block_num), max(timestamp), min(timestamp), max(minute), min(minute) GROUP BY contract ),

    -- projections --
    PROJECTION prj_address_contract ( SELECT * ORDER BY address, contract )
)
ENGINE = ReplacingMergeTree(block_num, is_deleted)
ORDER BY (contract, address)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'ERC-20 & Native balance changes per block for a given address / contract pair';

-- ERC-20 & Native balances --
-- There can only be a single ERC-20 balance change per block for a given address / contract pair --
CREATE TABLE IF NOT EXISTS native_balances (
    -- block --
    block_num            UInt32,
    block_hash           String EPHEMERAL,
    timestamp            DateTime(0, 'UTC'),

    -- balance --
    address             String COMMENT 'token holder address',
    balance             UInt256 COMMENT 'token balance',
    is_deleted          UInt8 MATERIALIZED balance = 0,

    -- indexes --
    INDEX idx_balance (balance) TYPE minmax GRANULARITY 1
)
ENGINE = ReplacingMergeTree(block_num, is_deleted)
ORDER BY (address)
COMMENT 'Native balance changes per block for a given address';

-- ERC-20 Token top holders: max 10,000 accounts per contract
CREATE MATERIALIZED VIEW IF NOT EXISTS erc20_balances_holders
REFRESH AFTER 60 MINUTE
ENGINE = MergeTree
ORDER BY (contract, balance DESC, address)
SETTINGS allow_experimental_reverse_key = 1
AS
SELECT
    block_num,
    timestamp,
    contract,
    address,
    balance
FROM erc20_balances FINAL
WHERE balance > 0
ORDER BY contract, balance DESC, address
LIMIT 10000 BY contract
SETTINGS max_threads = 4;

-- Native Token top holders: max 100,000 accounts
CREATE MATERIALIZED VIEW IF NOT EXISTS native_balances_holders
REFRESH AFTER 60 MINUTE
ENGINE = MergeTree
ORDER BY (balance DESC, address)
SETTINGS allow_experimental_reverse_key = 1
AS
SELECT
    block_num,
    timestamp,
    address,
    balance
FROM native_balances FINAL
WHERE balance > 0
ORDER BY balance DESC, address
LIMIT 100000
SETTINGS max_threads = 4;