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
WHERE interval_min = 1 AND pool = '0x97dec872013f6b5fb443861090ad931542878126' -- USDC-ETH on Uniswap V1
GROUP BY pool, timestamp
ORDER BY timestamp DESC
LIMIT 10;