-- Template Logs --
CREATE TABLE IF NOT EXISTS TEMPLATE_LOG (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 MATERIALIZED toRelativeMinuteNum(timestamp),

    -- transaction --
    tx_index                    UInt32, -- derived from Substreams
    tx_hash                     String,
    tx_from                     String,
    tx_to                       String,
    tx_nonce                    UInt64,
    tx_gas_price                UInt256,
    tx_gas_limit                UInt64,
    tx_gas_used                 UInt64,
    tx_value                    UInt256,

    -- log --
    log_index                   UInt32, -- derived from Substreams
    log_block_index             UInt32 COMMENT 'BlockIndex represents the index of the log relative to the Block.',
    log_address                 String,
    log_ordinal                 UInt32 COMMENT 'The block global ordinal when the log was recorded.',
    log_topics                  String COMMENT 'Comma-separated list of log topics',
    log_topic0                  String MATERIALIZED splitByChar(',', log_topics)[1], -- event signature
    log_topic1                  String MATERIALIZED splitByChar(',', log_topics)[2], -- second topic (topic1), empty string if no topics
    log_topic2                  String MATERIALIZED splitByChar(',', log_topics)[3], -- third topic (topic2), empty string if no topics
    log_topic3                  String MATERIALIZED splitByChar(',', log_topics)[4], -- fourth topic (topic3), empty string if no topics
    log_data                    String,

    -- call metadata (only available on chains with DetailLevel: EXTENDED) --
    call_caller                 String,
    call_index                  UInt32,
    call_begin_ordinal          UInt64,
    call_end_ordinal            UInt64,
    call_address                String,
    call_value                  UInt256,
    call_gas_consumed           UInt64,
    call_gas_limit              UInt64,
    call_depth                  UInt32,
    call_parent_index           UInt32,
    call_type                   LowCardinality(String),

    -- INDEXES --
    INDEX idx_block_num         (block_num)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_minute            (minute)                    TYPE minmax             GRANULARITY 1,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_tx_from_count ( SELECT tx_from, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY tx_from ),
    PROJECTION prj_call_caller_count ( SELECT call_caller, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY call_caller ),
    PROJECTION prj_log_address_count ( SELECT log_address, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address ),

    -- minute + timestamp --
    PROJECTION prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp GROUP BY tx_hash, minute, timestamp ),

    -- minute --
    PROJECTION prj_log_address_by_minute ( SELECT log_address, minute GROUP BY log_address, minute ),
    PROJECTION prj_call_caller_by_minute ( SELECT call_caller, minute, count() GROUP BY call_caller, minute ),
    PROJECTION prj_tx_from_by_minute ( SELECT tx_from, minute, count() GROUP BY tx_from, minute )
)
ENGINE = MergeTree
ORDER BY (
    timestamp, block_num
);
