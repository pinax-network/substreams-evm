-- ERC-20 & Native balances --
-- There can only be a single ERC-20 balance change per block for a given address / contract pair --
CREATE TABLE IF NOT EXISTS erc20_balances (
    -- block --
    block_num            UInt32,
    block_hash           String,
    timestamp            DateTime(0, 'UTC'),

    -- balance --
    contract            String COMMENT 'token contract address',
    address             String COMMENT 'token holder address',
    balance             UInt256 COMMENT 'token balance',

    -- projections --
    PROJECTION prj_contract_balance_address (SELECT * ORDER BY (contract, balance, address));
    PROJECTION prj_contract_address (SELECT * ORDER BY (contract, address, balance));
    PROJECTION prj_account_contract (SELECT * ORDER BY (address, contract, balance));
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (contract, address)
COMMENT 'ERC-20 & Native balance changes per block for a given address / contract pair'
SETTING deduplicate_merge_projection_mode = 'rebuild';


-- ERC-20 & Native balances --
-- There can only be a single ERC-20 balance change per block for a given address / contract pair --
CREATE TABLE IF NOT EXISTS native_balances (
    -- block --
    block_num            UInt32,
    block_hash           String,
    timestamp            DateTime(0, 'UTC'),

    -- balance --
    address             String COMMENT 'token holder address',
    balance             UInt256 COMMENT 'token balance',

    -- projections --
    PROJECTION prj_account (SELECT * ORDER BY (address, balance));
)
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (address)
COMMENT 'Native balance changes per block for a given address'
SETTING deduplicate_merge_projection_mode = 'rebuild';

