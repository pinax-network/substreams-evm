-- Blocks table for PostgreSQL
CREATE TABLE IF NOT EXISTS blocks (
    block_num            INTEGER PRIMARY KEY,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_block_hash ON blocks (block_hash);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks (timestamp);
