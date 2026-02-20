CREATE TABLE IF NOT EXISTS blocks (
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime(0, 'UTC'),

    -- PROJECTIONS --
    PROJECTION prj_block_hash ( SELECT * ORDER BY block_hash ),
    PROJECTION prj_timestamp ( SELECT * ORDER BY timestamp )
)
ENGINE = MergeTree
ORDER BY ( block_num )
COMMENT 'Blocks';