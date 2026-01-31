-- ERC-20 balances table for PostgreSQL
-- There can only be a single ERC-20 balance per address / contract pair (latest balance)
CREATE TABLE IF NOT EXISTS erc20_balances (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- balance --
    contract             TEXT NOT NULL,
    address              TEXT NOT NULL,
    balance              NUMERIC NOT NULL,

    PRIMARY KEY (contract, address)
);

-- Block indexes
CREATE INDEX IF NOT EXISTS idx_erc20_balances_block_num ON erc20_balances (block_num);
CREATE INDEX IF NOT EXISTS idx_erc20_balances_timestamp ON erc20_balances (timestamp);

-- Single column indexes
CREATE INDEX IF NOT EXISTS idx_erc20_balances_address ON erc20_balances (address);
CREATE INDEX IF NOT EXISTS idx_erc20_balances_balance ON erc20_balances (balance);

-- Composite indexes (non-zero balances only)
CREATE INDEX IF NOT EXISTS idx_erc20_balances_nonzero ON erc20_balances (contract, address) WHERE balance != 0;
CREATE INDEX IF NOT EXISTS idx_erc20_balances_address_contract ON erc20_balances (address, contract) WHERE balance != 0;

-- Sorted indexes for top/bottom balances per contract
CREATE INDEX IF NOT EXISTS idx_erc20_balances_contract_balance_desc ON erc20_balances (contract, balance DESC) WHERE balance != 0;
CREATE INDEX IF NOT EXISTS idx_erc20_balances_contract_balance_asc ON erc20_balances (contract, balance ASC) WHERE balance != 0;


-- Native balances table for PostgreSQL
-- There can only be a single native balance per address (latest balance)
CREATE TABLE IF NOT EXISTS native_balances (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- balance --
    address              TEXT PRIMARY KEY,
    balance              NUMERIC NOT NULL
);

-- Block indexes
CREATE INDEX IF NOT EXISTS idx_native_balances_block_num ON native_balances (block_num);
CREATE INDEX IF NOT EXISTS idx_native_balances_timestamp ON native_balances (timestamp);

-- Single column indexes
CREATE INDEX IF NOT EXISTS idx_native_balances_balance ON native_balances (balance);

-- Partial indexes (non-zero balances only)
CREATE INDEX IF NOT EXISTS idx_native_balances_nonzero ON native_balances (address) WHERE balance != 0;

-- Sorted indexes for top/bottom balances
CREATE INDEX IF NOT EXISTS idx_native_balances_balance_desc ON native_balances (balance DESC) WHERE balance != 0;
CREATE INDEX IF NOT EXISTS idx_native_balances_balance_asc ON native_balances (balance ASC) WHERE balance != 0;
