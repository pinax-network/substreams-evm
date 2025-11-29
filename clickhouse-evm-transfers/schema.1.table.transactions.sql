-- Transactions with fee information --
CREATE TABLE IF NOT EXISTS transactions AS TEMPLATE_TRANSACTION
COMMENT 'Transactions with fee information';
ALTER TABLE transactions
    -- fees --
    ADD COLUMN IF NOT EXISTS base_fee_per_gas   UInt256,
    ADD COLUMN IF NOT EXISTS transaction_fee    UInt256,
    ADD COLUMN IF NOT EXISTS burn_fee           UInt256,
    ADD COLUMN IF NOT EXISTS fee_paid           UInt256,

    -- INDEXES --
    ADD INDEX IF NOT EXISTS idx_transaction_fee (transaction_fee) TYPE minmax GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_burn_fee (burn_fee) TYPE minmax GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_fee_paid (fee_paid) TYPE minmax GRANULARITY 1,

    -- PROJECTIONS --
    -- minute: tx_from | tx_to --
    ADD PROJECTION IF NOT EXISTS prj_tx_from_by_minute ( SELECT tx_from, minute, count() GROUP BY tx_from, minute ),
    ADD PROJECTION IF NOT EXISTS prj_tx_to_by_minute ( SELECT tx_to, minute, count() GROUP BY tx_to, minute );
