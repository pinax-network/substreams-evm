-- EXTENDED transaction with more gas fields
CREATE TABLE IF NOT EXISTS transactions AS TEMPLATE_TRANSACTION
COMMENT 'Transactions with native value & gas/fee fields';

-- Native Transfers --
CREATE TABLE IF NOT EXISTS calls AS TEMPLATE_CALL
COMMENT 'Calls with native value transfers';