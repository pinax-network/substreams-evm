-- Swaps table to store DEX swap events from various protocols --
CREATE TABLE IF NOT EXISTS swaps (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

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

    -- call --
    call_caller                 String COMMENT 'Call-level caller address from shared log metadata',
    call_index                  UInt32 COMMENT 'Call index from shared log metadata',
    call_begin_ordinal          UInt64 COMMENT 'Call begin ordinal from shared log metadata',
    call_end_ordinal            UInt64 COMMENT 'Call end ordinal from shared log metadata',
    call_address                String COMMENT 'Call recipient address from shared log metadata',
    call_value                  UInt256 COMMENT 'Call value from shared log metadata',
    call_gas_consumed           UInt64 COMMENT 'Call gas consumed from shared log metadata',
    call_gas_limit              UInt64 COMMENT 'Call gas limit from shared log metadata',
    call_depth                  UInt32 COMMENT 'Call depth from shared log metadata',
    call_parent_index           UInt32 COMMENT 'Call parent index from shared log metadata',
    call_type                   LowCardinality(String) COMMENT 'Call type from shared log metadata',

    -- log --
    log_index                   UInt32, -- derived from Substreams
    log_block_index             UInt32 COMMENT 'BlockIndex represents the index of the log relative to the Block.',
    log_address                 String,
    log_ordinal                 UInt32 COMMENT 'The block global ordinal when the log was recorded.',
    log_topic0                  LowCardinality(String),

    -- swap event information --
    protocol                    Enum8(
        'sunpump' = 1,
        'uniswap_v1' = 2,
        'uniswap_v2' = 3,
        'uniswap_v3' = 4,
        'uniswap_v4' = 5,
        'curvefi' = 6,
        'balancer' = 7,
        'bancor' = 8,
        'cow' = 9,
        'aerodrome' = 10,
        'dodo' = 11,
        'woofi' = 12,
        'traderjoe' = 13,
        'kyber_elastic' = 14
    ) COMMENT 'protocol identifier',
    factory                     LowCardinality(String) COMMENT 'Factory contract address',
    pool                        String COMMENT 'Pool/exchange contract address',
    user                        String COMMENT 'User wallet address',
    input_contract              String COMMENT 'Input token contract address',
    input_amount                UInt256 COMMENT 'Amount of input tokens swapped',
    output_contract             String COMMENT 'Output token contract address',
    output_amount               UInt256 COMMENT 'Amount of output tokens received',

    -- contraints data validation --
    CONSTRAINT log_address_not_empty CHECK log_address != '',
    CONSTRAINT log_topic0_not_empty CHECK log_topic0 != '',
    CONSTRAINT tx_hash_not_empty CHECK tx_hash != '',
    CONSTRAINT tx_from_not_empty CHECK tx_from != '',
    CONSTRAINT factory_not_empty CHECK factory != '',
    CONSTRAINT pool_not_empty CHECK pool != '',
    CONSTRAINT user_not_empty CHECK user != '',
    CONSTRAINT input_contract_not_empty CHECK input_contract != '',
    CONSTRAINT output_contract_not_empty CHECK output_contract != '',
    CONSTRAINT input_amount_nonzero CHECK input_amount > 0,
    CONSTRAINT output_amount_nonzero CHECK output_amount > 0,

    -- materialized token pair (canonical ordering) --
    token0                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, input_contract, output_contract) COMMENT 'Lexicographically smaller token address',
    token1                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, output_contract, input_contract) COMMENT 'Lexicographically larger token address',
    amount0                     UInt256 MATERIALIZED if(input_contract <= output_contract, input_amount, output_amount) COMMENT 'Amount of token0 swapped',
    amount1                     UInt256 MATERIALIZED if(input_contract <= output_contract, output_amount, input_amount) COMMENT 'Amount of token1 swapped',

    -- INDEXES --
    INDEX idx_block_num         (block_num)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_minute            (minute)                    TYPE minmax             GRANULARITY 1,
    INDEX idx_amount0 (amount0) TYPE minmax,
    INDEX idx_amount1 (amount1) TYPE minmax,
    INDEX idx_input_amount (input_amount) TYPE minmax,
    INDEX idx_output_amount (output_amount) TYPE minmax,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_protocol_count ( SELECT protocol, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY protocol ),
    PROJECTION prj_factory_count ( SELECT factory, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY factory ),
    PROJECTION prj_pool_count ( SELECT pool, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY pool ),
    PROJECTION prj_tx_from_count ( SELECT tx_from, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY tx_from ),
    PROJECTION prj_call_caller_count ( SELECT call_caller, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY call_caller ),
    PROJECTION prj_user_count ( SELECT user, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY user ),
    PROJECTION prj_input_contract_count ( SELECT input_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY input_contract ),
    PROJECTION prj_output_contract_count ( SELECT output_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY output_contract ),
    PROJECTION prj_token0_count ( SELECT token0, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY token0 ),
    PROJECTION prj_token1_count ( SELECT token1, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY token1 ),

    -- used for `/pools` endpoint --
    PROJECTION prj_all_count (
        SELECT
            protocol,
            factory,
            pool,
            count(),
            min(block_num),
            max(block_num),
            min(timestamp),
            max(timestamp),
            min(minute),
            max(minute),
            token0,
            token1
        GROUP BY protocol, factory, pool, token0, token1
    ),

    -- minute + timestamp --
    PROJECTION prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp GROUP BY tx_hash, minute, timestamp ),

    -- minute --
    PROJECTION prj_log_address_by_minute ( SELECT log_address, minute GROUP BY log_address, minute ),
    PROJECTION prj_log_topic0_by_minute ( SELECT log_topic0, minute GROUP BY log_topic0, minute ),
    PROJECTION prj_call_type_by_minute ( SELECT call_type, minute GROUP BY call_type, minute ),
    PROJECTION prj_protocol_by_minute ( SELECT protocol, minute, count() GROUP BY protocol, minute ),
    PROJECTION prj_tx_from_by_minute ( SELECT tx_from, minute, count() GROUP BY tx_from, minute ),
    PROJECTION prj_call_caller_by_minute ( SELECT call_caller, minute, count() GROUP BY call_caller, minute ),
    PROJECTION prj_factory_by_minute ( SELECT factory, minute, count() GROUP BY factory, minute ),
    PROJECTION prj_pool_by_minute ( SELECT pool, minute, count() GROUP BY pool, minute ),
    PROJECTION prj_user_by_minute ( SELECT user, minute, count() GROUP BY user, minute ),
    PROJECTION prj_input_contract_by_minute ( SELECT input_contract, minute, count() GROUP BY input_contract, minute ),
    PROJECTION prj_output_contract_by_minute ( SELECT output_contract, minute, count() GROUP BY output_contract, minute ),
    PROJECTION prj_token0_by_minute ( SELECT token0, minute, count() GROUP BY token0, minute ),
    PROJECTION prj_token1_by_minute ( SELECT token1, minute, count() GROUP BY token1, minute )
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
)
COMMENT 'DEX swap events normalized across supported protocols';

