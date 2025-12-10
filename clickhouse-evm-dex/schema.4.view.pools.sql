-- Pools VIEW combining state_pools_initialize and state_pools_fees --
CREATE VIEW IF NOT EXISTS pools AS
SELECT
    -- tokens --
    t.factory as factory,
    t.pool as pool,
    t.protocol as protocol,
    t.input_contract as input_contract,
    t.output_contract as output_contract,

    -- initialize --
    i.block_num as initialize_block_num,
    i.timestamp as initialize_timestamp,
    i.tx_hash as initialize_tx_hash,

    -- fees --
    f.fee as fee,
    f.block_num as fee_block_num,
    f.timestamp as fee_timestamp,
    f.tx_hash as fee_tx_hash

FROM state_pools_tokens AS t
LEFT JOIN state_pools_fees AS f ON t.pool = f.pool AND t.factory = f.factory
LEFT JOIN state_pools_initialize AS i ON t.pool = i.pool AND t.factory = i.factory;