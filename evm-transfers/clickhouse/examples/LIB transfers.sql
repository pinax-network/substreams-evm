WITH
  mb AS (
    SELECT max(block_num) AS max_block_num
    FROM `mainnet:blocks@v0.1.0`.blocks
  ),
  filtered_transfers AS (
    SELECT *
    FROM `mainnet:evm-transfers@v0.2.1`.transfers
    ORDER BY minute DESC, timestamp DESC, block_num DESC
    LIMIT 10
  ),
  block_hashes AS (
    SELECT block_hash
    FROM `mainnet:blocks@v0.1.0`.blocks
    WHERE block_hash IN (SELECT DISTINCT block_hash FROM filtered_transfers)
  )
SELECT
  t.*
FROM filtered_transfers as t
WHERE
  t.block_num > (SELECT max_block_num FROM mb)
  OR t.block_hash IN (SELECT block_hash FROM block_hashes);