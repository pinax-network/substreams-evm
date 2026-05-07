-- Normalized x402 payment settlement logs written directly by `evm-x402`
CREATE TABLE IF NOT EXISTS x402_payments AS TEMPLATE_LOG
COMMENT 'Settled x402 payment events on EVM chains';

ALTER TABLE x402_payments
    -- payment --
    ADD COLUMN IF NOT EXISTS asset                         LowCardinality(String),
    ADD COLUMN IF NOT EXISTS payer                         String,
    ADD COLUMN IF NOT EXISTS recipient                     String,
    ADD COLUMN IF NOT EXISTS facilitator                   String,
    ADD COLUMN IF NOT EXISTS amount                        UInt256,
    ADD COLUMN IF NOT EXISTS nonce                         String,
    ADD COLUMN IF NOT EXISTS transfer_method               LowCardinality(String),
    ADD COLUMN IF NOT EXISTS settlement_source             LowCardinality(String),
    ADD COLUMN IF NOT EXISTS scheme                        LowCardinality(String),
    ADD COLUMN IF NOT EXISTS valid_after                   UInt256,
    ADD COLUMN IF NOT EXISTS valid_before                  UInt256,
    ADD COLUMN IF NOT EXISTS facilitator_allowlist_matched  Bool,
    ADD COLUMN IF NOT EXISTS confidence                    LowCardinality(String);

ALTER TABLE x402_payments
    -- INDEXES --
    ADD INDEX IF NOT EXISTS idx_amount (amount) TYPE minmax GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_asset (asset) TYPE set(1024) GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_facilitator (facilitator) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_recipient (recipient) TYPE bloom_filter GRANULARITY 1,
    ADD INDEX IF NOT EXISTS idx_payer (payer) TYPE bloom_filter GRANULARITY 1;

ALTER TABLE x402_payments
    -- PROJECTIONS --
    -- count() --
    ADD PROJECTION IF NOT EXISTS prj_facilitator_count ( SELECT facilitator, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY facilitator ),
    ADD PROJECTION IF NOT EXISTS prj_recipient_count ( SELECT recipient, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY recipient ),
    ADD PROJECTION IF NOT EXISTS prj_payer_count ( SELECT payer, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY payer ),
    ADD PROJECTION IF NOT EXISTS prj_facilitator_recipient_count ( SELECT facilitator, recipient, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY facilitator, recipient ),
    ADD PROJECTION IF NOT EXISTS prj_facilitator_payer_count ( SELECT facilitator, payer, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY facilitator, payer ),
    ADD PROJECTION IF NOT EXISTS prj_facilitator_recipient_payer_count ( SELECT facilitator, recipient, payer, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY facilitator, recipient, payer ),
    ADD PROJECTION IF NOT EXISTS prj_asset_count ( SELECT asset, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY asset ),
    ADD PROJECTION IF NOT EXISTS prj_transfer_method_count ( SELECT transfer_method, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY transfer_method ),
    ADD PROJECTION IF NOT EXISTS prj_settlement_source_count ( SELECT settlement_source, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY settlement_source ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_count ( SELECT log_address, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_facilitator_count ( SELECT log_address, facilitator, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, facilitator ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_recipient_count ( SELECT log_address, recipient, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, recipient ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_facilitator_recipient_count ( SELECT log_address, facilitator, recipient, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, facilitator, recipient ),

    -- minute + timestamp --
    ADD PROJECTION IF NOT EXISTS prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp GROUP BY tx_hash, minute, timestamp ),

    -- minute --
    ADD PROJECTION IF NOT EXISTS prj_facilitator_by_minute ( SELECT facilitator, minute GROUP BY facilitator, minute ),
    ADD PROJECTION IF NOT EXISTS prj_recipient_by_minute ( SELECT recipient, minute GROUP BY recipient, minute ),
    ADD PROJECTION IF NOT EXISTS prj_payer_by_minute ( SELECT payer, minute GROUP BY payer, minute ),
    ADD PROJECTION IF NOT EXISTS prj_facilitator_recipient_by_minute ( SELECT facilitator, recipient, minute GROUP BY facilitator, recipient, minute ),
    ADD PROJECTION IF NOT EXISTS prj_asset_by_minute ( SELECT asset, minute GROUP BY asset, minute ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_by_minute ( SELECT log_address, minute GROUP BY log_address, minute );
