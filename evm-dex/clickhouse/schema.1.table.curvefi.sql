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
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

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
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

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
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

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
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

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
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi PlainPoolDeployed --
CREATE TABLE IF NOT EXISTS curvefi_plain_pool_deployed AS TEMPLATE_LOG
COMMENT 'Curve.fi PlainPoolDeployed (pool creation) events';
ALTER TABLE curvefi_plain_pool_deployed
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS address            String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS a                  UInt256 COMMENT 'Amplification coefficient',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'Exchange fee',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi PoolInit (direct deployment, decoded from __init__ constructor calldata) --
CREATE TABLE IF NOT EXISTS curvefi_pool_init AS TEMPLATE_LOG
COMMENT 'Curve.fi pool initialisation decoded from __init__ constructor calldata (non-factory deployments)';
ALTER TABLE curvefi_pool_init
    -- event information (decoded constructor args) --
    ADD COLUMN IF NOT EXISTS address            String COMMENT 'Deployed pool contract address',
    ADD COLUMN IF NOT EXISTS owner              String COMMENT 'Initial owner/admin address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin (token) addresses',
    ADD COLUMN IF NOT EXISTS pool_token         String COMMENT 'LP token address',
    ADD COLUMN IF NOT EXISTS a                  UInt256 COMMENT 'Amplification coefficient (_A)',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'Exchange fee scaled to 1e10 (_fee)',
    ADD COLUMN IF NOT EXISTS admin_fee          UInt256 COMMENT 'Admin fee fraction scaled to 1e10 (_admin_fee)';

-- Curve.fi MetaPoolDeployed --
CREATE TABLE IF NOT EXISTS curvefi_meta_pool_deployed AS TEMPLATE_LOG
COMMENT 'Curve.fi MetaPoolDeployed (pool creation) events';
ALTER TABLE curvefi_meta_pool_deployed
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS address            String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS coin               String COMMENT 'Coin address',
    ADD COLUMN IF NOT EXISTS base_pool          String COMMENT 'Base pool address',
    ADD COLUMN IF NOT EXISTS a                  UInt256 COMMENT 'Amplification coefficient',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'Exchange fee',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

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
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi NewFee --
CREATE TABLE IF NOT EXISTS curvefi_new_fee AS TEMPLATE_LOG
COMMENT 'Curve.fi NewFee events';
ALTER TABLE curvefi_new_fee
    -- event information --
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'New fee',
    ADD COLUMN IF NOT EXISTS admin_fee          UInt256 COMMENT 'New admin fee',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CommitNewAdmin (Pool, StableSwap, CryptoSwap, MetaPoolRegistry — shared topic hash) --
CREATE TABLE IF NOT EXISTS curvefi_commit_new_admin AS TEMPLATE_LOG
COMMENT 'Curve.fi CommitNewAdmin events (shared topic across Pool, StableSwap, CryptoSwap, MetaPoolRegistry)';
ALTER TABLE curvefi_commit_new_admin
    -- event information --
    ADD COLUMN IF NOT EXISTS deadline           UInt256 COMMENT 'Deadline timestamp',
    ADD COLUMN IF NOT EXISTS admin              String COMMENT 'New admin address',

    -- store (optional: present when emitter is a known pool) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi NewAdmin (Pool, StableSwap, CryptoSwap, MetaPoolRegistry — shared topic hash) --
CREATE TABLE IF NOT EXISTS curvefi_new_admin AS TEMPLATE_LOG
COMMENT 'Curve.fi NewAdmin events (shared topic across Pool, StableSwap, CryptoSwap, MetaPoolRegistry)';
ALTER TABLE curvefi_new_admin
    -- event information --
    ADD COLUMN IF NOT EXISTS admin              String COMMENT 'New admin address',

    -- store (optional: present when emitter is a known pool) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi RampA (Pool / StableSwap — shared topic hash) --
