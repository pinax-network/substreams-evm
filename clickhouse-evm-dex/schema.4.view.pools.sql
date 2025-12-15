CREATE VIEW IF NOT EXISTS pools_by_token AS
SELECT
    -- timestamp & block number --
    min(min_timestamp) as min_timestamp,
    max(max_timestamp) as max_timestamp,
    min(min_block_num) as min_block_num,
    max(max_block_num) as max_block_num,

    -- DEX identity --
    pool,
    factory,
    protocol,
    arraySort(groupArrayDistinct(token)) as tokens,

    -- universal --
    sum(transactions) as transactions,
    uniqMerge(uniq_user) as uniq_user,
    uniqMerge(uniq_tx_from) as uniq_tx_from
FROM state_pools_aggregating_by_token
GROUP BY pool, factory, protocol;

CREATE VIEW IF NOT EXISTS pools_by_pool AS
SELECT
    -- timestamp & block number --
    min(min_timestamp) as min_timestamp,
    max(max_timestamp) as max_timestamp,
    min(min_block_num) as min_block_num,
    max(max_block_num) as max_block_num,

    -- DEX identity --
    pool,
    factory,
    protocol,

    -- universal --
    sum(transactions) as transactions,
    uniqMerge(uniq_user) as uniq_user,
    uniqMerge(uniq_tx_from) as uniq_tx_from
FROM state_pools_aggregating_by_pool
GROUP BY pool, factory, protocol;