-- Flash/complex swaps that cannot be losslessly normalized to a single input/output pair --
CREATE TABLE IF NOT EXISTS swaps_flash (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    UInt32,
    tx_hash                     String,
    tx_from                     String,
    tx_to                       String,
    tx_nonce                    UInt64,
    tx_gas_price                UInt256,
    tx_gas_limit                UInt64,
    tx_gas_used                 UInt64,
    tx_value                    UInt256,

    -- call --
    call_caller                 String COMMENT 'Call-level caller address from shared log metadata',
    call_index                  UInt32 COMMENT 'Call index from shared log metadata',
    call_begin_ordinal          UInt64 COMMENT 'Call begin ordinal from shared log metadata',
    call_end_ordinal            UInt64 COMMENT 'Call end ordinal from shared log metadata',
    call_address                String COMMENT 'Call recipient address from shared log metadata',
    call_value                  UInt256 COMMENT 'Call value from shared log metadata',
    call_gas_consumed           UInt64 COMMENT 'Call gas consumed from shared log metadata',
    call_gas_limit              UInt64 COMMENT 'Call gas limit from shared log metadata',
    call_depth                  UInt32 COMMENT 'Call depth from shared log metadata',
    call_parent_index           UInt32 COMMENT 'Call parent index from shared log metadata',
    call_type                   LowCardinality(String) COMMENT 'Call type from shared log metadata',

    -- log --
    log_index                   UInt32,
    log_block_index             UInt32 COMMENT 'BlockIndex represents the index of the log relative to the Block.',
    log_address                 String,
    log_ordinal                 UInt32 COMMENT 'The block global ordinal when the log was recorded.',
    log_topic0                  LowCardinality(String),

    -- swap event information --
    protocol                    LowCardinality(String) COMMENT 'flash/complex swap protocol identifier',
    factory                     LowCardinality(String) COMMENT 'Factory contract address',
    pool                        String COMMENT 'Pool/exchange contract address',
    user                        String COMMENT 'User wallet address',

    -- raw swap legs --
    token0                      String COMMENT 'Token0 contract address',
    token1                      String COMMENT 'Token1 contract address',
    amount0_in                  UInt256 COMMENT 'Amount of token0 in',
    amount1_in                  UInt256 COMMENT 'Amount of token1 in',
    amount0_out                 UInt256 COMMENT 'Amount of token0 out',
    amount1_out                 UInt256 COMMENT 'Amount of token1 out',

    -- indexes --
    INDEX idx_block_num         (block_num)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_timestamp         (timestamp)                 TYPE minmax             GRANULARITY 1,
    INDEX idx_minute            (minute)                    TYPE minmax             GRANULARITY 1
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
)
COMMENT 'Flash/complex DEX swap events kept out of the normalized swaps table';


