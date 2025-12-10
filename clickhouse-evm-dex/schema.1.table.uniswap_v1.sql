-- Uniswap V1 TokenPurchase (ETH->Token) --
CREATE TABLE IF NOT EXISTS uniswap_v1_token_purchase AS TEMPLATE_LOG
COMMENT 'Uniswap V1 TokenPurchase events (ETH sold for tokens)';
ALTER TABLE uniswap_v1_token_purchase
    -- swap event information --
    ADD COLUMN IF NOT EXISTS buyer              String COMMENT 'buyer wallet address',
    ADD COLUMN IF NOT EXISTS eth_sold           UInt256 COMMENT 'Amount of ETH sold',
    ADD COLUMN IF NOT EXISTS tokens_bought      UInt256 COMMENT 'Amount of tokens bought',

    -- NewExchange --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token              String COMMENT 'Token contract address';

-- Uniswap V1 EthPurchase (Token->ETH) --
CREATE TABLE IF NOT EXISTS uniswap_v1_eth_purchase AS TEMPLATE_LOG
COMMENT 'Uniswap V1 EthPurchase events (tokens sold for ETH)';
ALTER TABLE uniswap_v1_eth_purchase
    -- swap event information --
    ADD COLUMN IF NOT EXISTS buyer              String COMMENT 'buyer wallet address',
    ADD COLUMN IF NOT EXISTS tokens_sold        UInt256 COMMENT 'Amount of tokens sold',
    ADD COLUMN IF NOT EXISTS eth_bought         UInt256 COMMENT 'Amount of ETH bought',

    -- NewExchange --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token              String COMMENT 'Token contract address';

-- Uniswap V1 AddLiquidity --
CREATE TABLE IF NOT EXISTS uniswap_v1_add_liquidity AS TEMPLATE_LOG
COMMENT 'Uniswap V1 AddLiquidity events';
ALTER TABLE uniswap_v1_add_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS eth_amount         UInt256 COMMENT 'Amount of ETH added',
    ADD COLUMN IF NOT EXISTS token_amount       UInt256 COMMENT 'Amount of tokens added',

    -- NewExchange --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token              String COMMENT 'Token contract address';

-- Uniswap V1 RemoveLiquidity --
CREATE TABLE IF NOT EXISTS uniswap_v1_remove_liquidity AS TEMPLATE_LOG
COMMENT 'Uniswap V1 RemoveLiquidity events';
ALTER TABLE uniswap_v1_remove_liquidity
    -- event information --
    ADD COLUMN IF NOT EXISTS provider           String COMMENT 'Liquidity provider address',
    ADD COLUMN IF NOT EXISTS eth_amount         UInt256 COMMENT 'Amount of ETH removed',
    ADD COLUMN IF NOT EXISTS token_amount       UInt256 COMMENT 'Amount of tokens removed',

    -- NewExchange --
    ADD COLUMN IF NOT EXISTS factory           String COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS token              String COMMENT 'Token contract address';

-- Uniswap V1 NewExchange --
CREATE TABLE IF NOT EXISTS uniswap_v1_new_exchange AS TEMPLATE_LOG
COMMENT 'Uniswap V1 NewExchange events';
ALTER TABLE uniswap_v1_new_exchange
    -- event information --
    ADD COLUMN IF NOT EXISTS factory            String MATERIALIZED log_address COMMENT 'Factory contract address',
    ADD COLUMN IF NOT EXISTS exchange           String COMMENT 'Exchange contract address',
    ADD COLUMN IF NOT EXISTS token              String COMMENT 'Token contract address';
