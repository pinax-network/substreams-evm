-- Normalized swaps table written directly by `evm-dex`
CREATE TABLE IF NOT EXISTS swaps AS TEMPLATE_LOG
COMMENT 'DEX swap events normalized across supported protocols';

ALTER TABLE swaps
    ADD COLUMN IF NOT EXISTS protocol                    Enum8(
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
    ADD COLUMN IF NOT EXISTS factory                     LowCardinality(String),
    ADD COLUMN IF NOT EXISTS pool                        String,
    ADD COLUMN IF NOT EXISTS user                        String,
    ADD COLUMN IF NOT EXISTS input_contract              String,
    ADD COLUMN IF NOT EXISTS input_amount                UInt256,
    ADD COLUMN IF NOT EXISTS output_contract             String,
    ADD COLUMN IF NOT EXISTS output_amount               UInt256,
    ADD COLUMN IF NOT EXISTS token0                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, input_contract, output_contract),
    ADD COLUMN IF NOT EXISTS token1                      LowCardinality(String) MATERIALIZED if(input_contract <= output_contract, output_contract, input_contract),
    ADD COLUMN IF NOT EXISTS amount0                     UInt256 MATERIALIZED if(input_contract <= output_contract, input_amount, output_amount),
    ADD COLUMN IF NOT EXISTS amount1                     UInt256 MATERIALIZED if(input_contract <= output_contract, output_amount, input_amount);

ALTER TABLE swaps
    ADD INDEX IF NOT EXISTS idx_amount0 (amount0) TYPE minmax GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_amount1 (amount1) TYPE minmax GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_input_amount (input_amount) TYPE minmax GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_output_amount (output_amount) TYPE minmax GRANULARITY 1;

ALTER TABLE swaps
    -- count() --
    ADD PROJECTION IF NOT EXISTS prj_protocol_count ( SELECT protocol, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY protocol ),
    ADD PROJECTION IF NOT EXISTS prj_factory_count ( SELECT factory, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY factory ),
    ADD PROJECTION IF NOT EXISTS prj_pool_count ( SELECT pool, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY pool ),
    ADD PROJECTION IF NOT EXISTS prj_user_count ( SELECT user, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY user ),
    ADD PROJECTION IF NOT EXISTS prj_input_contract_count ( SELECT input_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY input_contract ),
    ADD PROJECTION IF NOT EXISTS prj_output_contract_count ( SELECT output_contract, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY output_contract ),

    -- minute --
    ADD PROJECTION IF NOT EXISTS prj_protocol_by_minute ( SELECT protocol, minute, count() GROUP BY protocol, minute ),
    ADD PROJECTION IF NOT EXISTS prj_factory_by_minute ( SELECT factory, minute, count() GROUP BY factory, minute ),
    ADD PROJECTION IF NOT EXISTS prj_pool_by_minute ( SELECT pool, minute, count() GROUP BY pool, minute ),
    ADD PROJECTION IF NOT EXISTS prj_user_by_minute ( SELECT user, minute, count() GROUP BY user, minute ),
    ADD PROJECTION IF NOT EXISTS prj_input_contract_by_minute ( SELECT input_contract, minute, count() GROUP BY input_contract, minute ),
    ADD PROJECTION IF NOT EXISTS prj_output_contract_by_minute ( SELECT output_contract, minute, count() GROUP BY output_contract, minute );
