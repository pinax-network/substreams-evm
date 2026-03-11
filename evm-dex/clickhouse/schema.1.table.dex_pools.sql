-- Unified DEX pool registry, populated directly by the map_dex_pools module --
-- Provides a protocol-agnostic pool registry with a generalized coins array   --
-- that works across 2-token and N-token AMMs (Uniswap, Curve, Balancer, ...). --
CREATE TABLE IF NOT EXISTS dex_pools (
    -- block --
    block_num           UInt32,
    block_hash          String,
    timestamp           DateTime('UTC'),
    minute              UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- pool identity --
    address             String  COMMENT 'Pool/exchange contract address (primary lookup key)',
    protocol            LowCardinality(String) COMMENT 'Protocol identifier, e.g. uniswap_v2, curvefi',
    factory             String  COMMENT 'Factory contract that created the pool (empty for direct deployments)',

    -- generalized token array --
    -- Comma-separated hex addresses in on-chain index order.
    -- For Uniswap V2/V3: "0x<token0>,0x<token1>"
    -- For Curve: "0x<coin0>,0x<coin1>,...,0x<coinN>"
    -- For Balancer V3: all registered TokenConfig addresses
    coins               String  COMMENT 'Comma-separated list of token addresses in on-chain order',

    -- creation context --
    tx_hash             String,
    log_index           UInt32,
    log_ordinal         UInt64,

    -- constraints --
    CONSTRAINT address_not_empty CHECK address != '',
    CONSTRAINT protocol_not_empty CHECK protocol != '',

    -- indexes --
    INDEX idx_block_num (block_num)  TYPE minmax GRANULARITY 1,
    INDEX idx_timestamp (timestamp)  TYPE minmax GRANULARITY 1,
    INDEX idx_minute    (minute)     TYPE minmax GRANULARITY 1,

    -- projections --
    PROJECTION prj_address_lookup ( SELECT address, protocol, factory, coins GROUP BY address, protocol, factory, coins ),
    PROJECTION prj_protocol_count ( SELECT protocol, count(), min(block_num), max(block_num) GROUP BY protocol )
)
ENGINE = MergeTree
ORDER BY (block_num, address)
COMMENT 'Unified DEX pool registry populated directly by map_dex_pools (generalized coins array)';
