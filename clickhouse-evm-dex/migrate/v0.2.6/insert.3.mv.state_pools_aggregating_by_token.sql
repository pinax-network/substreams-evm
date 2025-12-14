INSERT INTO state_pools_aggregating_by_token
SELECT
    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num,

    -- DEX identity
    protocol, factory, pool,
    input_contract AS token,

    -- universal --
    uniqState(user) AS uaw,
    count() as transactions
FROM swaps
GROUP BY token, protocol, factory, pool;

INSERT INTO state_pools_aggregating_by_token
SELECT
    -- timestamp & block number --
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num,

    -- DEX identity
    protocol, factory, pool,
    output_contract AS token,

    -- universal --
    uniqState(user) AS uaw,
    count() as transactions
FROM swaps
GROUP BY token, protocol, factory, pool;