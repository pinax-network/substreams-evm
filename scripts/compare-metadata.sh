#!/bin/bash

# Load environment variables from .env file
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/.env" ]; then
    source "$SCRIPT_DIR/.env"
else
    echo "Error: .env file not found in $SCRIPT_DIR"
    exit 1
fi

# ClickHouse connection settings (from .env: LEGACY_HOST, NEW_HOST, CH_USER, CH_PASSWORD)
PORT="9000"
LEGACY_DB="mainnet:evm-tokens@v1.17.4"
NEW_DB="metadata"

# Helper functions
legacy_query() {
    clickhouse client --host "$LEGACY_HOST" --password "$CH_PASSWORD" --user "$CH_USER" --port "$PORT" --database "$LEGACY_DB" --query "$1"
}

new_query() {
    clickhouse client --host "$NEW_HOST" --password "$CH_PASSWORD" --user "$CH_USER" --port "$PORT" --database "$NEW_DB" --query "$1"
}

# Interactive helpers for manual exploration
legacy() {
    clickhouse client --host "$LEGACY_HOST" --password "$CH_PASSWORD" --user "$CH_USER" --port "$PORT" --database "$LEGACY_DB"
}

new() {
    clickhouse client --host "$NEW_HOST" --password "$CH_PASSWORD" --user "$CH_USER" --port "$PORT" --database "$NEW_DB"
}

echo "=== ClickHouse Metadata Comparison ==="
echo ""

# 1. Basic counts
echo "### 1. Basic Counts ###"
echo ""
echo "Legacy (metadata_view):"
legacy_query "SELECT count() as total, max(timestamp) as latest FROM metadata_view FINAL"

echo ""
echo "New (metadata where network='mainnet'):"
new_query "SELECT count() as total, max(timestamp) as latest FROM metadata FINAL WHERE network = 'mainnet'"

echo ""
echo "### 2. Schema Comparison ###"
echo ""
echo "Legacy schema:"
legacy_query "DESCRIBE metadata_view"

echo ""
echo "New schema:"
new_query "DESCRIBE metadata"

echo ""
echo "### 3. Contract Address Comparison ###"
echo ""

# Contracts only in Legacy
echo "Contracts ONLY in Legacy (not in New):"
legacy_query "
SELECT count() as only_in_legacy
FROM metadata_view FINAL
WHERE address NOT IN (
    SELECT address FROM remote('$NEW_HOST', '$NEW_DB', 'metadata', '$USER', '$PASSWORD') FINAL WHERE network = 'mainnet'
)
"

# Contracts only in New
echo ""
echo "Contracts ONLY in New (not in Legacy):"
new_query "
SELECT count() as only_in_new
FROM metadata FINAL
WHERE network = 'mainnet'
AND address NOT IN (
    SELECT address FROM remote('$LEGACY_HOST', '$LEGACY_DB', 'metadata_view', '$USER', '$PASSWORD') FINAL
)
"

echo ""
echo "### 4. Sample of Contracts Only in Legacy ###"
legacy_query "
SELECT address, name, symbol, decimals
FROM metadata_view FINAL
WHERE address NOT IN (
    SELECT address FROM remote('$NEW_HOST', '$NEW_DB', 'metadata', '$USER', '$PASSWORD') FINAL WHERE network = 'mainnet'
)
LIMIT 10
"

echo ""
echo "### 5. Sample of Contracts Only in New ###"
new_query "
SELECT address, name, symbol, decimals
FROM metadata FINAL
WHERE network = 'mainnet'
AND address NOT IN (
    SELECT address FROM remote('$LEGACY_HOST', '$LEGACY_DB', 'metadata_view', '$USER', '$PASSWORD') FINAL
)
LIMIT 10
"

echo ""
echo "### 6. Data Quality: Null/Empty Fields Comparison ###"
echo ""
echo "Legacy - fields with NULL or empty values:"
legacy_query "
SELECT 
    countIf(name = '' OR name IS NULL) as empty_name,
    countIf(symbol = '' OR symbol IS NULL) as empty_symbol,
    countIf(decimals = 0 OR decimals IS NULL) as zero_decimals,
    count() as total
FROM metadata_view FINAL
"

echo ""
echo "New - fields with NULL or empty values:"
new_query "
SELECT 
    countIf(name = '' OR name IS NULL) as empty_name,
    countIf(symbol = '' OR symbol IS NULL) as empty_symbol,
    countIf(decimals = 0 OR decimals IS NULL) as zero_decimals,
    count() as total
FROM metadata FINAL
WHERE network = 'mainnet'
"

echo ""
echo "### 7. Matching Contracts - Field Differences ###"
echo "Checking contracts that exist in both but have different metadata:"
new_query "
SELECT count() as different_metadata
FROM metadata m FINAL
WHERE m.network = 'mainnet'
AND m.address IN (
    SELECT address FROM remote('$LEGACY_HOST', '$LEGACY_DB', 'metadata_view', '$USER', '$PASSWORD') FINAL
)
AND (m.name, m.symbol, m.decimals) NOT IN (
    SELECT name, symbol, decimals 
    FROM remote('$LEGACY_HOST', '$LEGACY_DB', 'metadata_view', '$USER', '$PASSWORD') FINAL 
    WHERE address = m.address
)
"

echo ""
echo "=== Comparison Complete ==="
