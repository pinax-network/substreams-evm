-- USDC Mint events --
CREATE TABLE IF NOT EXISTS usdc_mint AS TEMPLATE_LOG
COMMENT 'USDC Mint events';
ALTER TABLE usdc_mint
    -- event --
    ADD COLUMN IF NOT EXISTS minter      String,
    ADD COLUMN IF NOT EXISTS `to`        String,
    ADD COLUMN IF NOT EXISTS amount      UInt256;

-- USDC Burn events --
CREATE TABLE IF NOT EXISTS usdc_burn AS TEMPLATE_LOG
COMMENT 'USDC Burn events';
ALTER TABLE usdc_burn
    -- event --
    ADD COLUMN IF NOT EXISTS burner      String,
    ADD COLUMN IF NOT EXISTS amount      UInt256;

-- USDC AuthorizationUsed events (ERC-3009) --
-- Emitted on transferWithAuthorization / receiveWithAuthorization, the gasless
-- "exact" scheme that x402 payments settle through. `tx_from` is the facilitator
-- that relayed the tx; `authorizer` is the signer (payer). Join to erc20_transfers
-- in the same tx (from = authorizer) to recover payee + amount (see examples/x402_settlements.sql).
CREATE TABLE IF NOT EXISTS usdc_authorization_used AS TEMPLATE_LOG
COMMENT 'ERC-3009 AuthorizationUsed events (x402 gasless settlement rail)';
ALTER TABLE usdc_authorization_used
    -- event --
    ADD COLUMN IF NOT EXISTS authorizer  String,
    ADD COLUMN IF NOT EXISTS nonce       String COMMENT 'bytes32 authorization nonce';

-- USDC AuthorizationCanceled events (ERC-3009) --
CREATE TABLE IF NOT EXISTS usdc_authorization_canceled AS TEMPLATE_LOG
COMMENT 'ERC-3009 AuthorizationCanceled events';
ALTER TABLE usdc_authorization_canceled
    -- event --
    ADD COLUMN IF NOT EXISTS authorizer  String,
    ADD COLUMN IF NOT EXISTS nonce       String COMMENT 'bytes32 authorization nonce';