-- ============================================================================
-- MIGRATION NOTE: swap normalization moved to Substreams (map_dex_swaps)
-- ============================================================================
-- Previously this file contained materialized views (mv_swaps_*) that
-- translated each protocol's raw swap table into the unified `swaps` table.
--
-- These MVs have been SUPERSEDED by the `map_dex_swaps` Substreams module
-- which performs the same normalization at the Substreams layer.  The sink
-- (db_out) now writes directly to `swaps` and `swaps_flash`, making the
-- per-protocol MVs redundant and error-prone.
--
-- Benefits of moving normalization into Substreams:
--   * Logic is versioned, tested, and cached as a module cache.
--   * A single `map_dex_swaps` output feeds all downstream sinks.
--   * Adding a new protocol only requires updating one Rust file.
--   * Eliminates the dual-write / duplicate-row risk of the MV pattern.
--
-- If you are running an existing deployment with the old MVs, drop them with:
--   DROP VIEW IF EXISTS mv_swaps_sunpump_token_purchased;
--   DROP VIEW IF EXISTS mv_swaps_sunpump_token_sold;
--   DROP VIEW IF EXISTS mv_swaps_uniswap_v1_token_purchase;
--   DROP VIEW IF EXISTS mv_swaps_uniswap_v1_eth_purchase;
--   DROP VIEW IF EXISTS mv_swaps_uniswap_v2_swap;
--   DROP VIEW IF EXISTS mv_swaps_flash_uniswap_v2_swap;
--   DROP VIEW IF EXISTS mv_swaps_uniswap_v3_swap;
--   DROP VIEW IF EXISTS mv_swaps_uniswap_v4_swap;
--   DROP VIEW IF EXISTS mv_swaps_curvefi_token_exchange;
--   DROP VIEW IF EXISTS mv_swaps_balancer_vault_swap;
--   DROP VIEW IF EXISTS mv_swaps_bancor_conversion;
--   DROP VIEW IF EXISTS mv_swaps_cow_trade;
--   DROP VIEW IF EXISTS mv_swaps_aerodrome_swap;
--   DROP VIEW IF EXISTS mv_swaps_dodo_order_history;
--   DROP VIEW IF EXISTS mv_swaps_woofi_woo_swap;
--   DROP VIEW IF EXISTS mv_swaps_traderjoe_swap;
--   DROP VIEW IF EXISTS mv_swaps_kyber_elastic_swap;
-- ============================================================================
