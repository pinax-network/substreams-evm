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

-- ERC-20 token supply table for PostgreSQL
-- There can only be a single supply per contract (latest supply)
CREATE TABLE IF NOT EXISTS total_supply (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- supply --
    contract             TEXT NOT NULL PRIMARY KEY,
    amount               NUMERIC NOT NULL
);

-- Block indexes
CREATE INDEX IF NOT EXISTS idx_total_supply_block_num ON total_supply (block_num);
CREATE INDEX IF NOT EXISTS idx_total_supply_timestamp ON total_supply (timestamp);

-- Single column indexes
CREATE INDEX IF NOT EXISTS idx_total_supply_amount ON total_supply (amount);

-- Sorted indexes for top/bottom supply per contract
CREATE INDEX IF NOT EXISTS idx_total_supply_amount_desc ON total_supply (amount DESC) WHERE amount != 0;
CREATE INDEX IF NOT EXISTS idx_total_supply_amount_asc ON total_supply (amount ASC) WHERE amount != 0;

-- Upsert rule for total_supply
CREATE OR REPLACE RULE total_supply_upsert AS ON INSERT TO total_supply
WHERE EXISTS (SELECT 1 FROM total_supply WHERE contract = NEW.contract)
DO INSTEAD UPDATE total_supply SET
    block_num = NEW.block_num,
    block_hash = NEW.block_hash,
    timestamp = NEW.timestamp,
    amount = NEW.amount
WHERE contract = NEW.contract;
