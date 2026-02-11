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
CREATE TABLE IF NOT EXISTS supply (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- supply --
    contract             TEXT NOT NULL PRIMARY KEY,
    total_supply         NUMERIC NOT NULL,
    max_supply           NUMERIC
);

-- Block indexes
CREATE INDEX IF NOT EXISTS idx_supply_block_num ON supply (block_num);
CREATE INDEX IF NOT EXISTS idx_supply_timestamp ON supply (timestamp);

-- Single column indexes
CREATE INDEX IF NOT EXISTS idx_supply_total_supply ON supply (total_supply);
CREATE INDEX IF NOT EXISTS idx_supply_max_supply ON supply (max_supply);

-- Sorted indexes for top/bottom supply per contract
CREATE INDEX IF NOT EXISTS idx_supply_total_supply_desc ON supply (total_supply DESC) WHERE total_supply != 0;
CREATE INDEX IF NOT EXISTS idx_supply_total_supply_asc ON supply (total_supply ASC) WHERE total_supply != 0;

-- Upsert rule for supply
CREATE OR REPLACE RULE supply_upsert AS ON INSERT TO supply
WHERE EXISTS (SELECT 1 FROM supply WHERE contract = NEW.contract)
DO INSTEAD UPDATE supply SET
    block_num = NEW.block_num,
    block_hash = NEW.block_hash,
    timestamp = NEW.timestamp,
    total_supply = NEW.total_supply,
    max_supply = COALESCE(NEW.max_supply, supply.max_supply)
WHERE contract = NEW.contract;
