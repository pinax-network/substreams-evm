-- Normalized initialize logs written directly by `evm-dex`
CREATE TABLE IF NOT EXISTS initialize AS TEMPLATE_LOG
COMMENT 'DEX initialize events normalized across supported protocols';

ALTER TABLE initialize
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
    ADD COLUMN IF NOT EXISTS pool                        String;
