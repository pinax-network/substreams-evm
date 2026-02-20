CREATE TABLE IF NOT EXISTS contracts (
    block_num               INTEGER NOT NULL,
    block_hash              TEXT NOT NULL,
    timestamp               TIMESTAMP NOT NULL,

    transaction_hash        TEXT NOT NULL,
    transaction_index       INTEGER NOT NULL,

    ordinal                 BIGINT NOT NULL,
    address                 TEXT NOT NULL,
    "from"                  TEXT NOT NULL,
    "to"                    TEXT NOT NULL,
    deployer                TEXT NOT NULL,
    factory                 TEXT NOT NULL DEFAULT '',
    code                    TEXT NOT NULL DEFAULT '',
    code_hash               TEXT NOT NULL DEFAULT '',
    input                   TEXT NOT NULL DEFAULT '',

    PRIMARY KEY (block_hash, transaction_index, ordinal)
);

CREATE INDEX IF NOT EXISTS idx_contracts_block_num ON contracts (block_num);
CREATE INDEX IF NOT EXISTS idx_contracts_timestamp ON contracts (timestamp);
CREATE INDEX IF NOT EXISTS idx_contracts_address ON contracts (address);
CREATE INDEX IF NOT EXISTS idx_contracts_deployer ON contracts (deployer);
CREATE INDEX IF NOT EXISTS idx_contracts_factory ON contracts (factory);
CREATE INDEX IF NOT EXISTS idx_contracts_code_hash ON contracts (code_hash);
CREATE INDEX IF NOT EXISTS idx_contracts_tx_hash ON contracts (transaction_hash);
