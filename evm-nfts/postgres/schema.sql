-- ERC721 Transfers --
CREATE TABLE IF NOT EXISTS erc721_transfers (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    operator             TEXT NOT NULL DEFAULT '',
    "from"               TEXT NOT NULL,
    "to"                 TEXT NOT NULL,
    token_id             NUMERIC NOT NULL,
    amount               NUMERIC NOT NULL DEFAULT 1,

    -- classification --
    transfer_type        TEXT NOT NULL,
    token_standard       TEXT NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_erc721_transfers_timestamp ON erc721_transfers (timestamp);
CREATE INDEX IF NOT EXISTS idx_erc721_transfers_tx_hash ON erc721_transfers (tx_hash);
CREATE INDEX IF NOT EXISTS idx_erc721_transfers_caller ON erc721_transfers (caller);
CREATE INDEX IF NOT EXISTS idx_erc721_transfers_contract ON erc721_transfers (contract);
CREATE INDEX IF NOT EXISTS idx_erc721_transfers_from ON erc721_transfers ("from");
CREATE INDEX IF NOT EXISTS idx_erc721_transfers_to ON erc721_transfers ("to");
CREATE INDEX IF NOT EXISTS idx_erc721_transfers_token_id ON erc721_transfers (token_id);

-- ERC721 Approval --
CREATE TABLE IF NOT EXISTS erc721_approvals (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    owner                TEXT NOT NULL,
    approved             TEXT NOT NULL,
    token_id             NUMERIC NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_erc721_approvals_timestamp ON erc721_approvals (timestamp);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_tx_hash ON erc721_approvals (tx_hash);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_caller ON erc721_approvals (caller);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_contract ON erc721_approvals (contract);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_owner ON erc721_approvals (owner);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_approved ON erc721_approvals (approved);

-- ERC721 Approval For All --
CREATE TABLE IF NOT EXISTS erc721_approvals_for_all (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    owner                TEXT NOT NULL,
    operator             TEXT NOT NULL,
    approved             BOOLEAN NOT NULL,

    -- classification --
    token_standard       TEXT NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_erc721_approvals_for_all_timestamp ON erc721_approvals_for_all (timestamp);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_for_all_tx_hash ON erc721_approvals_for_all (tx_hash);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_for_all_contract ON erc721_approvals_for_all (contract);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_for_all_owner ON erc721_approvals_for_all (owner);
CREATE INDEX IF NOT EXISTS idx_erc721_approvals_for_all_operator ON erc721_approvals_for_all (operator);

-- ERC1155 Transfers --
CREATE TABLE IF NOT EXISTS erc1155_transfers (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    operator             TEXT NOT NULL DEFAULT '',
    "from"               TEXT NOT NULL,
    "to"                 TEXT NOT NULL,
    token_id             NUMERIC NOT NULL,
    amount               NUMERIC NOT NULL DEFAULT 1,

    -- classification --
    transfer_type        TEXT NOT NULL,
    token_standard       TEXT NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_erc1155_transfers_timestamp ON erc1155_transfers (timestamp);
CREATE INDEX IF NOT EXISTS idx_erc1155_transfers_tx_hash ON erc1155_transfers (tx_hash);
CREATE INDEX IF NOT EXISTS idx_erc1155_transfers_contract ON erc1155_transfers (contract);
CREATE INDEX IF NOT EXISTS idx_erc1155_transfers_from ON erc1155_transfers ("from");
CREATE INDEX IF NOT EXISTS idx_erc1155_transfers_to ON erc1155_transfers ("to");

-- ERC1155 Approval For All --
CREATE TABLE IF NOT EXISTS erc1155_approvals_for_all (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    owner                TEXT NOT NULL,
    operator             TEXT NOT NULL,
    approved             BOOLEAN NOT NULL,

    -- classification --
    token_standard       TEXT NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_erc1155_approvals_for_all_timestamp ON erc1155_approvals_for_all (timestamp);
CREATE INDEX IF NOT EXISTS idx_erc1155_approvals_for_all_contract ON erc1155_approvals_for_all (contract);
CREATE INDEX IF NOT EXISTS idx_erc1155_approvals_for_all_owner ON erc1155_approvals_for_all (owner);

-- ERC721 Token Metadata --
CREATE TABLE IF NOT EXISTS erc721_metadata_by_contract (
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    contract             TEXT NOT NULL,
    symbol               TEXT NOT NULL DEFAULT '',
    name                 TEXT NOT NULL DEFAULT '',

    PRIMARY KEY (contract)
);

CREATE INDEX IF NOT EXISTS idx_erc721_metadata_by_contract_symbol ON erc721_metadata_by_contract (symbol);
CREATE INDEX IF NOT EXISTS idx_erc721_metadata_by_contract_name ON erc721_metadata_by_contract (name);

CREATE TABLE IF NOT EXISTS erc721_metadata_by_token (
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    contract             TEXT NOT NULL,
    token_id             NUMERIC NOT NULL,
    uri                  TEXT NOT NULL DEFAULT '',

    PRIMARY KEY (contract, token_id)
);

CREATE TABLE IF NOT EXISTS erc721_total_supply (
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    contract             TEXT NOT NULL,
    total_supply         NUMERIC NOT NULL,

    PRIMARY KEY (contract)
);

CREATE TABLE IF NOT EXISTS erc721_base_uri (
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    contract             TEXT NOT NULL,
    base_uri             TEXT NOT NULL,

    PRIMARY KEY (contract)
);

-- ERC1155 Token Metadata --
CREATE TABLE IF NOT EXISTS erc1155_metadata_by_contract (
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    contract             TEXT NOT NULL,
    symbol               TEXT NOT NULL DEFAULT '',
    name                 TEXT NOT NULL DEFAULT '',

    PRIMARY KEY (contract)
);

CREATE TABLE IF NOT EXISTS erc1155_metadata_by_token (
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,
    contract             TEXT NOT NULL,
    token_id             NUMERIC NOT NULL,
    uri                  TEXT NOT NULL DEFAULT '',

    PRIMARY KEY (contract, token_id)
);

-- CryptoPunk Assigns --
CREATE TABLE IF NOT EXISTS punk_assigns (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    "to"                 TEXT NOT NULL,
    punk_index           NUMERIC NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_punk_assigns_timestamp ON punk_assigns (timestamp);
CREATE INDEX IF NOT EXISTS idx_punk_assigns_tx_hash ON punk_assigns (tx_hash);
CREATE INDEX IF NOT EXISTS idx_punk_assigns_to ON punk_assigns ("to");

-- CryptoPunk Transfers --
CREATE TABLE IF NOT EXISTS punk_transfers (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    "from"               TEXT NOT NULL,
    "to"                 TEXT NOT NULL,
    punk_index           NUMERIC NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_punk_transfers_timestamp ON punk_transfers (timestamp);
CREATE INDEX IF NOT EXISTS idx_punk_transfers_tx_hash ON punk_transfers (tx_hash);
CREATE INDEX IF NOT EXISTS idx_punk_transfers_from ON punk_transfers ("from");
CREATE INDEX IF NOT EXISTS idx_punk_transfers_to ON punk_transfers ("to");

-- CryptoPunk Bought --
CREATE TABLE IF NOT EXISTS punk_bought (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    "from"               TEXT NOT NULL,
    "to"                 TEXT NOT NULL,
    punk_index           NUMERIC NOT NULL,
    value                NUMERIC NOT NULL,
    value_is_null        BOOLEAN NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_punk_bought_timestamp ON punk_bought (timestamp);
CREATE INDEX IF NOT EXISTS idx_punk_bought_tx_hash ON punk_bought (tx_hash);
CREATE INDEX IF NOT EXISTS idx_punk_bought_from ON punk_bought ("from");
CREATE INDEX IF NOT EXISTS idx_punk_bought_to ON punk_bought ("to");

-- CryptoPunk BidEntered --
CREATE TABLE IF NOT EXISTS punk_bid_entered (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    "from"               TEXT NOT NULL,
    punk_index           NUMERIC NOT NULL,
    value                NUMERIC NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_punk_bid_entered_timestamp ON punk_bid_entered (timestamp);
CREATE INDEX IF NOT EXISTS idx_punk_bid_entered_tx_hash ON punk_bid_entered (tx_hash);
CREATE INDEX IF NOT EXISTS idx_punk_bid_entered_from ON punk_bid_entered ("from");

-- CryptoPunk BidWithdrawn --
CREATE TABLE IF NOT EXISTS punk_bid_withdrawn (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    "from"               TEXT NOT NULL,
    punk_index           NUMERIC NOT NULL,
    value                NUMERIC NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_punk_bid_withdrawn_timestamp ON punk_bid_withdrawn (timestamp);
CREATE INDEX IF NOT EXISTS idx_punk_bid_withdrawn_tx_hash ON punk_bid_withdrawn (tx_hash);
CREATE INDEX IF NOT EXISTS idx_punk_bid_withdrawn_from ON punk_bid_withdrawn ("from");

-- CryptoPunk NoLongerForSale --
CREATE TABLE IF NOT EXISTS punk_no_longer_for_sale (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    punk_index           NUMERIC NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_punk_no_longer_for_sale_timestamp ON punk_no_longer_for_sale (timestamp);
CREATE INDEX IF NOT EXISTS idx_punk_no_longer_for_sale_tx_hash ON punk_no_longer_for_sale (tx_hash);

-- CryptoPunk PunkOffered --
CREATE TABLE IF NOT EXISTS punk_offered (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    "to"                 TEXT NOT NULL,
    punk_index           NUMERIC NOT NULL,
    min_value            NUMERIC NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_punk_offered_timestamp ON punk_offered (timestamp);
CREATE INDEX IF NOT EXISTS idx_punk_offered_tx_hash ON punk_offered (tx_hash);
CREATE INDEX IF NOT EXISTS idx_punk_offered_to ON punk_offered ("to");

-- Seaport Order Fulfilled --
CREATE TABLE IF NOT EXISTS seaport_order_fulfilled (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    order_hash           TEXT NOT NULL,
    offerer              TEXT NOT NULL,
    zone                 TEXT NOT NULL,
    recipient            TEXT NOT NULL,

    -- event (JSON) --
    offer_raw            TEXT NOT NULL,
    consideration_raw    TEXT NOT NULL,

    PRIMARY KEY (order_hash)
);

CREATE INDEX IF NOT EXISTS idx_seaport_order_fulfilled_timestamp ON seaport_order_fulfilled (timestamp);
CREATE INDEX IF NOT EXISTS idx_seaport_order_fulfilled_block_num ON seaport_order_fulfilled (block_num);
CREATE INDEX IF NOT EXISTS idx_seaport_order_fulfilled_tx_hash ON seaport_order_fulfilled (tx_hash);
CREATE INDEX IF NOT EXISTS idx_seaport_order_fulfilled_caller ON seaport_order_fulfilled (caller);
CREATE INDEX IF NOT EXISTS idx_seaport_order_fulfilled_offerer ON seaport_order_fulfilled (offerer);
CREATE INDEX IF NOT EXISTS idx_seaport_order_fulfilled_zone ON seaport_order_fulfilled (zone);
CREATE INDEX IF NOT EXISTS idx_seaport_order_fulfilled_recipient ON seaport_order_fulfilled (recipient);

-- Seaport Orders Matched --
CREATE TABLE IF NOT EXISTS seaport_orders_matched (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    order_hashes_raw     TEXT NOT NULL,

    PRIMARY KEY (block_num, "index")
);

CREATE INDEX IF NOT EXISTS idx_seaport_orders_matched_timestamp ON seaport_orders_matched (timestamp);
CREATE INDEX IF NOT EXISTS idx_seaport_orders_matched_tx_hash ON seaport_orders_matched (tx_hash);

-- Seaport Order Cancelled --
CREATE TABLE IF NOT EXISTS seaport_order_cancelled (
    -- block --
    block_num            INTEGER NOT NULL,
    block_hash           TEXT NOT NULL,
    timestamp            TIMESTAMP NOT NULL,

    -- ordering --
    ordinal              BIGINT NOT NULL,
    "index"              BIGINT NOT NULL,
    global_sequence      BIGINT NOT NULL,

    -- transaction --
    tx_hash              TEXT NOT NULL,

    -- call --
    caller               TEXT NOT NULL,

    -- log --
    contract             TEXT NOT NULL,

    -- event --
    order_hash           TEXT NOT NULL,
    offerer              TEXT NOT NULL,
    zone                 TEXT NOT NULL,

    PRIMARY KEY (order_hash)
);

CREATE INDEX IF NOT EXISTS idx_seaport_order_cancelled_timestamp ON seaport_order_cancelled (timestamp);
CREATE INDEX IF NOT EXISTS idx_seaport_order_cancelled_block_num ON seaport_order_cancelled (block_num);
CREATE INDEX IF NOT EXISTS idx_seaport_order_cancelled_tx_hash ON seaport_order_cancelled (tx_hash);
CREATE INDEX IF NOT EXISTS idx_seaport_order_cancelled_offerer ON seaport_order_cancelled (offerer);
CREATE INDEX IF NOT EXISTS idx_seaport_order_cancelled_zone ON seaport_order_cancelled (zone);

-- Cursors --
CREATE TABLE IF NOT EXISTS cursors (
    id         TEXT NOT NULL,
    cursor     TEXT NOT NULL,
    block_num  BIGINT NOT NULL,
    block_id   TEXT NOT NULL,

    PRIMARY KEY (id)
);
