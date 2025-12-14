CREATE VIEW IF NOT EXISTS pools_tokens AS
SELECT
    pool,
    factory,
    protocol,
    arraySort(groupArrayDistinct(token)) as tokens,
    sum(transactions) as transactions,
    uniqMerge(uniq_user) as uniq_user,
    uniqMerge(uniq_tx_from) as uniq_tx_from,
    min(min_timestamp) as min_timestamp,
    max(max_timestamp) as max_timestamp,
    min(min_block_num) as min_block_num,
    max(max_block_num) as max_block_num
FROM state_pools_aggregating_by_token
GROUP BY pool, factory, protocol;

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
    f.tx_hash       AS last_fee_tx_hash

-- transactions (summing) --
FROM state_pools_aggregating_by_pool AS s

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

SETTINGS join_use_nulls = 1;
