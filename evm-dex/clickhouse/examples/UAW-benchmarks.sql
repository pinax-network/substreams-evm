-- UAW benchmark queries for issue #186
-- Run these after ClickHouse is fully deployed and the DEX dataset is synced.
-- Recommended process:
--   1. run each query once to warm caches
--   2. run it again and record elapsed / rows read / bytes read
--   3. compare raw swaps vs specialized UAW tables vs unified state_pools_uaw

-- -----------------------------------------------------------------------------
-- RAW `swaps` BASELINE
-- -----------------------------------------------------------------------------

-- unique tx_from by factory
EXPLAIN indexes = 1, projections = 1
SELECT
    factory,
    uniqExact(tx_from) AS uaw_tx_from
FROM swaps
GROUP BY factory
ORDER BY uaw_tx_from DESC
LIMIT 50;

SELECT
    factory,
    uniqExact(tx_from) AS uaw_tx_from
FROM swaps
GROUP BY factory
ORDER BY uaw_tx_from DESC
LIMIT 50;

-- unique user by pool + factory
EXPLAIN indexes = 1, projections = 1
SELECT
    pool,
    factory,
    uniqExact(user) AS uaw_user
FROM swaps
GROUP BY pool, factory
ORDER BY uaw_user DESC
LIMIT 50;

SELECT
    pool,
    factory,
    uniqExact(user) AS uaw_user
FROM swaps
GROUP BY pool, factory
ORDER BY uaw_user DESC
LIMIT 50;

-- unique caller by protocol + factory
EXPLAIN indexes = 1, projections = 1
SELECT
    protocol,
    factory,
    uniqExact(caller) AS uaw_caller
FROM swaps
GROUP BY protocol, factory
ORDER BY uaw_caller DESC
LIMIT 50;

SELECT
    protocol,
    factory,
    uniqExact(caller) AS uaw_caller
FROM swaps
GROUP BY protocol, factory
ORDER BY uaw_caller DESC
LIMIT 50;

-- -----------------------------------------------------------------------------
-- SPECIALIZED UAW TABLES
-- -----------------------------------------------------------------------------

-- tx_from distinct counts by factory
EXPLAIN indexes = 1, projections = 1
SELECT
    factory,
    count() AS uaw_tx_from
FROM state_pools_uaw_by_tx_from
GROUP BY factory
ORDER BY uaw_tx_from DESC
LIMIT 50;

SELECT
    factory,
    count() AS uaw_tx_from
FROM state_pools_uaw_by_tx_from
GROUP BY factory
ORDER BY uaw_tx_from DESC
LIMIT 50;

-- tx_from distinct counts by pool + factory
EXPLAIN indexes = 1, projections = 1
SELECT
    pool,
    factory,
    count() AS uaw_tx_from
FROM state_pools_uaw_by_tx_from
GROUP BY pool, factory
ORDER BY uaw_tx_from DESC
LIMIT 50;

SELECT
    pool,
    factory,
    count() AS uaw_tx_from
FROM state_pools_uaw_by_tx_from
GROUP BY pool, factory
ORDER BY uaw_tx_from DESC
LIMIT 50;

-- user distinct counts by factory
EXPLAIN indexes = 1, projections = 1
SELECT
    factory,
    count() AS uaw_user
FROM state_pools_uaw_by_user
GROUP BY factory
ORDER BY uaw_user DESC
LIMIT 50;

SELECT
    factory,
    count() AS uaw_user
FROM state_pools_uaw_by_user
GROUP BY factory
ORDER BY uaw_user DESC
LIMIT 50;

-- caller distinct counts by pool + factory
EXPLAIN indexes = 1, projections = 1
SELECT
    pool,
    factory,
    count() AS uaw_caller
FROM state_pools_uaw_by_caller
GROUP BY pool, factory
ORDER BY uaw_caller DESC
LIMIT 50;

SELECT
    pool,
    factory,
    count() AS uaw_caller
FROM state_pools_uaw_by_caller
GROUP BY pool, factory
ORDER BY uaw_caller DESC
LIMIT 50;

-- -----------------------------------------------------------------------------
-- UNIFIED UAW TABLE
-- -----------------------------------------------------------------------------

-- all dimensions by factory
EXPLAIN indexes = 1, projections = 1
SELECT
    dimension,
    factory,
    count() AS unique_addresses
FROM state_pools_uaw
GROUP BY dimension, factory
ORDER BY dimension, unique_addresses DESC
LIMIT 100;

SELECT
    dimension,
    factory,
    count() AS unique_addresses
FROM state_pools_uaw
GROUP BY dimension, factory
ORDER BY dimension, unique_addresses DESC
LIMIT 100;

-- all dimensions by pool + factory
EXPLAIN indexes = 1, projections = 1
SELECT
    dimension,
    pool,
    factory,
    count() AS unique_addresses
FROM state_pools_uaw
GROUP BY dimension, pool, factory
ORDER BY dimension, unique_addresses DESC
LIMIT 100;

SELECT
    dimension,
    pool,
    factory,
    count() AS unique_addresses
FROM state_pools_uaw
GROUP BY dimension, pool, factory
ORDER BY dimension, unique_addresses DESC
LIMIT 100;

-- focused benchmark for caller only
EXPLAIN indexes = 1, projections = 1
SELECT
    dimension,
    factory,
    count() AS unique_addresses
FROM state_pools_uaw
WHERE dimension = 'caller'
GROUP BY dimension, factory
ORDER BY unique_addresses DESC
LIMIT 50;

SELECT
    dimension,
    factory,
    count() AS unique_addresses
FROM state_pools_uaw
WHERE dimension = 'caller'
GROUP BY dimension, factory
ORDER BY unique_addresses DESC
LIMIT 50;

-- -----------------------------------------------------------------------------
-- OHLC UAW SURFACE
-- -----------------------------------------------------------------------------

EXPLAIN indexes = 1, projections = 1
SELECT
    interval_min,
    pool,
    factory,
    uaw_tx_from,
    uaw_user,
    uaw_caller
FROM ohlc_prices_uaw
WHERE interval_min = 60
ORDER BY timestamp DESC
LIMIT 50;

SELECT
    interval_min,
    pool,
    factory,
    uaw_tx_from,
    uaw_user,
    uaw_caller
FROM ohlc_prices_uaw
WHERE interval_min = 60
ORDER BY timestamp DESC
LIMIT 50;

-- -----------------------------------------------------------------------------
-- OPTIONAL: QUERY LOG HELPERS
-- -----------------------------------------------------------------------------

-- After running the benchmarks above, inspect recent query metrics.
-- Uncomment and adjust the time window as needed.

-- SELECT
--     query_duration_ms,
--     read_rows,
--     read_bytes,
--     memory_usage,
--     query
-- FROM system.query_log
-- WHERE event_time >= now() - INTERVAL 15 MINUTE
--   AND type = 'QueryFinish'
--   AND current_database = currentDatabase()
-- ORDER BY event_time DESC
-- LIMIT 50;
