-- =============================================================================
-- x402 settlements from ERC-3009 AuthorizationUsed events
-- =============================================================================
-- ERC-3009 `AuthorizationUsed(authorizer, nonce)` is the on-chain footprint of a
-- gasless "exact" transfer — the scheme x402 payments settle through. The event
-- carries no payee or amount; recover them from the token `Transfer` the same call
-- emits. FiatToken emits AuthorizationUsed *immediately before* that Transfer, so the
-- settling Transfer is the nearest one *after* the authorization (same tx, same token,
-- from = authorizer). ASOF JOIN picks that nearest-following Transfer, which stays
-- correct even when a relayer batches several same-payer authorizations in one tx.
--
--   payer       = authorization.authorizer   -- the signer
--   facilitator = authorization.tx_from      -- submitter of the tx (a relayer for
--                                               transferWithAuthorization; equals the
--                                               payee for receiveWithAuthorization)
--   payee       = transfer.to
--   amount      = transfer.amount            -- token-native decimals
--   token       = log_address
--
-- NOTE: AuthorizationUsed is matched by event signature across ALL ERC-3009 tokens
-- (USDC, EURC, PYUSD, USDT, …), so group/filter by `token` when a single currency is
-- meant. Amounts below divide by 1e6 (true for USDC/EURC/PYUSD/USDT — all 6 decimals).
-- NOTE: these base tables carry a 1-day TTL (schema.0.templates.sql), matching the
-- other token-event tables; multi-day aggregates reflect only the recent retention
-- window on a stock deployment.
-- =============================================================================

-- 1) Recover recent settlements (payer -> payee -> amount + facilitator).
SELECT
    a.timestamp    AS timestamp,
    a.tx_hash      AS tx_hash,
    a.log_address  AS token,
    a.tx_from      AS facilitator,
    a.authorizer   AS payer,
    t.`to`         AS payee,
    t.amount       AS amount,
    a.nonce        AS nonce
FROM usdc_authorization_used AS a
ASOF JOIN erc20_transfers AS t
    ON  a.tx_hash     = t.tx_hash
    AND a.log_address = t.log_address
    AND a.authorizer  = t.`from`
    AND a.log_ordinal < t.log_ordinal   -- Transfer is emitted just AFTER the AuthorizationUsed
ORDER BY a.timestamp DESC
LIMIT 100;

-- 2) Facilitator league table. The `settlements` CTE reuses the ASOF pairing so each
--    authorization maps to exactly one transfer (no double-count in batched txs).
WITH settlements AS (
    SELECT a.tx_from AS facilitator, a.log_address AS token, a.authorizer AS payer, t.amount AS amount
    FROM usdc_authorization_used AS a
    ASOF JOIN erc20_transfers AS t
        ON a.tx_hash = t.tx_hash AND a.log_address = t.log_address
       AND a.authorizer = t.`from` AND a.log_ordinal < t.log_ordinal
)
SELECT
    facilitator,
    token,
    count()                    AS settlements,
    uniqExact(payer)           AS unique_payers,
    round(sum(amount) / 1e6, 2) AS volume_6dp
FROM settlements
GROUP BY facilitator, token
ORDER BY settlements DESC
LIMIT 25;

-- 3) Top payees (merchants / receiving agents) by settlement count.
WITH settlements AS (
    SELECT t.`to` AS payee, a.log_address AS token, t.amount AS amount
    FROM usdc_authorization_used AS a
    ASOF JOIN erc20_transfers AS t
        ON a.tx_hash = t.tx_hash AND a.log_address = t.log_address
       AND a.authorizer = t.`from` AND a.log_ordinal < t.log_ordinal
)
SELECT
    payee,
    token,
    count()                     AS payments,
    round(sum(amount) / 1e6, 2) AS received_6dp,
    round(avg(amount) / 1e6, 4) AS avg_payment_6dp
FROM settlements
GROUP BY payee, token
ORDER BY payments DESC
LIMIT 25;

-- 4) Daily settlement volume, per token.
WITH settlements AS (
    SELECT a.timestamp AS timestamp, a.log_address AS token, t.amount AS amount
    FROM usdc_authorization_used AS a
    ASOF JOIN erc20_transfers AS t
        ON a.tx_hash = t.tx_hash AND a.log_address = t.log_address
       AND a.authorizer = t.`from` AND a.log_ordinal < t.log_ordinal
)
SELECT
    toDate(timestamp)           AS day,
    token,
    count()                     AS settlements,
    round(sum(amount) / 1e6, 2) AS volume_6dp
FROM settlements
GROUP BY day, token
ORDER BY day DESC, volume_6dp DESC;
