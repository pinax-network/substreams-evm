-- x402 totals grouped for facilitator-first access patterns
CREATE TABLE IF NOT EXISTS state_x402_totals_by_facilitator (
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

    -- aggregates --
    payments                SimpleAggregateFunction(sum, UInt64),
    amount                  SimpleAggregateFunction(sum, UInt256),
    uniq_payer              AggregateFunction(uniq, String),
    uniq_tx_from            AggregateFunction(uniq, String),
    uniq_tx_hash            AggregateFunction(uniq, String),

    -- indexes --
    INDEX idx_min_timestamp     (min_timestamp)     TYPE minmax       GRANULARITY 1,
    INDEX idx_max_timestamp     (max_timestamp)     TYPE minmax       GRANULARITY 1,
    INDEX idx_min_block_num     (min_block_num)     TYPE minmax       GRANULARITY 1,
    INDEX idx_max_block_num     (max_block_num)     TYPE minmax       GRANULARITY 1,
    INDEX idx_facilitator       (facilitator)       TYPE bloom_filter GRANULARITY 1,
    INDEX idx_recipient         (recipient)         TYPE bloom_filter GRANULARITY 1,
    INDEX idx_asset             (asset)             TYPE set(1024)    GRANULARITY 1,
    INDEX idx_payments          (payments)          TYPE minmax       GRANULARITY 1,
    INDEX idx_amount            (amount)            TYPE minmax       GRANULARITY 1,

    -- projections --
    PROJECTION prj_group_by_facilitator (
        SELECT
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num),
            facilitator,
            asset,
            transfer_method,
            settlement_source,
            scheme,
            sum(payments),
            sum(amount),
            uniqMerge(uniq_payer),
            uniqMerge(uniq_tx_from),
            uniqMerge(uniq_tx_hash)
        GROUP BY facilitator, asset, transfer_method, settlement_source, scheme
    ),
    PROJECTION prj_group_by_facilitator_recipient (
        SELECT
            min(min_timestamp),
            max(max_timestamp),
            min(min_block_num),
            max(max_block_num),
            facilitator,
            recipient,
            asset,
            transfer_method,
            settlement_source,
            scheme,
            sum(payments),
            sum(amount),
            uniqMerge(uniq_payer),
            uniqMerge(uniq_tx_from),
            uniqMerge(uniq_tx_hash)
        GROUP BY facilitator, recipient, asset, transfer_method, settlement_source, scheme
    )
)
ENGINE = AggregatingMergeTree
ORDER BY (facilitator, recipient, asset, transfer_method, settlement_source, scheme)
SETTINGS deduplicate_merge_projection_mode = 'rebuild'
COMMENT 'x402 totals grouped by facilitator first and recipient second';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_state_x402_totals_by_facilitator
TO state_x402_totals_by_facilitator
AS
SELECT
    min(timestamp) AS min_timestamp,
    max(timestamp) AS max_timestamp,
    min(block_num) AS min_block_num,
    max(block_num) AS max_block_num,

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
FROM x402_payments
GROUP BY facilitator, recipient, asset, transfer_method, settlement_source, scheme;
