CREATE VIEW IF NOT EXISTS pools AS
SELECT
    -- transactions (summing) --
    s.factory        AS factory,
    s.pool           AS pool,
    s.protocol       AS protocol,
    s.min_timestamp  AS min_timestamp,
    s.max_timestamp  AS max_timestamp,
    s.min_block_num  AS min_block_num,
    s.max_block_num  AS max_block_num,
    s.transactions   AS transactions,

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
    ) AS tokens

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

SETTINGS allow_experimental_correlated_subqueries = 1, join_use_nulls = 1;
