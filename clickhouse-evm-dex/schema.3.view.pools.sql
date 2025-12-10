-- Pools VIEW combining state_pools_initialize and state_pools_fees --
CREATE VIEW IF NOT EXISTS pools AS
SELECT
    i.block_num,
    i.block_hash,
    i.timestamp,
    i.minute,
    i.tx_hash,
    i.factory,
    i.pool,
    i.tokens,
    i.token0,
    i.token1,
    i.token2,
    i.token3,
    i.protocol,
    f.fee
FROM state_pools_initialize AS i
LEFT JOIN (
    SELECT 
        pool,
        protocol,
        fee
    FROM state_pools_fees
    WHERE (pool, protocol, block_num) IN (
        SELECT 
            pool,
            protocol,
            max(block_num) AS block_num
        FROM state_pools_fees
        GROUP BY pool, protocol
    )
) AS f ON i.pool = f.pool AND i.protocol = f.protocol;
