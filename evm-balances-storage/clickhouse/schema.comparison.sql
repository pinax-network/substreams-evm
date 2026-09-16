-- Use a dedicated comparison database: these names match the live sink's
-- interface, but retain block history instead of replacing every holder row.
-- Finalized input only. This schema does not implement fork rollback.
CREATE TABLE IF NOT EXISTS blocks (
    block_num UInt64,
    block_hash String,
    timestamp DateTime('UTC')
) ENGINE = ReplacingMergeTree
ORDER BY (block_num, block_hash);

CREATE TABLE IF NOT EXISTS native_balances (
    block_num UInt64,
    block_hash String,
    timestamp DateTime('UTC'),
    address String,
    balance UInt256
) ENGINE = ReplacingMergeTree
ORDER BY (block_num, block_hash, address);

CREATE TABLE IF NOT EXISTS erc20_balances (
    block_num UInt64,
    block_hash String,
    timestamp DateTime('UTC'),
    contract String,
    address String,
    balance UInt256
) ENGINE = ReplacingMergeTree
ORDER BY (block_num, block_hash, contract, address);
