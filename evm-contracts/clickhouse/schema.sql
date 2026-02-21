CREATE TABLE IF NOT EXISTS blocks (
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime(0, 'UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- PROJECTIONS --
    PROJECTION prj_block_hash ( SELECT * ORDER BY block_hash ),
    PROJECTION prj_timestamp ( SELECT * ORDER BY timestamp )
)
ENGINE = MergeTree
ORDER BY ( block_num )
COMMENT 'Blocks';

CREATE TABLE IF NOT EXISTS contracts (
    -- block --
    block_num               UInt32,
    block_hash              String,
    block_date              Date MATERIALIZED toDate(timestamp),
    timestamp               DateTime(0, 'UTC'),

    -- transaction --
    transaction_hash        String,
    transaction_index       UInt32,

    -- contract creation --
    ordinal                 UInt64,
    address                 String,
    "from"                  String,
    "to"                    String,
    deployer                String,
    factory                 String DEFAULT '',
    code                    String DEFAULT '',
    code_hash               String DEFAULT '',
    input                   String DEFAULT '',

    -- indexes --
    INDEX idx_address       (address)       TYPE bloom_filter(0.005) GRANULARITY 1,
    INDEX idx_from          (from)          TYPE bloom_filter(0.005) GRANULARITY 1,
    INDEX idx_deployer      (deployer)      TYPE bloom_filter(0.005) GRANULARITY 1,
    INDEX idx_factory       (factory)       TYPE bloom_filter(0.005) GRANULARITY 1,
    INDEX idx_code_hash     (code_hash)     TYPE bloom_filter(0.005) GRANULARITY 1,
    INDEX idx_tx_hash       (transaction_hash) TYPE bloom_filter(0.005) GRANULARITY 1
)
ENGINE = ReplacingMergeTree
ORDER BY (timestamp, block_num, block_hash, transaction_index, ordinal);
