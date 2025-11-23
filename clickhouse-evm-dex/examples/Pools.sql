-- `/evm/pools` by pool --
SELECT
    protocol,
    factory,
    pool,
    token0,
    token1,
    count() as swaps,
    min(block_num) as min_block_num,
    min(timestamp) as min_timestamp,
    max(block_num) as max_block_num,
    max(timestamp) as max_timestamp
FROM swaps
GROUP BY
    protocol,
    factory,
    pool,
    token0,
    token1
ORDER BY swaps DESC
LIMIT 10;

-- `/evm/pools` by protocol/factory --
SELECT
    protocol,
    factory,
    count() as swaps,
    min(block_num) as min_block_num,
    min(timestamp) as min_timestamp,
    max(block_num) as max_block_num,
    max(timestamp) as max_timestamp
FROM swaps
GROUP BY
    protocol,
    factory
ORDER BY swaps DESC
LIMIT 10;