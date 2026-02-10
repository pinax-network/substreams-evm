-- latest swaps by pool --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE pool = '0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640'
    GROUP BY minute
    ORDER BY minute DESC
    LIMIT 10
)
SELECT * FROM swaps
WHERE
    pool = '0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640' AND
    minute IN minutes
ORDER BY minute DESC, timestamp DESC, block_num DESC
LIMIT 10;

-- latest swaps by user --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE user = '0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45'
    GROUP BY minute
    ORDER BY minute DESC
    LIMIT 10
)
SELECT * FROM swaps
WHERE
    user = '0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45' AND
    minute IN minutes
ORDER BY minute DESC, timestamp DESC, block_num DESC
LIMIT 10;

-- latest swaps by protocol --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE protocol = 'uniswap_v3'
    GROUP BY minute
    ORDER BY minute DESC
    LIMIT 10
)
SELECT * FROM swaps
WHERE
    protocol = 'uniswap_v3' AND
    minute IN minutes
ORDER BY minute DESC, timestamp DESC, block_num DESC
LIMIT 10;

-- latest swaps by input token --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE input_contract = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2'
    GROUP BY minute
    ORDER BY minute DESC
    LIMIT 10
)
SELECT * FROM swaps
WHERE
    input_contract = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2' AND
    minute IN minutes
ORDER BY minute DESC, timestamp DESC, block_num DESC
LIMIT 10;

-- latest swaps by pool and user (intersect minutes) --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE pool = '0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640'
    GROUP BY minute

    INTERSECT

    SELECT minute
    FROM swaps
    WHERE user = '0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45'
    GROUP BY minute
)
SELECT * FROM swaps
WHERE
    pool = '0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640' AND
    user = '0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45' AND
    minute IN minutes
ORDER BY minute DESC, timestamp DESC, block_num DESC
LIMIT 10;

-- latest swaps by tx_from --
WITH minutes AS (
    SELECT minute
    FROM swaps
    WHERE tx_from = '0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45'
    GROUP BY minute
    ORDER BY minute DESC
    LIMIT 10
)
SELECT * FROM swaps
WHERE
    tx_from = '0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45' AND
    minute IN minutes
ORDER BY minute DESC, timestamp DESC, block_num DESC
LIMIT 10;
