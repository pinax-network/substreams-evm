-- SunPump TokenPurchased --
CREATE TABLE IF NOT EXISTS sunpump_token_purchased AS TEMPLATE_LOG
COMMENT 'SunPump TokenPurchased and TokenSold swap events';
ALTER TABLE sunpump_token_purchased
    -- swap event information --
    ADD COLUMN IF NOT EXISTS buyer                  String COMMENT 'User wallet address',
    ADD COLUMN IF NOT EXISTS trx_amount             UInt256 COMMENT 'Amount of input tokens swapped',
    ADD COLUMN IF NOT EXISTS token                  LowCardinality(String) COMMENT 'Token contract address',
    ADD COLUMN IF NOT EXISTS token_amount           UInt256 COMMENT 'Amount of output tokens received',
    ADD COLUMN IF NOT EXISTS fee                    UInt256 COMMENT 'Swap fee amount',
    ADD COLUMN IF NOT EXISTS token_reserve          UInt256 COMMENT 'Token reserve after swap (only for purchases)',

    -- TokenCreate --
    ADD COLUMN IF NOT EXISTS factory                String COMMENT 'Factory contract address';

-- SunPump TokenSold --
CREATE TABLE IF NOT EXISTS sunpump_token_sold AS TEMPLATE_LOG
COMMENT 'SunPump TokenPurchased and TokenSold swap events';
ALTER TABLE sunpump_token_sold
    -- swap event information --
    ADD COLUMN IF NOT EXISTS seller             String COMMENT 'User wallet address',
    ADD COLUMN IF NOT EXISTS token              LowCardinality(String) COMMENT 'Token contract address',
    ADD COLUMN IF NOT EXISTS token_amount       UInt256 COMMENT 'Amount of output tokens received',
    ADD COLUMN IF NOT EXISTS trx_amount         UInt256 COMMENT 'Amount of input tokens swapped',
    ADD COLUMN IF NOT EXISTS fee                UInt256 COMMENT 'Swap fee amount',

    -- TokenCreate --
    ADD COLUMN IF NOT EXISTS factory                String COMMENT 'Factory contract address';

-- SunPump LaunchPending --
CREATE TABLE IF NOT EXISTS sunpump_launch_pending AS TEMPLATE_LOG
COMMENT 'SunPump LaunchPending events';
ALTER TABLE sunpump_launch_pending
    -- event information --
    ADD COLUMN IF NOT EXISTS token              LowCardinality(String) COMMENT 'Token contract address';

-- SunPump LauncherChanged --
CREATE TABLE IF NOT EXISTS sunpump_launcher_changed AS TEMPLATE_LOG
COMMENT 'SunPump LauncherChanged events';
ALTER TABLE sunpump_launcher_changed
    -- event information --
    ADD COLUMN IF NOT EXISTS old_launcher       String COMMENT 'Old launcher address',
    ADD COLUMN IF NOT EXISTS new_launcher       String COMMENT 'New launcher address';

-- SunPump MinTxFeeSet --
CREATE TABLE IF NOT EXISTS sunpump_min_tx_fee_set AS TEMPLATE_LOG
COMMENT 'SunPump MinTxFeeSet events';
ALTER TABLE sunpump_min_tx_fee_set
    -- event information --
    ADD COLUMN IF NOT EXISTS old_fee            UInt256 COMMENT 'Old minimum transaction fee',
    ADD COLUMN IF NOT EXISTS new_fee            UInt256 COMMENT 'New minimum transaction fee';

-- SunPump MintFeeSet --
CREATE TABLE IF NOT EXISTS sunpump_mint_fee_set AS TEMPLATE_LOG
COMMENT 'SunPump MintFeeSet events';
ALTER TABLE sunpump_mint_fee_set
    -- event information --
    ADD COLUMN IF NOT EXISTS old_fee            UInt256 COMMENT 'Old mint fee',
    ADD COLUMN IF NOT EXISTS new_fee            UInt256 COMMENT 'New mint fee';

