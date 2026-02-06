-- ERC-20 Balances FINAL --
-- Refreshable materialized view that maintains deduplicated (FINAL) balances
-- Ordered by (contract, balance, address) to support efficient top/bottom holder queries per contract
CREATE MATERIALIZED VIEW IF NOT EXISTS erc20_balances_final
REFRESH EVERY 1 HOUR
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (contract, balance, address)
AS
SELECT *
FROM erc20_balances FINAL
WHERE balance > 0;

-- Native Balances FINAL --
-- Refreshable materialized view that maintains deduplicated (FINAL) balances
-- Ordered by (balance, address) to support efficient top/bottom holder queries
CREATE MATERIALIZED VIEW IF NOT EXISTS native_balances_final
REFRESH EVERY 1 HOUR
ENGINE = ReplacingMergeTree(block_num)
ORDER BY (balance, address)
AS
SELECT *
FROM native_balances FINAL
WHERE balance > 0;
