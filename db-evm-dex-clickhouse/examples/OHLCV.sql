-- OHLCV candles for a specific pool (1 minute interval) --
SELECT
    timestamp,
    interval_min,
    pool,
    protocol,
    token0,
    token1,
    open0,
    high_quantile0,
    low_quantile0,
    close0,
    gross_volume0,
    gross_volume1,
    transactions
FROM ohlc_prices
WHERE
    pool = '0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640' AND
    interval_min = 1
ORDER BY timestamp DESC
LIMIT 100;

-- OHLCV candles for a token pair (1 hour interval) --
SELECT
    timestamp,
    interval_min,
    pool,
    protocol,
    token0,
    token1,
    open0,
    high_quantile0,
    low_quantile0,
    close0,
    gross_volume0,
    gross_volume1,
    net_flow0,
    net_flow1,
    transactions
FROM ohlc_prices
WHERE
    token0 = '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48' AND
    token1 = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2' AND
    interval_min = 60
ORDER BY timestamp DESC
LIMIT 100;

-- OHLCV candles filtered by protocol (1 day interval) --
SELECT
    timestamp,
    pool,
    token0,
    token1,
    open0,
    high_quantile0,
    low_quantile0,
    close0,
    gross_volume0,
    gross_volume1,
    transactions
FROM ohlc_prices
WHERE
    protocol = 'uniswap_v3' AND
    interval_min = 1440
ORDER BY timestamp DESC
LIMIT 100;

-- top pools by volume over the last 24 hours (1 day interval) --
SELECT
    pool,
    protocol,
    token0,
    token1,
    gross_volume0,
    gross_volume1,
    transactions
FROM ohlc_prices
WHERE
    interval_min = 1440 AND
    timestamp >= now() - INTERVAL 1 DAY
ORDER BY transactions DESC
LIMIT 20;
