CREATE VIEW IF NOT EXISTS pools AS
SELECT
    -- transactions (summing) --
    s.factory       AS factory,
    s.pool          AS pool,
    s.protocol      AS protocol,
    s.transactions  AS transactions,

    -- initialize (required) --
    i.block_num     AS initialize_block_num,
    i.timestamp     AS initialize_timestamp,
    i.tx_hash       AS initialize_tx_hash,

    -- fees (optional) --
    f.fee           AS fee,
    f.block_num     AS last_fee_block_num,
    f.timestamp     AS last_fee_timestamp,
    f.tx_hash       AS last_fee_tx_hash,

    -- tokens (required >=2 token) --
    (
        SELECT arraySort(groupArrayDistinct(token))
        FROM state_pools_tokens
        WHERE factory = s.factory AND pool = s.pool AND protocol = s.protocol
    ) AS tokens,
    t.block_num     AS last_activity_block_num,
    t.timestamp     AS last_activity_timestamp,
    t.tx_hash       AS last_activity_tx_hash

-- transactions (summing) --
FROM state_pool_activity_summary AS s

-- initialize (required) --
JOIN state_pools_initialize AS i
    ON s.pool = i.pool
   AND s.factory = i.factory
   AND s.protocol = i.protocol

-- fees (optional) --
LEFT JOIN state_pools_fees AS f
    ON s.pool = f.pool
   AND s.factory = f.factory
   AND s.protocol = f.protocol

LEFT ANY JOIN state_pools_tokens AS t
    ON s.pool = t.pool
   AND s.factory = t.factory
   AND s.protocol = t.protocol

-- must have at least 2 tokens --
WHERE length(tokens) >= 2

SETTINGS allow_experimental_correlated_subqueries = 1, join_use_nulls = 1;
