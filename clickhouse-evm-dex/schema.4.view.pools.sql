CREATE VIEW IF NOT EXISTS pools AS
SELECT
    t.factory AS factory,
    t.pool AS pool,
    t.protocol AS protocol,
    t.tokens AS tokens,  -- Array of tokens

    -- initialize --
    i.block_num  AS initialize_block_num,
    i.timestamp  AS initialize_timestamp,
    i.tx_hash    AS initialize_tx_hash,

    -- fees --
    f.fee        AS fee,
    f.block_num  AS fee_block_num,
    f.timestamp  AS fee_timestamp,
    f.tx_hash    AS fee_tx_hash
FROM
(
    SELECT
        factory,
        pool,
        protocol,
        groupArrayDistinct(token) AS tokens
    FROM state_pools_tokens
    GROUP BY
        factory,
        pool,
        protocol
) AS t
LEFT JOIN state_pools_fees AS f
    ON t.pool = f.pool
   AND t.factory = f.factory
   AND t.protocol = f.protocol
LEFT JOIN state_pools_initialize AS i
    ON t.pool = i.pool
   AND t.factory = i.factory
   AND t.protocol = i.protocol
SETTINGS join_use_nulls = 1;