-- SunPump OperatorChanged --
CREATE TABLE IF NOT EXISTS sunpump_operator_changed AS TEMPLATE_LOG
COMMENT 'SunPump OperatorChanged events';
ALTER TABLE sunpump_operator_changed
    -- event information --
    ADD COLUMN IF NOT EXISTS old_operator       String COMMENT 'Old operator address',
    ADD COLUMN IF NOT EXISTS new_operator       String COMMENT 'New operator address';

-- SunPump OwnerChanged --
CREATE TABLE IF NOT EXISTS sunpump_owner_changed AS TEMPLATE_LOG
COMMENT 'SunPump OwnerChanged events';
ALTER TABLE sunpump_owner_changed
    -- event information --
    ADD COLUMN IF NOT EXISTS old_owner          String COMMENT 'Old owner address',
    ADD COLUMN IF NOT EXISTS new_owner          String COMMENT 'New owner address';

-- SunPump PendingOwnerSet --
CREATE TABLE IF NOT EXISTS sunpump_pending_owner_set AS TEMPLATE_LOG
COMMENT 'SunPump PendingOwnerSet events';
ALTER TABLE sunpump_pending_owner_set
    -- event information --
    ADD COLUMN IF NOT EXISTS old_pending_owner  String COMMENT 'Old pending owner address',
    ADD COLUMN IF NOT EXISTS new_pending_owner  String COMMENT 'New pending owner address';

-- SunPump PurchaseFeeSet --
CREATE TABLE IF NOT EXISTS sunpump_purchase_fee_set AS TEMPLATE_LOG
COMMENT 'SunPump PurchaseFeeSet events';
ALTER TABLE sunpump_purchase_fee_set
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS old_fee            UInt256 COMMENT 'Old purchase fee',
    ADD COLUMN IF NOT EXISTS new_fee            UInt256 COMMENT 'New purchase fee';

-- SunPump SaleFeeSet --
CREATE TABLE IF NOT EXISTS sunpump_sale_fee_set AS TEMPLATE_LOG
COMMENT 'SunPump SaleFeeSet events';
ALTER TABLE sunpump_sale_fee_set
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS old_fee            UInt256 COMMENT 'Old sale fee',
    ADD COLUMN IF NOT EXISTS new_fee            UInt256 COMMENT 'New sale fee';

-- SunPump TokenCreate --
CREATE TABLE IF NOT EXISTS sunpump_token_create AS TEMPLATE_LOG
COMMENT 'SunPump TokenCreate events';
ALTER TABLE sunpump_token_create
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token_address      LowCardinality(String) COMMENT 'Token contract address',
    ADD COLUMN IF NOT EXISTS token_index        UInt256 COMMENT 'Token index',
    ADD COLUMN IF NOT EXISTS creator            String COMMENT 'Creator address';

-- SunPump TokenCreateLegacy --
CREATE TABLE IF NOT EXISTS sunpump_token_create_legacy AS TEMPLATE_LOG
COMMENT 'SunPump TokenCreate - Legacy events';
ALTER TABLE sunpump_token_create_legacy
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token_address      LowCardinality(String) COMMENT 'Token contract address',
    ADD COLUMN IF NOT EXISTS creator            String COMMENT 'Creator address',
    ADD COLUMN IF NOT EXISTS nft_max_supply     UInt256 COMMENT 'Max NFT supply',
    ADD COLUMN IF NOT EXISTS nft_threshold      UInt256 COMMENT 'NFT threshold',
    ADD COLUMN IF NOT EXISTS name               String COMMENT 'Token name',
    ADD COLUMN IF NOT EXISTS symbol            String COMMENT 'Token symbol';

-- SunPump TokenLaunched --
CREATE TABLE IF NOT EXISTS sunpump_token_launched AS TEMPLATE_LOG
COMMENT 'SunPump TokenLaunched events';
ALTER TABLE sunpump_token_launched
    -- event information --
    ADD COLUMN IF NOT EXISTS token              LowCardinality(String) COMMENT 'Token contract address';
