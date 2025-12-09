-- Curve.fi TokenExchange (Swap) --
CREATE TABLE IF NOT EXISTS curvefi_token_exchange AS TEMPLATE_LOG
COMMENT 'Curve.fi TokenExchange (swap) events';
ALTER TABLE curvefi_token_exchange
    -- event information --
    ADD COLUMN IF NOT EXISTS buyer              String COMMENT 'Buyer wallet address',
    ADD COLUMN IF NOT EXISTS sold_id            Int128 COMMENT 'ID of token sold',
    ADD COLUMN IF NOT EXISTS sold_amount        UInt256 COMMENT 'Amount of tokens sold',
    ADD COLUMN IF NOT EXISTS sold_token         String COMMENT 'Token sold',
    ADD COLUMN IF NOT EXISTS bought_id          Int128 COMMENT 'ID of token bought',
    ADD COLUMN IF NOT EXISTS bought_amount      UInt256 COMMENT 'Amount of tokens bought',
    ADD COLUMN IF NOT EXISTS bought_token       String COMMENT 'Token bought',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi AddLiquidity --
CREATE TABLE IF NOT EXISTS curvefi_add_liquidity AS TEMPLATE_LOG
COMMENT 'Curve.fi AddLiquidity events';
ALTER TABLE curvefi_add_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts added',
    ADD COLUMN IF NOT EXISTS fees               String COMMENT 'Comma-separated fees',
    ADD COLUMN IF NOT EXISTS invariant          UInt256 COMMENT 'Invariant value',
    ADD COLUMN IF NOT EXISTS token_supply       UInt256 COMMENT 'Total token supply',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi RemoveLiquidity --
CREATE TABLE IF NOT EXISTS curvefi_remove_liquidity AS TEMPLATE_LOG
COMMENT 'Curve.fi RemoveLiquidity events';
ALTER TABLE curvefi_remove_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts removed',
    ADD COLUMN IF NOT EXISTS fees               String COMMENT 'Comma-separated fees',
    ADD COLUMN IF NOT EXISTS token_supply       UInt256 COMMENT 'Total token supply',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi RemoveLiquidityOne --
CREATE TABLE IF NOT EXISTS curvefi_remove_liquidity_one AS TEMPLATE_LOG
COMMENT 'Curve.fi RemoveLiquidityOne events';
ALTER TABLE curvefi_remove_liquidity_one
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amount       UInt256 COMMENT 'Token amount',
    ADD COLUMN IF NOT EXISTS coin_amount        UInt256 COMMENT 'Coin amount',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi RemoveLiquidityImbalance --
CREATE TABLE IF NOT EXISTS curvefi_remove_liquidity_imbalance AS TEMPLATE_LOG
COMMENT 'Curve.fi RemoveLiquidityImbalance events';
ALTER TABLE curvefi_remove_liquidity_imbalance
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts',
    ADD COLUMN IF NOT EXISTS fees               String COMMENT 'Comma-separated fees',
    ADD COLUMN IF NOT EXISTS invariant          UInt256 COMMENT 'Invariant value',
    ADD COLUMN IF NOT EXISTS token_supply       UInt256 COMMENT 'Total token supply',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi PlainPoolDeployed --
CREATE TABLE IF NOT EXISTS curvefi_plain_pool_deployed AS TEMPLATE_LOG
COMMENT 'Curve.fi PlainPoolDeployed (pool creation) events';
ALTER TABLE curvefi_plain_pool_deployed
    -- event information --
    ADD COLUMN IF NOT EXISTS address            String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS a                  UInt256 COMMENT 'Amplification coefficient',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'Exchange fee',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi MetaPoolDeployed --
CREATE TABLE IF NOT EXISTS curvefi_meta_pool_deployed AS TEMPLATE_LOG
COMMENT 'Curve.fi MetaPoolDeployed (pool creation) events';
ALTER TABLE curvefi_meta_pool_deployed
    -- event information --
    ADD COLUMN IF NOT EXISTS address            String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS coin               String COMMENT 'Coin address',
    ADD COLUMN IF NOT EXISTS base_pool          String COMMENT 'Base pool address',
    ADD COLUMN IF NOT EXISTS a                  UInt256 COMMENT 'Amplification coefficient',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'Exchange fee',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address',

    -- indexes --
    ADD INDEX IF NOT EXISTS idx_address (address) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_coin (coin) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_base_pool (base_pool) TYPE bloom_filter,
    ADD INDEX IF NOT EXISTS idx_deployer (deployer) TYPE bloom_filter;

-- Curve.fi CommitNewFee --
CREATE TABLE IF NOT EXISTS curvefi_commit_new_fee AS TEMPLATE_LOG
COMMENT 'Curve.fi CommitNewFee events';
ALTER TABLE curvefi_commit_new_fee
    -- event information --
    ADD COLUMN IF NOT EXISTS deadline           UInt256 COMMENT 'Deadline timestamp',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'New fee',
    ADD COLUMN IF NOT EXISTS admin_fee          UInt256 COMMENT 'New admin fee',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address',

    -- indexes (PlainPoolDeployed) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

-- Curve.fi NewFee --
CREATE TABLE IF NOT EXISTS curvefi_new_fee AS TEMPLATE_LOG
COMMENT 'Curve.fi NewFee events';
ALTER TABLE curvefi_new_fee
    -- event information --
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'New fee',
    ADD COLUMN IF NOT EXISTS admin_fee          UInt256 COMMENT 'New admin fee',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address',

    -- indexes (PlainPoolDeployed) --
    ADD INDEX IF NOT EXISTS idx_factory (factory) TYPE bloom_filter;

