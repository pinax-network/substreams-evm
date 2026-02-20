-- Blocks table for PostgreSQL
CREATE TABLE IF NOT EXISTS blocks (
    block_num            INTEGER NOT NULL PRIMARY KEY,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_block_hash ON blocks (block_hash);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks (timestamp);

-- Upsert rule for blocks
CREATE OR REPLACE RULE blocks_upsert AS ON INSERT TO blocks
WHERE EXISTS (SELECT 1 FROM blocks WHERE block_num = NEW.block_num)
DO INSTEAD UPDATE blocks SET
    block_hash = NEW.block_hash,
    timestamp = NEW.timestamp
WHERE block_num = NEW.block_num;


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

-- Upsert rule for erc20_balances
CREATE OR REPLACE RULE erc20_balances_upsert AS ON INSERT TO erc20_balances
WHERE EXISTS (SELECT 1 FROM erc20_balances WHERE contract = NEW.contract AND address = NEW.address)
DO INSTEAD UPDATE erc20_balances SET
    block_num = NEW.block_num,
    block_hash = NEW.block_hash,
    timestamp = NEW.timestamp,
    balance = NEW.balance
WHERE contract = NEW.contract AND address = NEW.address;


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

-- Upsert rule for native_balances
CREATE OR REPLACE RULE native_balances_upsert AS ON INSERT TO native_balances
WHERE EXISTS (SELECT 1 FROM native_balances WHERE address = NEW.address)
DO INSTEAD UPDATE native_balances SET
    block_num = NEW.block_num,
    block_hash = NEW.block_hash,
    timestamp = NEW.timestamp,
    balance = NEW.balance
WHERE address = NEW.address;
