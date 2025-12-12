CREATE VIEW IF NOT EXISTS pools AS
SELECT
    -- initialize --
    i.factory       AS factory,
    i.pool          AS pool,
    i.protocol      AS protocol,
    i.block_num     AS initialize_block_num,
    i.timestamp     AS initialize_timestamp,
    i.tx_hash       AS initialize_tx_hash,

    -- fees --
    f.fee           AS fee,
    f.block_num     AS fee_block_num,
    f.timestamp     AS fee_timestamp,
    f.tx_hash       AS fee_tx_hash,

    -- tokens (Array) --
    (
        SELECT arraySort(groupArrayDistinct(token))
        FROM state_pools_tokens
        WHERE factory = i.factory AND pool = i.pool AND protocol = i.protocol
    ) AS tokens
FROM state_pools_initialize AS i
-- fees (optional) --
LEFT JOIN state_pools_fees AS f
    ON i.protocol = f.protocol
   AND i.factory = f.factory
   AND i.pool = f.pool
WHERE length(tokens) >= 2
SETTINGS allow_experimental_correlated_subqueries = 1, join_use_nulls = 1;
