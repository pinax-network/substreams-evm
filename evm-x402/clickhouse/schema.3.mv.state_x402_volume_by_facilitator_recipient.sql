-- x402 time-windowed payment volume grouped by facilitator and recipient
CREATE TABLE IF NOT EXISTS state_x402_volume_by_facilitator_recipient (
    -- bar interval --
    timestamp               DateTime('UTC', 0) COMMENT 'beginning of the bar',
    interval_min            UInt16 DEFAULT 1 COMMENT 'bar interval in minutes (1m, 5m, 10m, 30m, 1h, 4h, 1d, 1w)',

    -- timestamp & block number --
    min_timestamp           SimpleAggregateFunction(min, DateTime('UTC', 0)) COMMENT 'first timestamp seen',
    max_timestamp           SimpleAggregateFunction(max, DateTime('UTC', 0)) COMMENT 'last timestamp seen',
    min_block_num           SimpleAggregateFunction(min, UInt32) COMMENT 'first block number seen',
    max_block_num           SimpleAggregateFunction(max, UInt32) COMMENT 'last block number seen',

    -- x402 identity --
    facilitator             String,
    recipient               String,
    asset                   LowCardinality(String),
    transfer_method         LowCardinality(String),
    settlement_source       LowCardinality(String),
    scheme                  LowCardinality(String),

    -- volume --
    payments                SimpleAggregateFunction(sum, UInt64),
    amount                  SimpleAggregateFunction(sum, UInt256),
    uniq_payer              AggregateFunction(uniq, String),
    uniq_tx_from            AggregateFunction(uniq, String),
    uniq_tx_hash            AggregateFunction(uniq, String),

    -- indexes --
    INDEX idx_timestamp         (timestamp)         TYPE minmax       GRANULARITY 1,
    INDEX idx_min_timestamp     (min_timestamp)     TYPE minmax       GRANULARITY 1,
    INDEX idx_max_timestamp     (max_timestamp)     TYPE minmax       GRANULARITY 1,
    INDEX idx_min_block_num     (min_block_num)     TYPE minmax       GRANULARITY 1,
    INDEX idx_max_block_num     (max_block_num)     TYPE minmax       GRANULARITY 1,
    INDEX idx_facilitator       (facilitator)       TYPE bloom_filter GRANULARITY 1,
    INDEX idx_recipient         (recipient)         TYPE bloom_filter GRANULARITY 1,
    INDEX idx_asset             (asset)             TYPE set(1024)    GRANULARITY 1,
    INDEX idx_payments          (payments)          TYPE minmax       GRANULARITY 1,
    INDEX idx_amount            (amount)            TYPE minmax       GRANULARITY 1
)
ENGINE = AggregatingMergeTree
ORDER BY (
    facilitator,
    recipient,
    asset,
    transfer_method,
    settlement_source,
    scheme,
    interval_min,
    timestamp
)
COMMENT 'x402 payment volume by interval, facilitator, recipient, and asset';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_x402_volume_by_facilitator_recipient
TO state_x402_volume_by_facilitator_recipient
AS
WITH
    -- predefined intervals --
    -- in minutes: 1m, 5m, 10m, 30m, 1h, 4h, 1d, 1w
    [1, 5, 10, 30, 60, 240, 1440, 10080] AS intervals
SELECT
    arrayJoin(intervals) AS interval_min,
    toDateTime(intDiv(toUInt32(p.timestamp), interval_min * 60) * interval_min * 60) AS timestamp,

    min(p.timestamp) AS min_timestamp,
    max(p.timestamp) AS max_timestamp,
    min(p.block_num) AS min_block_num,
    max(p.block_num) AS max_block_num,

    facilitator,
    recipient,
    asset,
    transfer_method,
    settlement_source,
    scheme,

    count() AS payments,
    sum(amount) AS amount,
    uniqState(payer) AS uniq_payer,
    uniqState(tx_from) AS uniq_tx_from,
    uniqState(tx_hash) AS uniq_tx_hash
FROM x402_payments AS p
GROUP BY
    interval_min,
    timestamp,
    facilitator,
    recipient,
    asset,
    transfer_method,
    settlement_source,
    scheme;
