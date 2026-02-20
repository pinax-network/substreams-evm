-- Finalized read view over the AggregatingMergeTree "state" table
CREATE VIEW IF NOT EXISTS historical_erc20_balances AS
SELECT
    -- block/window
    interval_min,
    timestamp,
    min(min_block_num)                     AS min_block_num,       -- SimpleAggregateFunction(min)
    max(max_block_num)                     AS max_block_num,       -- SimpleAggregateFunction(max)

    -- keys
    contract,
    address,

    -- OHLC finalized
    argMinMerge(open)                      AS open,            -- from AggregateFunction(argMin, UInt256, UInt32)
    max(high)                              AS high,            -- SimpleAggregateFunction(max)
    min(low)                               AS low,             -- SimpleAggregateFunction(min)
    argMaxMerge(close)                     AS close,           -- from AggregateFunction(argMax, UInt256, UInt32)

    -- activity finalized
    sum(transactions)                      AS transactions     -- SimpleAggregateFunction(sum)
FROM historical_erc20_balances_state
GROUP BY
    interval_min, address, contract, timestamp;
