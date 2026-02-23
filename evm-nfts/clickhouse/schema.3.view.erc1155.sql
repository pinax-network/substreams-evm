CREATE TABLE IF NOT EXISTS erc1155_balances (
    -- block --
    block_num            SimpleAggregateFunction(max, UInt32),
    timestamp            SimpleAggregateFunction(max, DateTime(0, 'UTC')),

    -- balance --
    contract             String,
    token_id             UInt256,
    owner                String,
    balance              SimpleAggregateFunction(sum, Int256)
)
ENGINE = AggregatingMergeTree
ORDER BY (contract, token_id, owner);

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_erc1155_balance_to
TO erc1155_balances
AS
SELECT
    block_num,
    timestamp,
    log_address AS contract,
    token_id,
    `to` AS owner,
    CAST(amount, 'Int256') AS balance
FROM erc1155_transfers;

CREATE MATERIALIZED VIEW IF NOT EXISTS  mv_erc1155_balance_from
TO erc1155_balances
AS
SELECT
    block_num,
    timestamp,
    log_address AS contract,
    token_id,
    `from` AS owner,
    -CAST(amount, 'Int256') as balance
FROM erc1155_transfers;
