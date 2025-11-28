-- Detect time intervals --
SELECT
    interval_min,
    count() AS intervals_count
FROM ohlc_prices
GROUP BY interval_min
ORDER BY interval_min;

-- OHLC Prices by Pool --
WITH (
      pow(10, 18) AS scale0,
      pow(10, 6) AS scale1,
      pow(10, 18-6) AS scale,
      2 AS precision -- user defined
) SELECT
      timestamp,
      'ETH/USDC' AS ticker,

      -- OHLC --
      floor(argMinMerge(open0) * scale, precision)                        AS open,
      floor(quantileDeterministicMerge(0.99)(quantile0) * scale, precision)   AS high,
      floor(quantileDeterministicMerge(0.01)(quantile0) * scale, precision)    AS low,
      floor(argMaxMerge(close0) * scale, precision)                       AS close,

      -- volume --
      floor(sum(gross_volume0) / scale0, precision)         AS "gross volume (ETH)",
      floor(sum(gross_volume1) / scale1, precision)         AS "gross volume (USDC)",
      floor(sum(net_flow0) / scale0, precision)             AS "net flow (ETH)",
      floor(sum(net_flow1) / scale1, precision)             AS "net flow (USDC)",

      -- universal --
      uniqMerge(uaw)          AS uaw,
      sum(transactions)       AS transactions
FROM ohlc_prices
WHERE interval_min = 1 AND pool = '0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36' -- USDC-ETH on Uniswap V3
GROUP BY pool, timestamp
ORDER BY timestamp DESC
LIMIT 10;

-- 0x4e68Ccd3E89f51C3074ca5072bbAC773960dFa36 USDT-ETH V3
-- 0x72331fcb696b0151904c03584b66dc8365bc63f8a144d89a773384e3a579ca73 USDT-ETH V4

-- Check if Token0/Token1 have no duplicates --
SELECT token0, token1, count()
FROM ohlc_prices
WHERE pool = lower('0x72331fcb696b0151904c03584b66dc8365bc63f8a144d89a773384e3a579ca73')
GROUP BY pool, token0, token1
LIMIT 10;

-- Check no duplicates in bulk
WITH pools AS (
      SELECT pool, count() as count
      FROM ohlc_prices
      WHERE interval_min = 1
      GROUP BY pool, token0, token1
)
SELECT pool, count()
FROM pools
GROUP BY pool
ORDER BY count() DESC
LIMIT 20;