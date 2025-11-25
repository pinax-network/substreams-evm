-- `/evm/swaps` by protocol grouped by hour (0.2s) --
WITH hours AS (
    SELECT round(minute/60) as hour
    FROM swaps
    WHERE protocol = 'uniswap-v2'
    GROUP BY hour
)
SELECT * FROM swaps
WHERE round(minute/60) IN hours AND protocol = 'uniswap-v2'
ORDER BY minute DESC
LIMIT 10;

-- `/evm/swaps` by user (0.2s) --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE user = '0x3328f7f4a1d1c57c35df56bbf0c9dcafca309c49'
    GROUP BY minute
)
SELECT * FROM swaps
WHERE minute IN minutes AND user = '0x3328f7f4a1d1c57c35df56bbf0c9dcafca309c49'
ORDER BY minute DESC
LIMIT 10;

-- `/evm/swaps` by pool (0.26s) --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE pool = '0xa478c2975ab1ea89e8196811f51a7b7ade33eb11'
    GROUP BY minute
)
SELECT * FROM swaps
WHERE minute IN minutes AND pool = '0xa478c2975ab1ea89e8196811f51a7b7ade33eb11'
ORDER BY minute DESC
LIMIT 10;
