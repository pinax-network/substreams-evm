INSERT INTO state_pools_aggregating_by_pool
SELECT
    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num,

    -- DEX identity
    protocol, factory, pool,

    -- universal --
    uniqState(user) AS uaw,
    count() as transactions
FROM swaps
GROUP BY protocol, factory, pool;