-- Template Transactions --
CREATE TABLE IF NOT EXISTS TEMPLATE_TRANSACTION (
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
    tx_value                    UInt256
)
ENGINE = MergeTree
-- TTL is applied to all base data tables
-- to automatically clean up old data
-- production tables are derived from MV's on these base tables
TTL timestamp + INTERVAL 1 DAY
ORDER BY (
    minute, timestamp, block_num
);

-- Template Logs --
CREATE TABLE IF NOT EXISTS TEMPLATE_LOG AS TEMPLATE_TRANSACTION;
ALTER TABLE TEMPLATE_LOG
    ADD COLUMN IF NOT EXISTS log_index                   UInt32, -- derived from Substreams
    ADD COLUMN IF NOT EXISTS log_address                 LowCardinality(String),
    ADD COLUMN IF NOT EXISTS log_ordinal                 UInt32,
    ADD COLUMN IF NOT EXISTS log_topics                  String COMMENT 'Comma-separated list of log topics',
    ADD COLUMN IF NOT EXISTS log_topic0                  String MATERIALIZED splitByChar(',', log_topics)[1], -- event signature
    ADD COLUMN IF NOT EXISTS log_topic1                  String MATERIALIZED splitByChar(',', log_topics)[2], -- second topic (topic1), empty string if no topics
    ADD COLUMN IF NOT EXISTS log_topic2                  String MATERIALIZED splitByChar(',', log_topics)[3], -- third topic (topic2), empty string if no topics
    ADD COLUMN IF NOT EXISTS log_topic3                  String MATERIALIZED splitByChar(',', log_topics)[4], -- fourth topic (topic3), empty string if no topics
    ADD COLUMN IF NOT EXISTS log_data                    String,

    -- call metadata --
    ADD COLUMN IF NOT EXISTS call_caller                 String,
    ADD COLUMN IF NOT EXISTS call_index                  UInt32,
    ADD COLUMN IF NOT EXISTS call_depth                  UInt32,
    ADD COLUMN IF NOT EXISTS call_type                   LowCardinality(String);

-- Template Calls --
CREATE TABLE IF NOT EXISTS TEMPLATE_CALL AS TEMPLATE_TRANSACTION;
ALTER TABLE TEMPLATE_CALL
    ADD COLUMN IF NOT EXISTS call_index              UInt32, -- derived from Substreams
    ADD COLUMN IF NOT EXISTS call_begin_ordinal      UInt64,
    ADD COLUMN IF NOT EXISTS call_end_ordinal        UInt64,
    ADD COLUMN IF NOT EXISTS call_caller             String,
    ADD COLUMN IF NOT EXISTS call_address            String,
    ADD COLUMN IF NOT EXISTS call_value              UInt256,
    ADD COLUMN IF NOT EXISTS call_gas_consumed       UInt64,
    ADD COLUMN IF NOT EXISTS call_gas_limit          UInt64,
    ADD COLUMN IF NOT EXISTS call_depth              UInt32,
    ADD COLUMN IF NOT EXISTS call_parent_index       UInt32,
    ADD COLUMN IF NOT EXISTS call_type               LowCardinality(String);