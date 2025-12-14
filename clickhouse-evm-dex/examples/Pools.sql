-- Query By pool --
EXPLAIN indexes = 1
WITH
    filter_by_pools AS (
        SELECT DISTINCT pool
        FROM state_pools_aggregating_by_token
        WHERE token = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2'
    )
SELECT
    p.*,
    pt.tokens,
    pt.uaw,
FROM pools AS p
INNER JOIN filter_by_pools AS f ON f.pool = p.pool
LEFT JOIN pools_tokens AS pt ON pt.pool = p.pool
ORDER BY p.transactions DESC
LIMIT 5;