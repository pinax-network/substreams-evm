-- This file is generated. Do not edit.

-- Blocks table for PostgreSQL
CREATE TABLE IF NOT EXISTS blocks (
    block_num            INTEGER PRIMARY KEY,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_block_hash ON blocks (block_hash);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks (timestamp);


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

CREATE INDEX IF NOT EXISTS idx_erc20_balances_address ON erc20_balances (address);
CREATE INDEX IF NOT EXISTS idx_erc20_balances_block_num ON erc20_balances (block_num);
CREATE INDEX IF NOT EXISTS idx_erc20_balances_balance ON erc20_balances (balance);

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

CREATE INDEX IF NOT EXISTS idx_native_balances_block_num ON native_balances (block_num);
CREATE INDEX IF NOT EXISTS idx_native_balances_balance ON native_balances (balance);


