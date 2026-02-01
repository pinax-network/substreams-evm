-- Add new projection for log_address --
ALTER TABLE transfers ON CLUSTER 'dev1'
    ADD PROJECTION IF NOT EXISTS prj_log_address ( SELECT log_address, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address );

-- Materialize the projection for existing data --
ALTER TABLE transfers ON CLUSTER 'dev1'
    MATERIALIZE PROJECTION prj_log_address;