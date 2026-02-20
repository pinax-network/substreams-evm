-- Trader Joe V2 Swap (Liquidity Book) --
CREATE TABLE IF NOT EXISTS traderjoe_swap AS TEMPLATE_LOG
COMMENT 'Trader Joe V2 LBPair Swap events';
ALTER TABLE traderjoe_swap
    -- swap event information --
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'sender wallet address',
    ADD COLUMN IF NOT EXISTS `to`               String COMMENT 'recipient wallet address',
    ADD COLUMN IF NOT EXISTS id                 UInt32 COMMENT 'Active bin id',
    ADD COLUMN IF NOT EXISTS amount_in_x        UInt256 COMMENT 'tokenX amount in (decoded from packed bytes32)',
    ADD COLUMN IF NOT EXISTS amount_in_y        UInt256 COMMENT 'tokenY amount in (decoded from packed bytes32)',
    ADD COLUMN IF NOT EXISTS amount_out_x       UInt256 COMMENT 'tokenX amount out (decoded from packed bytes32)',
    ADD COLUMN IF NOT EXISTS amount_out_y       UInt256 COMMENT 'tokenY amount out (decoded from packed bytes32)',
    ADD COLUMN IF NOT EXISTS volatility_accumulator UInt32 COMMENT 'Volatility accumulator',
    ADD COLUMN IF NOT EXISTS total_fees_x       UInt256 COMMENT 'tokenX total fees',
    ADD COLUMN IF NOT EXISTS total_fees_y       UInt256 COMMENT 'tokenY total fees',
    ADD COLUMN IF NOT EXISTS protocol_fees_x    UInt256 COMMENT 'tokenX protocol fees',
    ADD COLUMN IF NOT EXISTS protocol_fees_y    UInt256 COMMENT 'tokenY protocol fees',

    -- LbPairCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'tokenX contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'tokenY contract address',
    ADD COLUMN IF NOT EXISTS bin_step           UInt32 COMMENT 'Bin step';

-- Trader Joe V2 DepositedToBins --
CREATE TABLE IF NOT EXISTS traderjoe_deposited_to_bins AS TEMPLATE_LOG
COMMENT 'Trader Joe V2 DepositedToBins events';
ALTER TABLE traderjoe_deposited_to_bins
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS `to`               String COMMENT 'Recipient address',

    -- LbPairCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'tokenX contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'tokenY contract address',
    ADD COLUMN IF NOT EXISTS bin_step           UInt32 COMMENT 'Bin step';

-- Trader Joe V2 WithdrawnFromBins --
CREATE TABLE IF NOT EXISTS traderjoe_withdrawn_from_bins AS TEMPLATE_LOG
COMMENT 'Trader Joe V2 WithdrawnFromBins events';
ALTER TABLE traderjoe_withdrawn_from_bins
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS `to`               String COMMENT 'Recipient address',

    -- LbPairCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'tokenX contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'tokenY contract address',
    ADD COLUMN IF NOT EXISTS bin_step           UInt32 COMMENT 'Bin step';

-- Trader Joe V2 CompositionFees --
CREATE TABLE IF NOT EXISTS traderjoe_composition_fees AS TEMPLATE_LOG
COMMENT 'Trader Joe V2 CompositionFees events';
ALTER TABLE traderjoe_composition_fees
    ADD COLUMN IF NOT EXISTS sender             String COMMENT 'Sender address',
    ADD COLUMN IF NOT EXISTS id                 UInt32 COMMENT 'Bin id',
    ADD COLUMN IF NOT EXISTS total_fees_x       UInt256 COMMENT 'tokenX total fees',
    ADD COLUMN IF NOT EXISTS total_fees_y       UInt256 COMMENT 'tokenY total fees',
    ADD COLUMN IF NOT EXISTS protocol_fees_x    UInt256 COMMENT 'tokenX protocol fees',
    ADD COLUMN IF NOT EXISTS protocol_fees_y    UInt256 COMMENT 'tokenY protocol fees',

    -- LbPairCreated --
    ADD COLUMN IF NOT EXISTS factory            String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token0             String COMMENT 'tokenX contract address',
    ADD COLUMN IF NOT EXISTS token1             String COMMENT 'tokenY contract address',
    ADD COLUMN IF NOT EXISTS bin_step           UInt32 COMMENT 'Bin step';

-- Trader Joe V2 LbPairCreated --
CREATE TABLE IF NOT EXISTS traderjoe_lb_pair_created AS TEMPLATE_LOG
COMMENT 'Trader Joe V2 LbPairCreated events';
ALTER TABLE traderjoe_lb_pair_created
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token_x            String COMMENT 'tokenX contract address',
    ADD COLUMN IF NOT EXISTS token_y            String COMMENT 'tokenY contract address',
    ADD COLUMN IF NOT EXISTS bin_step           UInt32 COMMENT 'Bin step',
    ADD COLUMN IF NOT EXISTS lb_pair            String COMMENT 'LBPair contract address',
    ADD COLUMN IF NOT EXISTS pid                UInt32 COMMENT 'Pair ID';