CREATE TABLE IF NOT EXISTS curvefi_ramp_a AS TEMPLATE_LOG
COMMENT 'Curve.fi RampA events';
ALTER TABLE curvefi_ramp_a
    -- event information --
    ADD COLUMN IF NOT EXISTS old_a              UInt256 COMMENT 'Previous amplification coefficient',
    ADD COLUMN IF NOT EXISTS new_a              UInt256 COMMENT 'New amplification coefficient',
    ADD COLUMN IF NOT EXISTS initial_time       UInt256 COMMENT 'Ramp start timestamp',
    ADD COLUMN IF NOT EXISTS future_time        UInt256 COMMENT 'Ramp end timestamp',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi StopRampA (Pool / StableSwap — shared topic hash) --
CREATE TABLE IF NOT EXISTS curvefi_stop_ramp_a AS TEMPLATE_LOG
COMMENT 'Curve.fi StopRampA events';
ALTER TABLE curvefi_stop_ramp_a
    -- event information --
    ADD COLUMN IF NOT EXISTS a                  UInt256 COMMENT 'Current amplification coefficient',
    ADD COLUMN IF NOT EXISTS t                  UInt256 COMMENT 'Timestamp',

    -- PlainPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi BasePoolAdded (Factory) --
CREATE TABLE IF NOT EXISTS curvefi_base_pool_added AS TEMPLATE_LOG
COMMENT 'Curve.fi BasePoolAdded events';
ALTER TABLE curvefi_base_pool_added
    -- event information --
    ADD COLUMN IF NOT EXISTS base_pool          String COMMENT 'Base pool contract address';

-- Curve.fi LiquidityGaugeDeployed (Factory) --
CREATE TABLE IF NOT EXISTS curvefi_liquidity_gauge_deployed AS TEMPLATE_LOG
COMMENT 'Curve.fi LiquidityGaugeDeployed events from Factory';
ALTER TABLE curvefi_liquidity_gauge_deployed
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS gauge              String COMMENT 'Gauge contract address';

-- ── CryptoSwap events (contract-specific: unique topic hashes) ────────────────

-- Curve.fi CryptoSwap TokenExchange --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_token_exchange AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap TokenExchange (swap) events';
ALTER TABLE curvefi_cryptoswap_token_exchange
    -- event information --
    ADD COLUMN IF NOT EXISTS buyer              String COMMENT 'Buyer wallet address',
    ADD COLUMN IF NOT EXISTS sold_id            UInt256 COMMENT 'ID of token sold',
    ADD COLUMN IF NOT EXISTS sold_amount        UInt256 COMMENT 'Amount of tokens sold',
    ADD COLUMN IF NOT EXISTS sold_token         String COMMENT 'Token sold',
    ADD COLUMN IF NOT EXISTS bought_id          UInt256 COMMENT 'ID of token bought',
    ADD COLUMN IF NOT EXISTS bought_amount      UInt256 COMMENT 'Amount of tokens bought',
    ADD COLUMN IF NOT EXISTS bought_token       String COMMENT 'Token bought',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap AddLiquidity --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_add_liquidity AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap AddLiquidity events';
ALTER TABLE curvefi_cryptoswap_add_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts added',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'Total fee paid',
    ADD COLUMN IF NOT EXISTS token_supply       UInt256 COMMENT 'Total LP token supply',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap RemoveLiquidity --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_remove_liquidity AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap RemoveLiquidity events';
ALTER TABLE curvefi_cryptoswap_remove_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amounts      String COMMENT 'Comma-separated token amounts removed',
    ADD COLUMN IF NOT EXISTS token_supply       UInt256 COMMENT 'Total LP token supply',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap RemoveLiquidityOne --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_remove_liquidity_one AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap RemoveLiquidityOne events';
ALTER TABLE curvefi_cryptoswap_remove_liquidity_one
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS token_amount       UInt256 COMMENT 'LP token amount burned',
    ADD COLUMN IF NOT EXISTS coin_index         UInt256 COMMENT 'Index of coin withdrawn',
    ADD COLUMN IF NOT EXISTS coin_amount        UInt256 COMMENT 'Coin amount received',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap ClaimAdminFee --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_claim_admin_fee AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap ClaimAdminFee events';
ALTER TABLE curvefi_cryptoswap_claim_admin_fee
    -- event information --
    ADD COLUMN IF NOT EXISTS admin              String COMMENT 'Admin address',
    ADD COLUMN IF NOT EXISTS tokens             UInt256 COMMENT 'Amount of tokens claimed',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap CommitNewParameters --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_commit_new_parameters AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap CommitNewParameters events';
