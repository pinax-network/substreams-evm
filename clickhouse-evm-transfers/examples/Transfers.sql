-- `/evm/transfers` by from (0.05s) --
WITH minutes AS (
    SELECT minute
    FROM trc20_transfer
    WHERE `from` = 'TM1zzNDZD2DPASbKcgdVoTYhfmYgtfwx9R'
    GROUP BY minute
)
SELECT * FROM trc20_transfer
WHERE `from` = 'TM1zzNDZD2DPASbKcgdVoTYhfmYgtfwx9R'
AND minute IN minutes
ORDER BY minute DESC
LIMIT 10;

-- `/evm/transfers` by from (0.12s) --
WITH from_minutes AS (
    SELECT minute
    FROM trc20_transfer
    WHERE `from` = 'TM1zzNDZD2DPASbKcgdVoTYhfmYgtfwx9R'
    GROUP BY minute
), contract_hours AS (
    SELECT round(minute / 60) as hour
    FROM trc20_transfer
    WHERE log_address = 'TNUC9Qb1rRpS5CbWLmNMxXBjyFoydXjWFR'
    GROUP BY hour
)
SELECT * FROM trc20_transfer
WHERE
    minute IN from_minutes AND
    round(minute / 60) IN contract_hours AND
    `from` = 'TM1zzNDZD2DPASbKcgdVoTYhfmYgtfwx9R' AND
    log_address = 'TNUC9Qb1rRpS5CbWLmNMxXBjyFoydXjWFR'
ORDER BY minute DESC
LIMIT 10;


SELECT * FROM trc20_transfer
WHERE
    `from` = 'TM1zzNDZD2DPASbKcgdVoTYhfmYgtfwx9R' AND
    log_address = 'TNUC9Qb1rRpS5CbWLmNMxXBjyFoydXjWFR'
ORDER BY minute DESC
LIMIT 10;

-- INTERSECT ALL --
SELECT minute
FROM trc20_transfer
WHERE `to` = 'THWuviP5wEiPBLZ1g1iPPiH4kV7FRXWFP1'
GROUP BY minute

INTERSECT ALL

SELECT minute
FROM trc20_transfer
WHERE log_address = 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t'
GROUP BY minute

-- UNION ALL --
WITH minutes_union AS (
    SELECT minute
    FROM trc20_transfer
    WHERE `from` = 'TM1zzNDZD2DPASbKcgdVoTYhfmYgtfwx9R'
    GROUP BY minute

    UNION ALL

    SELECT minute
    FROM trc20_transfer
    WHERE log_address = 'TNUC9Qb1rRpS5CbWLmNMxXBjyFoydXjWFR'
    GROUP BY minute
)
SELECT minute FROM minutes_union
GROUP BY minute
HAVING count() >= 2
ORDER BY minute DESC
LIMIT 1 BY minute
LIMIT 1000;
