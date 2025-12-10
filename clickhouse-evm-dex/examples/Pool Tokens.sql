-- Get all token pairs for a specific pool --
SELECT
    pool,
    factory,
    protocol,
    input_token,
    output_token,
    transactions
FROM state_pools_tokens
WHERE pool = '0xa478c2975ab1ea89e8196811f51a7b7ade33eb11'
ORDER BY transactions DESC;

-- Get top pools by transaction count for a specific token pair --
SELECT
    pool,
    factory,
    protocol,
    input_token,
    output_token,
    transactions
FROM state_pools_tokens
WHERE input_token = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2'
  AND output_token = '0xdac17f958d2ee523a2206206994597c13d831ec7'
ORDER BY transactions DESC
LIMIT 10;

-- Get all token combinations in a pool (useful for multi-token pools) --
SELECT
    input_token,
    output_token,
    sum(transactions) as total_swaps
FROM state_pools_tokens
WHERE pool = '0xa478c2975ab1ea89e8196811f51a7b7ade33eb11'
GROUP BY input_token, output_token
ORDER BY total_swaps DESC;