ALTER TABLE curvefi_cryptoswap_commit_new_parameters
    -- event information --
    ADD COLUMN IF NOT EXISTS deadline               UInt256 COMMENT 'Deadline timestamp',
    ADD COLUMN IF NOT EXISTS admin_fee              UInt256 COMMENT 'New admin fee',
    ADD COLUMN IF NOT EXISTS mid_fee                UInt256 COMMENT 'New mid fee',
    ADD COLUMN IF NOT EXISTS out_fee                UInt256 COMMENT 'New out fee',
    ADD COLUMN IF NOT EXISTS fee_gamma              UInt256 COMMENT 'New fee gamma',
    ADD COLUMN IF NOT EXISTS allowed_extra_profit   UInt256 COMMENT 'New allowed extra profit',
    ADD COLUMN IF NOT EXISTS adjustment_step        UInt256 COMMENT 'New adjustment step',
    ADD COLUMN IF NOT EXISTS ma_half_time           UInt256 COMMENT 'New MA half time',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap NewParameters --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_new_parameters AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap NewParameters events';
ALTER TABLE curvefi_cryptoswap_new_parameters
    -- event information --
    ADD COLUMN IF NOT EXISTS admin_fee              UInt256 COMMENT 'New admin fee',
    ADD COLUMN IF NOT EXISTS mid_fee                UInt256 COMMENT 'New mid fee',
    ADD COLUMN IF NOT EXISTS out_fee                UInt256 COMMENT 'New out fee',
    ADD COLUMN IF NOT EXISTS fee_gamma              UInt256 COMMENT 'New fee gamma',
    ADD COLUMN IF NOT EXISTS allowed_extra_profit   UInt256 COMMENT 'New allowed extra profit',
    ADD COLUMN IF NOT EXISTS adjustment_step        UInt256 COMMENT 'New adjustment step',
    ADD COLUMN IF NOT EXISTS ma_half_time           UInt256 COMMENT 'New MA half time',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap RampAgamma --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_ramp_agamma AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap RampAgamma events';
ALTER TABLE curvefi_cryptoswap_ramp_agamma
    -- event information --
    ADD COLUMN IF NOT EXISTS initial_a          UInt256 COMMENT 'Initial amplification coefficient',
    ADD COLUMN IF NOT EXISTS future_a           UInt256 COMMENT 'Future amplification coefficient',
    ADD COLUMN IF NOT EXISTS initial_gamma      UInt256 COMMENT 'Initial gamma',
    ADD COLUMN IF NOT EXISTS future_gamma       UInt256 COMMENT 'Future gamma',
    ADD COLUMN IF NOT EXISTS initial_time       UInt256 COMMENT 'Ramp start timestamp',
    ADD COLUMN IF NOT EXISTS future_time        UInt256 COMMENT 'Ramp end timestamp',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- Curve.fi CryptoSwap StopRampA --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswap_stop_ramp_a AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwap StopRampA events';
ALTER TABLE curvefi_cryptoswap_stop_ramp_a
    -- event information --
    ADD COLUMN IF NOT EXISTS current_a          UInt256 COMMENT 'Current amplification coefficient',
    ADD COLUMN IF NOT EXISTS current_gamma      UInt256 COMMENT 'Current gamma',
    ADD COLUMN IF NOT EXISTS time               UInt256 COMMENT 'Timestamp',

    -- CryptoPoolDeployed (store) --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses';

-- ── CryptoSwapFactory events ──────────────────────────────────────────────────

