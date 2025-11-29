CREATE TABLE IF NOT EXISTS TEMPLATE_LOG (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    UInt32, -- derived from Substreams
    tx_hash                     String,
    tx_from                     String,
    tx_to                       LowCardinality(String),
    tx_nonce                    UInt64,
    tx_gas_price                UInt256,
    tx_gas_limit                UInt64,
    tx_gas_used                 UInt64,
    tx_value                    UInt256,

    -- log --
    log_index                   UInt32, -- derived from Substreams
    log_address                 LowCardinality(String),
    log_ordinal                 UInt32,
    log_topics                  String COMMENT 'Comma-separated list of log topics',
    log_topic0                  String MATERIALIZED splitByChar(',', log_topics)[1], -- first topic (topic0), empty string if no topics
    log_topic1                  String MATERIALIZED splitByChar(',', log_topics)[2], -- second topic (topic1), empty string if no topics
    log_topic2                  String MATERIALIZED splitByChar(',', log_topics)[3], -- third topic (topic2), empty string if no topics
    log_topic3                  String MATERIALIZED splitByChar(',', log_topics)[4], -- fourth topic (topic3), empty string if no topics
    log_data                    String,

    -- INDEXES --
    INDEX idx_tx_value (tx_value) TYPE minmax GRANULARITY 1,
    INDEX idx_log_ordinal (log_ordinal) TYPE minmax GRANULARITY 1,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_tx_from_count ( SELECT tx_from, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY tx_from ),
    PROJECTION prj_tx_to_count ( SELECT tx_to, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY tx_to ),
    PROJECTION prj_tx_to_from_count ( SELECT tx_to, tx_from, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY tx_to, tx_from ),
    PROJECTION prj_log_topic0_count ( SELECT log_topic0, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_topic0 ),
    PROJECTION prj_log_address_count ( SELECT log_address, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address ),

    -- minute --
    PROJECTION prj_block_hash_by_timestamp ( SELECT block_hash, minute, timestamp, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute)  GROUP BY block_hash, minute,timestamp ),
    PROJECTION prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY tx_hash, minute, timestamp ),
    PROJECTION prj_log_address_by_minute ( SELECT log_address, minute, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, minute )
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num,
    tx_index, log_index,
    block_hash
);

-- Template for Transactions (without log fields) --
CREATE TABLE IF NOT EXISTS TEMPLATE_TRANSACTION AS TEMPLATE_LOG
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num,
    tx_index,
    block_hash
);
ALTER TABLE TEMPLATE_TRANSACTION
    DROP PROJECTION IF EXISTS prj_log_address_by_minute,
    DROP PROJECTION IF EXISTS prj_log_topic0_count,
    DROP PROJECTION IF EXISTS prj_log_address_count,
    DROP INDEX IF EXISTS idx_log_ordinal,
    DROP COLUMN IF EXISTS log_index,
    DROP COLUMN IF EXISTS log_address,
    DROP COLUMN IF EXISTS log_ordinal,
    DROP COLUMN IF EXISTS log_topic0,
    DROP COLUMN IF EXISTS log_topic1,
    DROP COLUMN IF EXISTS log_topic2,
    DROP COLUMN IF EXISTS log_topic3,
    DROP COLUMN IF EXISTS log_topics,
    DROP COLUMN IF EXISTS log_data;

-- Template for Calls (extends Transaction template with call-specific fields) --
CREATE TABLE IF NOT EXISTS TEMPLATE_CALL AS TEMPLATE_TRANSACTION
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num,
    tx_index, call_index,
    block_hash
);
ALTER TABLE TEMPLATE_CALL
    -- call --
    ADD COLUMN IF NOT EXISTS call_index         UInt32,
    ADD COLUMN IF NOT EXISTS call_gas_consumed  UInt64,
    ADD COLUMN IF NOT EXISTS call_gas_limit     UInt64,
    ADD COLUMN IF NOT EXISTS call_depth         UInt32;
