CREATE VIEW IF NOT EXISTS ohlc_prices ON CLUSTER 'tokenapis-a' AS
SELECT
    -- bar interval --
    timestamp,
    interval_min,

    -- timestamp & block number --
    min(min_timestamp) as min_timestamp,
    max(max_timestamp) as max_timestamp,
    min(min_block_num) as min_block_num,
    max(max_block_num) as max_block_num,

    -- DEX identity --
    pool,
    factory,
    protocol,
    token0,
    token1,

    -- Aggregate --
    argMinMerge(open0) AS open0,
    quantileDeterministicMerge(0.95)(quantile0) as high_quantile0,
    quantileDeterministicMerge(0.05)(quantile0) as low_quantile0,
    argMaxMerge(close0) AS close0,

    -- volume --
    sum(gross_volume0) AS gross_volume0,
    sum(gross_volume1) AS gross_volume1,
    sum(net_flow0) AS net_flow0,
    sum(net_flow1) AS net_flow1,

    -- universal --
    sum(transactions) as transactions,
    uniqMerge(uniq_user) as uniq_user,
    uniqMerge(uniq_tx_from) as uniq_tx_from
FROM state_ohlc_prices
GROUP BY
    interval_min,
    pool, factory, protocol, token0, token1,
    timestamp;
