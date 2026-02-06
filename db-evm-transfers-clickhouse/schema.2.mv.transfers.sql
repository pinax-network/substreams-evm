-- Transfers including ERC-20, WETH & Native value from calls & transactions --
CREATE TABLE IF NOT EXISTS transfers (
    -- block --
    block_num                   UInt32,
    block_hash                  String,
    timestamp                   DateTime('UTC'),
    minute                      UInt32 COMMENT 'toRelativeMinuteNum(timestamp)',

    -- transaction --
    tx_index                    UInt32, -- derived from Substreams
    tx_hash                     String,

    -- log --
    log_index                   Nullable(UInt32), -- derived from Substreams
    log_address                 LowCardinality(String),
    log_ordinal                 Nullable(UInt32),
    log_topic0                  LowCardinality(String),

    -- transfer --
    `from`                      String,
    `to`                        String,
    amount                      UInt256,

    -- type --
    transfer_type               Enum8(
        'transfer' = 1,
        'deposit' = 2,
        'withdrawal' = 3,
        'mint' = 4,
        'burn' = 5,
        'issue' = 6,
        'redeem' = 7,
        'transfer_shares' = 8,
        'destroyed_black_funds' = 9,
        'submitted' = 10,
        'shares_burnt' = 11,
        'external_shares_minted' = 12,
        'external_shares_burnt' = 13,
        'token_rebased' = 14
    ),

    -- INDEXES --
    INDEX idx_amount (amount) TYPE minmax,

    -- PROJECTIONS --
    -- count() --
    PROJECTION prj_transfer_type_count ( SELECT transfer_type, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY transfer_type ),
    PROJECTION prj_log_address ( SELECT log_address, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address ),
    PROJECTION prj_from_count ( SELECT `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from` ),
    PROJECTION prj_to_count ( SELECT `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `to` ),
    PROJECTION prj_from_to_count ( SELECT `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY `from`, `to` ),
    PROJECTION prj_log_address_from_count ( SELECT log_address, `from`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `from` ),
    PROJECTION prj_log_address_to_count ( SELECT log_address, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute) GROUP BY log_address, `to` ),
    PROJECTION prj_log_address_to_from_count ( SELECT log_address, `from`, `to`, count(), min(block_num), max(block_num), min(timestamp), max(timestamp), min(minute), max(minute)  GROUP BY log_address, `from`, `to` ),

    -- minute + timestamp --
    PROJECTION prj_tx_hash_by_timestamp ( SELECT tx_hash, minute, timestamp GROUP BY tx_hash, minute, timestamp ),

    -- minute --
    PROJECTION prj_transfer_type_by_minute ( SELECT transfer_type, minute GROUP BY transfer_type, minute ),
    PROJECTION prj_from_by_minute ( SELECT `from`, minute GROUP BY `from`, minute ),
    PROJECTION prj_to_by_minute ( SELECT `to`, minute GROUP BY `to`, minute ),
    PROJECTION prj_log_address_by_minute ( SELECT log_address, minute GROUP BY log_address, minute )
)
ENGINE = MergeTree
ORDER BY (
    minute, timestamp, block_num
)
COMMENT 'Transfers including ERC-20, WETH transfers';

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_erc20_transfers TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,
    log_ordinal,
    log_topic0,

    -- transfer --
    `from`,
    `to`,
    amount,
    'transfer' AS transfer_type
FROM erc20_transfers
WHERE amount > 0 AND `from` != `to`;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_weth_deposit TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- WETH contract
    log_ordinal,
    log_topic0,

    -- transfer (native ETH leg) --
    dst AS `from`,          -- user
    log_address AS `to`,    -- WETH contract
    wad AS amount,
    'deposit' AS transfer_type
FROM weth_deposit
WHERE amount > 0;

CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_weth_withdrawal TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- WETH contract
    log_ordinal,
    log_topic0,

    -- transfer (native ETH leg) --
    log_address AS `from`,  -- WETH contract
    src AS `to`,            -- user
    wad AS amount,
    'withdrawal' AS transfer_type
FROM weth_withdrawal
WHERE amount > 0;

-- USDC Mint (minting tokens to user) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_usdc_mint TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- USDC contract
    log_ordinal,
    log_topic0,

    -- transfer --
    minter AS `from`,       -- minter address
    `to`,                   -- recipient
    amount,
    'mint' AS transfer_type
FROM usdc_mint
WHERE amount > 0;

-- USDC Burn (burning tokens from user) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_usdc_burn TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- USDC contract
    log_ordinal,
    log_topic0,

    -- transfer --
    burner AS `from`,           -- burner address
    log_address AS `to`,        -- contract address (burned)
    amount,
    'burn' AS transfer_type
