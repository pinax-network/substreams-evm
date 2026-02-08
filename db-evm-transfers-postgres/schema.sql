-- Blocks table for PostgreSQL
CREATE TABLE IF NOT EXISTS blocks (
    block_num            INTEGER NOT NULL PRIMARY KEY,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_block_hash ON blocks (block_hash);
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks (timestamp);

-- Transactions table for PostgreSQL
CREATE TABLE IF NOT EXISTS transactions (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index)
);

CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions (timestamp);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_hash ON transactions (tx_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_from ON transactions (tx_from);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_to ON transactions (tx_to);

-- Calls table for PostgreSQL
CREATE TABLE IF NOT EXISTS calls (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- call --
    call_index           INTEGER NOT NULL,
    call_begin_ordinal   BIGINT NOT NULL,
    call_end_ordinal     BIGINT NOT NULL,
    call_caller          TEXT NOT NULL,
    call_address         TEXT NOT NULL,
    call_value           NUMERIC NOT NULL,
    call_gas_consumed    BIGINT NOT NULL,
    call_gas_limit       BIGINT NOT NULL,
    call_depth           INTEGER NOT NULL,
    call_parent_index    INTEGER NOT NULL,
    call_type            TEXT NOT NULL,

    PRIMARY KEY (block_num, tx_index, call_index)
);

CREATE INDEX IF NOT EXISTS idx_calls_timestamp ON calls (timestamp);
CREATE INDEX IF NOT EXISTS idx_calls_tx_hash ON calls (tx_hash);
CREATE INDEX IF NOT EXISTS idx_calls_call_caller ON calls (call_caller);
CREATE INDEX IF NOT EXISTS idx_calls_call_address ON calls (call_address);

-- ERC20 Transfers table for PostgreSQL
CREATE TABLE IF NOT EXISTS erc20_transfers (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- transfer --
    "from"               TEXT NOT NULL,
    "to"                 TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_erc20_transfers_timestamp ON erc20_transfers (timestamp);
CREATE INDEX IF NOT EXISTS idx_erc20_transfers_log_address ON erc20_transfers (log_address);
CREATE INDEX IF NOT EXISTS idx_erc20_transfers_from ON erc20_transfers ("from");
CREATE INDEX IF NOT EXISTS idx_erc20_transfers_to ON erc20_transfers ("to");

-- ERC20 Approvals table for PostgreSQL
CREATE TABLE IF NOT EXISTS erc20_approvals (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- approval --
    owner                TEXT NOT NULL,
    spender              TEXT NOT NULL,
    value                NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_erc20_approvals_timestamp ON erc20_approvals (timestamp);
CREATE INDEX IF NOT EXISTS idx_erc20_approvals_log_address ON erc20_approvals (log_address);
CREATE INDEX IF NOT EXISTS idx_erc20_approvals_owner ON erc20_approvals (owner);
CREATE INDEX IF NOT EXISTS idx_erc20_approvals_spender ON erc20_approvals (spender);

-- WETH Deposit table for PostgreSQL
CREATE TABLE IF NOT EXISTS weth_deposit (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- deposit --
    dst                  TEXT NOT NULL,
    wad                  NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_weth_deposit_timestamp ON weth_deposit (timestamp);
CREATE INDEX IF NOT EXISTS idx_weth_deposit_dst ON weth_deposit (dst);

-- WETH Withdrawal table for PostgreSQL
CREATE TABLE IF NOT EXISTS weth_withdrawal (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- withdrawal --
    src                  TEXT NOT NULL,
    wad                  NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_weth_withdrawal_timestamp ON weth_withdrawal (timestamp);
CREATE INDEX IF NOT EXISTS idx_weth_withdrawal_src ON weth_withdrawal (src);

-- USDC Mint table for PostgreSQL
CREATE TABLE IF NOT EXISTS usdc_mint (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- mint --
    minter               TEXT NOT NULL,
    "to"                 TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdc_mint_timestamp ON usdc_mint (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdc_mint_minter ON usdc_mint (minter);
CREATE INDEX IF NOT EXISTS idx_usdc_mint_to ON usdc_mint ("to");

-- USDC Burn table for PostgreSQL
CREATE TABLE IF NOT EXISTS usdc_burn (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- burn --
    burner               TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdc_burn_timestamp ON usdc_burn (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdc_burn_burner ON usdc_burn (burner);

-- USDT Issue table for PostgreSQL
CREATE TABLE IF NOT EXISTS usdt_issue (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- issue --
    owner                TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_issue_timestamp ON usdt_issue (timestamp);

-- USDT Redeem table for PostgreSQL
CREATE TABLE IF NOT EXISTS usdt_redeem (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- redeem --
    owner                TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_redeem_timestamp ON usdt_redeem (timestamp);

-- USDT DestroyedBlackFunds table for PostgreSQL
CREATE TABLE IF NOT EXISTS usdt_destroyed_black_funds (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- destroyed black funds --
    black_listed_user    TEXT NOT NULL,
    balance              NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_destroyed_black_funds_timestamp ON usdt_destroyed_black_funds (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_destroyed_black_funds_black_listed_user ON usdt_destroyed_black_funds (black_listed_user);

-- USDT BlockPlaced table for PostgreSQL (v0.8.4)
CREATE TABLE IF NOT EXISTS usdt_block_placed (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- block placed --
    "user"               TEXT NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_block_placed_timestamp ON usdt_block_placed (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_block_placed_user ON usdt_block_placed ("user");

-- USDT BlockReleased table for PostgreSQL (v0.8.4)
CREATE TABLE IF NOT EXISTS usdt_block_released (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- block released --
    "user"               TEXT NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_block_released_timestamp ON usdt_block_released (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_block_released_user ON usdt_block_released ("user");

-- USDT Mint table for PostgreSQL (v0.8.4)
CREATE TABLE IF NOT EXISTS usdt_mint (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- mint --
    destination          TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_mint_timestamp ON usdt_mint (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_mint_destination ON usdt_mint (destination);

-- USDT DestroyedBlockedFunds table for PostgreSQL (v0.8.4)
CREATE TABLE IF NOT EXISTS usdt_destroyed_blocked_funds (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- destroyed blocked funds --
    blocked_user         TEXT NOT NULL,
    balance              NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_destroyed_blocked_funds_timestamp ON usdt_destroyed_blocked_funds (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_destroyed_blocked_funds_blocked_user ON usdt_destroyed_blocked_funds (blocked_user);

-- USDT NewPrivilegedContract table for PostgreSQL (v0.8.4)
CREATE TABLE IF NOT EXISTS usdt_new_privileged_contract (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- new privileged contract --
    contract             TEXT NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_new_privileged_contract_timestamp ON usdt_new_privileged_contract (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_new_privileged_contract_contract ON usdt_new_privileged_contract (contract);

-- USDT RemovedPrivilegedContract table for PostgreSQL (v0.8.4)
CREATE TABLE IF NOT EXISTS usdt_removed_privileged_contract (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- removed privileged contract --
    contract             TEXT NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_removed_privileged_contract_timestamp ON usdt_removed_privileged_contract (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_removed_privileged_contract_contract ON usdt_removed_privileged_contract (contract);

-- USDT LogSwapin table for PostgreSQL (swap_asset)
CREATE TABLE IF NOT EXISTS usdt_log_swapin (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- log swapin --
    txhash               TEXT NOT NULL,
    account              TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_log_swapin_timestamp ON usdt_log_swapin (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_log_swapin_account ON usdt_log_swapin (account);

-- USDT LogSwapout table for PostgreSQL (swap_asset)
CREATE TABLE IF NOT EXISTS usdt_log_swapout (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- log swapout --
    account              TEXT NOT NULL,
    bindaddr             TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_log_swapout_timestamp ON usdt_log_swapout (timestamp);
CREATE INDEX IF NOT EXISTS idx_usdt_log_swapout_account ON usdt_log_swapout (account);

-- USDT LogChangeDCRMOwner table for PostgreSQL (swap_asset)
CREATE TABLE IF NOT EXISTS usdt_log_change_dcrm_owner (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- log change dcrm owner --
    old_owner            TEXT NOT NULL,
    new_owner            TEXT NOT NULL,
    effective_height     NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_usdt_log_change_dcrm_owner_timestamp ON usdt_log_change_dcrm_owner (timestamp);

-- WBTC Mint table for PostgreSQL
CREATE TABLE IF NOT EXISTS wbtc_mint (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- mint --
    "to"                 TEXT NOT NULL,
    amount               NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_wbtc_mint_timestamp ON wbtc_mint (timestamp);
CREATE INDEX IF NOT EXISTS idx_wbtc_mint_to ON wbtc_mint ("to");

-- WBTC Burn table for PostgreSQL
CREATE TABLE IF NOT EXISTS wbtc_burn (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- burn --
    burner               TEXT NOT NULL,
    value                NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_wbtc_burn_timestamp ON wbtc_burn (timestamp);
CREATE INDEX IF NOT EXISTS idx_wbtc_burn_burner ON wbtc_burn (burner);

-- SAI Mint table for PostgreSQL
CREATE TABLE IF NOT EXISTS sai_mint (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- mint --
    guy                  TEXT NOT NULL,
    wad                  NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_sai_mint_timestamp ON sai_mint (timestamp);
CREATE INDEX IF NOT EXISTS idx_sai_mint_guy ON sai_mint (guy);

-- SAI Burn table for PostgreSQL
CREATE TABLE IF NOT EXISTS sai_burn (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- burn --
    guy                  TEXT NOT NULL,
    wad                  NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_sai_burn_timestamp ON sai_burn (timestamp);
CREATE INDEX IF NOT EXISTS idx_sai_burn_guy ON sai_burn (guy);

-- stETH Submitted table for PostgreSQL
CREATE TABLE IF NOT EXISTS steth_submitted (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- submitted --
    sender               TEXT NOT NULL,
    amount               NUMERIC NOT NULL,
    referral             TEXT NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_steth_submitted_timestamp ON steth_submitted (timestamp);
CREATE INDEX IF NOT EXISTS idx_steth_submitted_sender ON steth_submitted (sender);

-- stETH TokenRebased table for PostgreSQL
CREATE TABLE IF NOT EXISTS steth_token_rebased (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- token rebased --
    report_timestamp       NUMERIC NOT NULL,
    time_elapsed           NUMERIC NOT NULL,
    pre_total_shares       NUMERIC NOT NULL,
    pre_total_ether        NUMERIC NOT NULL,
    post_total_shares      NUMERIC NOT NULL,
    post_total_ether       NUMERIC NOT NULL,
    shares_minted_as_fees  NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_steth_token_rebased_timestamp ON steth_token_rebased (timestamp);

-- stETH SharesBurnt table for PostgreSQL
CREATE TABLE IF NOT EXISTS steth_shares_burnt (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- shares burnt --
    account                    TEXT NOT NULL,
    pre_rebase_token_amount    NUMERIC NOT NULL,
    post_rebase_token_amount   NUMERIC NOT NULL,
    shares_amount              NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_steth_shares_burnt_timestamp ON steth_shares_burnt (timestamp);
CREATE INDEX IF NOT EXISTS idx_steth_shares_burnt_account ON steth_shares_burnt (account);

-- stETH TransferShares table for PostgreSQL
CREATE TABLE IF NOT EXISTS steth_transfer_shares (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- transfer shares --
    "from"               TEXT NOT NULL,
    "to"                 TEXT NOT NULL,
    shares_value         NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_steth_transfer_shares_timestamp ON steth_transfer_shares (timestamp);
CREATE INDEX IF NOT EXISTS idx_steth_transfer_shares_from ON steth_transfer_shares ("from");
CREATE INDEX IF NOT EXISTS idx_steth_transfer_shares_to ON steth_transfer_shares ("to");

-- stETH ExternalSharesMinted table for PostgreSQL
CREATE TABLE IF NOT EXISTS steth_external_shares_minted (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- external shares minted --
    recipient            TEXT NOT NULL,
    amount_of_shares     NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_steth_external_shares_minted_timestamp ON steth_external_shares_minted (timestamp);
CREATE INDEX IF NOT EXISTS idx_steth_external_shares_minted_recipient ON steth_external_shares_minted (recipient);

-- stETH ExternalSharesBurnt table for PostgreSQL
CREATE TABLE IF NOT EXISTS steth_external_shares_burnt (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- transaction --
    tx_index             INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    tx_from              TEXT NOT NULL,
    tx_to                TEXT,
    tx_nonce             BIGINT NOT NULL,
    tx_gas_price         NUMERIC NOT NULL,
    tx_gas_limit         BIGINT NOT NULL,
    tx_gas_used          BIGINT NOT NULL,
    tx_value             NUMERIC NOT NULL,

    -- log --
    log_index            INTEGER NOT NULL,
    log_address          TEXT NOT NULL,
    log_ordinal          INTEGER NOT NULL,
    log_topics           TEXT NOT NULL,
    log_data             TEXT NOT NULL,

    -- external shares burnt --
    owner                TEXT NOT NULL DEFAULT '',
    amount_of_shares     NUMERIC NOT NULL,

    PRIMARY KEY (block_num, tx_index, log_index)
);

CREATE INDEX IF NOT EXISTS idx_steth_external_shares_burnt_timestamp ON steth_external_shares_burnt (timestamp);
CREATE INDEX IF NOT EXISTS idx_steth_external_shares_burnt_owner ON steth_external_shares_burnt (owner);

-- Block Rewards table for PostgreSQL
CREATE TABLE IF NOT EXISTS block_rewards (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- block reward --
    index                INTEGER NOT NULL,
    miner                TEXT NOT NULL,
    value                NUMERIC NOT NULL,
    reason               TEXT NOT NULL,

    PRIMARY KEY (block_num, index)
);

CREATE INDEX IF NOT EXISTS idx_block_rewards_timestamp ON block_rewards (timestamp);
CREATE INDEX IF NOT EXISTS idx_block_rewards_miner ON block_rewards (miner);

-- Withdrawals table for PostgreSQL
CREATE TABLE IF NOT EXISTS withdrawals (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- withdrawal --
    index                INTEGER NOT NULL,
    address              TEXT NOT NULL,
    value                NUMERIC NOT NULL,

    PRIMARY KEY (block_num, index)
);

CREATE INDEX IF NOT EXISTS idx_withdrawals_timestamp ON withdrawals (timestamp);
CREATE INDEX IF NOT EXISTS idx_withdrawals_address ON withdrawals (address);

-- Selfdestructs table for PostgreSQL
CREATE TABLE IF NOT EXISTS selfdestructs (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- selfdestruct --
    index                INTEGER NOT NULL,
    tx_hash              TEXT NOT NULL,
    from_address         TEXT NOT NULL,
    to_address           TEXT NOT NULL,
    value                NUMERIC NOT NULL,

    PRIMARY KEY (block_num, index)
);

CREATE INDEX IF NOT EXISTS idx_selfdestructs_timestamp ON selfdestructs (timestamp);
CREATE INDEX IF NOT EXISTS idx_selfdestructs_from_address ON selfdestructs (from_address);
CREATE INDEX IF NOT EXISTS idx_selfdestructs_to_address ON selfdestructs (to_address);

-- Genesis Balances table for PostgreSQL
CREATE TABLE IF NOT EXISTS genesis_balances (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- genesis balance --
    index                INTEGER NOT NULL,
    address              TEXT NOT NULL,
    value                NUMERIC NOT NULL,

    PRIMARY KEY (block_num, index)
);

CREATE INDEX IF NOT EXISTS idx_genesis_balances_address ON genesis_balances (address);

-- DAO Transfers table for PostgreSQL
CREATE TABLE IF NOT EXISTS dao_transfers (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    minute               INTEGER NOT NULL,

    -- dao transfer --
    index                INTEGER NOT NULL,
    address              TEXT NOT NULL,
    old_value            NUMERIC NOT NULL,
    new_value            NUMERIC NOT NULL,
    reason               TEXT NOT NULL,

    PRIMARY KEY (block_num, index)
);

CREATE INDEX IF NOT EXISTS idx_dao_transfers_address ON dao_transfers (address);