-- Curve.fi CryptoSwapFactory CryptoPoolDeployed --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswapfactory_crypto_pool_deployed AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwapFactory CryptoPoolDeployed events';
ALTER TABLE curvefi_cryptoswapfactory_crypto_pool_deployed
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS address            String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS token              String COMMENT 'LP token address',
    ADD COLUMN IF NOT EXISTS coins              String COMMENT 'Comma-separated coin addresses',
    ADD COLUMN IF NOT EXISTS a                  UInt256 COMMENT 'Amplification coefficient',
    ADD COLUMN IF NOT EXISTS gamma              UInt256 COMMENT 'Gamma parameter',
    ADD COLUMN IF NOT EXISTS mid_fee            UInt256 COMMENT 'Mid fee',
    ADD COLUMN IF NOT EXISTS out_fee            UInt256 COMMENT 'Out fee',
    ADD COLUMN IF NOT EXISTS allowed_extra_profit UInt256 COMMENT 'Allowed extra profit',
    ADD COLUMN IF NOT EXISTS fee_gamma          UInt256 COMMENT 'Fee gamma',
    ADD COLUMN IF NOT EXISTS adjustment_step    UInt256 COMMENT 'Adjustment step',
    ADD COLUMN IF NOT EXISTS admin_fee          UInt256 COMMENT 'Admin fee',
    ADD COLUMN IF NOT EXISTS ma_half_time       UInt256 COMMENT 'MA half time',
    ADD COLUMN IF NOT EXISTS initial_price      UInt256 COMMENT 'Initial price',
    ADD COLUMN IF NOT EXISTS deployer           String COMMENT 'Deployer address';

-- Curve.fi CryptoSwapFactory LiquidityGaugeDeployed --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswapfactory_liquidity_gauge_deployed AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwapFactory LiquidityGaugeDeployed events';
ALTER TABLE curvefi_cryptoswapfactory_liquidity_gauge_deployed
    -- event information --
    ADD COLUMN IF NOT EXISTS pool               String COMMENT 'Pool contract address',
    ADD COLUMN IF NOT EXISTS token              String COMMENT 'LP token address',
    ADD COLUMN IF NOT EXISTS gauge              String COMMENT 'Gauge contract address';

-- Curve.fi CryptoSwapFactory TransferOwnership --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswapfactory_transfer_ownership AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwapFactory TransferOwnership events';
ALTER TABLE curvefi_cryptoswapfactory_transfer_ownership
    -- event information --
    ADD COLUMN IF NOT EXISTS old_owner          String COMMENT 'Previous owner address',
    ADD COLUMN IF NOT EXISTS new_owner          String COMMENT 'New owner address';

-- Curve.fi CryptoSwapFactory UpdateFeeReceiver --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswapfactory_update_fee_receiver AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwapFactory UpdateFeeReceiver events';
ALTER TABLE curvefi_cryptoswapfactory_update_fee_receiver
    -- event information --
    ADD COLUMN IF NOT EXISTS old_fee_receiver   String COMMENT 'Previous fee receiver address',
    ADD COLUMN IF NOT EXISTS new_fee_receiver   String COMMENT 'New fee receiver address';

-- Curve.fi CryptoSwapFactory UpdateGaugeImplementation --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswapfactory_update_gauge_implementation AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwapFactory UpdateGaugeImplementation events';
ALTER TABLE curvefi_cryptoswapfactory_update_gauge_implementation
    -- event information --
    ADD COLUMN IF NOT EXISTS old_gauge_implementation String COMMENT 'Previous gauge implementation address',
    ADD COLUMN IF NOT EXISTS new_gauge_implementation String COMMENT 'New gauge implementation address';

-- Curve.fi CryptoSwapFactory UpdatePoolImplementation --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswapfactory_update_pool_implementation AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwapFactory UpdatePoolImplementation events';
ALTER TABLE curvefi_cryptoswapfactory_update_pool_implementation
    -- event information --
    ADD COLUMN IF NOT EXISTS old_pool_implementation String COMMENT 'Previous pool implementation address',
    ADD COLUMN IF NOT EXISTS new_pool_implementation String COMMENT 'New pool implementation address';

-- Curve.fi CryptoSwapFactory UpdateTokenImplementation --
CREATE TABLE IF NOT EXISTS curvefi_cryptoswapfactory_update_token_implementation AS TEMPLATE_LOG
COMMENT 'Curve.fi CryptoSwapFactory UpdateTokenImplementation events';
ALTER TABLE curvefi_cryptoswapfactory_update_token_implementation
    -- event information --
    ADD COLUMN IF NOT EXISTS old_token_implementation String COMMENT 'Previous token implementation address',
    ADD COLUMN IF NOT EXISTS new_token_implementation String COMMENT 'New token implementation address';


