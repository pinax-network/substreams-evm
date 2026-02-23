-- CryptoPunk Assigns --
CREATE TABLE IF NOT EXISTS punk_assigns AS TEMPLATE_LOG
COMMENT 'CryptoPunk Assign events';
ALTER TABLE punk_assigns
    -- event --
    ADD COLUMN IF NOT EXISTS `to`                 String,
    ADD COLUMN IF NOT EXISTS punk_index           UInt256;

-- CryptoPunk Transfers --
CREATE TABLE IF NOT EXISTS punk_transfers AS TEMPLATE_LOG
COMMENT 'CryptoPunk Transfer events';
ALTER TABLE punk_transfers
    -- event --
    ADD COLUMN IF NOT EXISTS `from`               String,
    ADD COLUMN IF NOT EXISTS `to`                 String,
    ADD COLUMN IF NOT EXISTS punk_index           UInt256;

-- CryptoPunk Bought --
CREATE TABLE IF NOT EXISTS punk_bought AS TEMPLATE_LOG
COMMENT 'CryptoPunk Bought events';
ALTER TABLE punk_bought
    -- event --
    ADD COLUMN IF NOT EXISTS `from`               String,
    ADD COLUMN IF NOT EXISTS `to`                 String,
    ADD COLUMN IF NOT EXISTS punk_index           UInt256,
    ADD COLUMN IF NOT EXISTS value                UInt256,
    ADD COLUMN IF NOT EXISTS value_is_null        Bool;

-- CryptoPunk BidEntered --
CREATE TABLE IF NOT EXISTS punk_bid_entered AS TEMPLATE_LOG
COMMENT 'CryptoPunk BidEntered events';
ALTER TABLE punk_bid_entered
    -- event --
    ADD COLUMN IF NOT EXISTS `from`               String,
    ADD COLUMN IF NOT EXISTS punk_index           UInt256,
    ADD COLUMN IF NOT EXISTS value                UInt256;

-- CryptoPunk BidWithdrawn --
CREATE TABLE IF NOT EXISTS punk_bid_withdrawn AS TEMPLATE_LOG
COMMENT 'CryptoPunk BidWithdrawn events';
ALTER TABLE punk_bid_withdrawn
    -- event --
    ADD COLUMN IF NOT EXISTS `from`               String,
    ADD COLUMN IF NOT EXISTS punk_index           UInt256,
    ADD COLUMN IF NOT EXISTS value                UInt256;

-- CryptoPunk NoLongerForSale --
CREATE TABLE IF NOT EXISTS punk_no_longer_for_sale AS TEMPLATE_LOG
COMMENT 'CryptoPunk NoLongerForSale events';
ALTER TABLE punk_no_longer_for_sale
    -- event --
    ADD COLUMN IF NOT EXISTS punk_index           UInt256;

-- CryptoPunk PunkOffered --
CREATE TABLE IF NOT EXISTS punk_offered AS TEMPLATE_LOG
COMMENT 'CryptoPunk PunkOffered events';
ALTER TABLE punk_offered
    -- event --
    ADD COLUMN IF NOT EXISTS `to`                 String,
    ADD COLUMN IF NOT EXISTS punk_index           UInt256,
    ADD COLUMN IF NOT EXISTS min_value            UInt256;