FROM usdc_burn
WHERE amount > 0;

-- USDT Issue (minting tokens) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_usdt_issue TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- USDT contract
    log_ordinal,
    log_topic0,

    -- transfer --
    '' AS `from`,
    '' AS `to`,
    amount,
    'issue' AS transfer_type
FROM usdt_issue
WHERE amount > 0;

-- USDT Redeem (burning tokens) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_usdt_redeem TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- USDT contract
    log_ordinal,
    log_topic0,

    -- transfer --
    '' AS `from`,
    '' AS `to`,
    amount,
    'redeem' AS transfer_type
FROM usdt_redeem
WHERE amount > 0;

-- USDT DestroyedBlackFunds (destroying blacklisted funds) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_usdt_destroyed_black_funds TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- USDT contract
    log_ordinal,
    log_topic0,

    -- transfer --
    black_listed_user AS `from`,
    '' AS `to`,
    balance AS amount,
    'destroyed_black_funds' AS transfer_type
FROM usdt_destroyed_black_funds
WHERE balance > 0;

-- WBTC Mint (minting tokens to user) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_wbtc_mint TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- WBTC contract
    log_ordinal,
    log_topic0,

    -- transfer --
    '' AS `from`,
    `to`,
    amount,
    'mint' AS transfer_type
FROM wbtc_mint
WHERE amount > 0;

-- WBTC Burn (burning tokens from user) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_wbtc_burn TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- WBTC contract
    log_ordinal,
    log_topic0,

    -- transfer --
    burner AS `from`,
    '' AS `to`,
    value AS amount,
    'burn' AS transfer_type
FROM wbtc_burn
WHERE amount > 0;

-- SAI Mint (minting tokens to user) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_sai_mint TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- SAI contract
    log_ordinal,
    log_topic0,

    -- transfer --
    '' AS `from`,
    guy AS `to`,
    wad AS amount,
    'mint' AS transfer_type
FROM sai_mint
WHERE amount > 0;

-- SAI Burn (burning tokens from user) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_sai_burn TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- SAI contract
    log_ordinal,
    log_topic0,

    -- transfer --
    guy AS `from`,
    '' AS `to`,
    wad AS amount,
    'burn' AS transfer_type
FROM sai_burn
WHERE amount > 0;

-- stETH Submitted (user submitting ETH) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_steth_submitted TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- stETH contract
    log_ordinal,
    log_topic0,

    -- transfer --
    sender AS `from`,
    '' AS `to`,
    amount,
    'submitted' AS transfer_type
FROM steth_submitted
WHERE amount > 0;

-- stETH TransferShares (share transfers) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_steth_transfer_shares TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- stETH contract
    log_ordinal,
    log_topic0,

    -- transfer --
    `from`,
    `to`,
    shares_value AS amount,
    'transfer_shares' AS transfer_type
FROM steth_transfer_shares
WHERE shares_value > 0 AND `from` != `to`;

-- stETH SharesBurnt (burning shares from account) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_steth_shares_burnt TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- stETH contract
    log_ordinal,
    log_topic0,

    -- transfer --
    account AS `from`,
    '' AS `to`,
    shares_amount AS amount,
    'shares_burnt' AS transfer_type
FROM steth_shares_burnt
WHERE shares_amount > 0;

-- stETH ExternalSharesMinted (minting external shares) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_steth_external_shares_minted TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- stETH contract
    log_ordinal,
    log_topic0,

    -- transfer --
    '' AS `from`,
    recipient AS `to`,
    amount_of_shares AS amount,
    'external_shares_minted' AS transfer_type
FROM steth_external_shares_minted
WHERE amount_of_shares > 0;

-- stETH ExternalSharesBurnt (burning external shares) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_steth_external_shares_burnt TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- stETH contract
    log_ordinal,
    log_topic0,

    -- transfer --
    '' AS `from`,
    '' AS `to`,
    amount_of_shares AS amount,
    'external_shares_burnt' AS transfer_type
FROM steth_external_shares_burnt
WHERE amount_of_shares > 0;

-- stETH TokenRebased (rebase event) --
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_transfers_steth_token_rebased TO transfers AS
SELECT
    -- block --
    block_num,
    block_hash,
    timestamp,
    minute,

    -- transaction --
    tx_index,
    tx_hash,

    -- log --
    log_index,
    log_address,  -- stETH contract
    log_ordinal,
    log_topic0,

    -- transfer --
    '' AS `from`,
    '' AS `to`,
    shares_minted_as_fees AS amount,
    'token_rebased' AS transfer_type
FROM steth_token_rebased
WHERE shares_minted_as_fees > 0;
