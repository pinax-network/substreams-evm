-- ERC20 Transfer Logs --
CREATE TABLE IF NOT EXISTS erc20_transfer AS TEMPLATE_LOG
COMMENT 'ERC20 Token Transfer events';
ALTER TABLE erc20_transfer
    -- transfer --
    ADD COLUMN IF NOT EXISTS `from`        String,
    ADD COLUMN IF NOT EXISTS `to`          String,
    ADD COLUMN IF NOT EXISTS amount        UInt256,

    -- INDEXES --
    ADD INDEX IF NOT EXISTS idx_amount (amount) TYPE minmax GRANULARITY 1,

    -- PROJECTIONS --
    -- count() --
    ADD PROJECTION IF NOT EXISTS prj_from_count ( SELECT `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from` ),
    ADD PROJECTION IF NOT EXISTS prj_to_count ( SELECT `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `to` ),
    ADD PROJECTION IF NOT EXISTS prj_to_from_count ( SELECT `to`, `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `to`, `from` ),
    ADD PROJECTION IF NOT EXISTS prj_from_to_count ( SELECT `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from`, `to` ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_from_count ( SELECT log_address, `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `from` ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_to_count ( SELECT log_address, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `to` ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_to_from_count ( SELECT log_address, `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute)  GROUP BY log_address, `from`, `to` ),
    ADD PROJECTION IF NOT EXISTS prj_log_address_from_to_count ( SELECT log_address, `to`, `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute)  GROUP BY log_address, `to`, `from` ),

    -- minute: log_address | from | to --
    ADD PROJECTION IF NOT EXISTS prj_from_by_minute ( SELECT `from`, minute GROUP BY `from`, minute ),
    ADD PROJECTION IF NOT EXISTS prj_to_by_minute ( SELECT `to`, minute GROUP BY `to`, minute );