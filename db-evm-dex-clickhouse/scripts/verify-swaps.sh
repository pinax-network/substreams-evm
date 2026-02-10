#!/bin/bash

# Script to verify DEX swaps between ClickHouse and RPC transaction receipts
# Usage: ./verify-swaps.sh [OPTIONS]

set -e

# Load .env file if it exists (from script directory)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "$SCRIPT_DIR/.env" ]]; then
    set -a
    source "$SCRIPT_DIR/.env"
    set +a
fi

# Default values
CH_HOST="${CH_HOST:-localhost}"
CH_PORT="${CH_PORT:-9000}"
CH_USER="${CH_USER:-default}"
CH_PASSWORD="${CH_PASSWORD:-}"
CH_DATABASE="${CH_DATABASE:-default}"
RPC_ENDPOINT="${RPC_ENDPOINT:-https://eth.llamarpc.com}"
POOL="${POOL:-}"
LIMIT="${LIMIT:-30}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Verify DEX swaps between ClickHouse and RPC transaction receipts"
    echo ""
    echo "Options:"
    echo "  --ch-host HOST        ClickHouse host (default: localhost)"
    echo "  --ch-port PORT        ClickHouse port (default: 9000)"
    echo "  --ch-user USER        ClickHouse user (default: default)"
    echo "  --ch-password PASS    ClickHouse password"
    echo "  --ch-database DB      ClickHouse database (default: default)"
    echo "  --rpc-endpoint URL    Ethereum RPC endpoint (default: https://eth.llamarpc.com)"
    echo "  --pool ADDR           Pool contract address to filter (optional)"
    echo "  --limit NUM           Number of recent swaps to check (default: 30)"
    echo "  -h, --help            Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  CH_HOST, CH_PORT, CH_USER, CH_PASSWORD, CH_DATABASE"
    echo "  RPC_ENDPOINT, POOL, LIMIT"
    echo ""
    echo "Example:"
    echo "  $0 --ch-host ch-node890h.riv.eosn.io --ch-password 'YOUR_PASSWORD' \\"
    echo "     --ch-database 'mainnet:evm-dex@v0.1.0' --pool '0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640'"
    exit 1
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --ch-host)
            CH_HOST="$2"
            shift 2
            ;;
        --ch-port)
            CH_PORT="$2"
            shift 2
            ;;
        --ch-user)
            CH_USER="$2"
            shift 2
            ;;
        --ch-password)
            CH_PASSWORD="$2"
            shift 2
            ;;
        --ch-database)
            CH_DATABASE="$2"
            shift 2
            ;;
        --rpc-endpoint)
            RPC_ENDPOINT="$2"
            shift 2
            ;;
        --pool)
            POOL="$2"
            shift 2
            ;;
        --limit)
            LIMIT="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Check for required tools
command -v clickhouse-client >/dev/null 2>&1 || { echo "Error: clickhouse-client is required but not installed."; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "Error: curl is required but not installed."; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "Error: jq is required but not installed."; exit 1; }

# Build ClickHouse client command
CH_CMD="clickhouse-client --host $CH_HOST --port $CH_PORT --user $CH_USER --database $CH_DATABASE"
if [[ -n "$CH_PASSWORD" ]]; then
    CH_CMD="$CH_CMD --password '$CH_PASSWORD'"
fi

# Get latest block from ClickHouse
LATEST_BLOCK_QUERY="SELECT max(block_num), max(timestamp) FROM blocks FORMAT TabSeparated"
LATEST_BLOCK_DATA=$(eval "$CH_CMD --query \"$LATEST_BLOCK_QUERY\"" 2>/dev/null)
LATEST_BLOCK_NUM=$(echo "$LATEST_BLOCK_DATA" | cut -f1)
LATEST_BLOCK_TS=$(echo "$LATEST_BLOCK_DATA" | cut -f2)

# Build pool filter clause
POOL_FILTER=""
if [[ -n "$POOL" ]]; then
    POOL_FILTER="AND pool = lower('$POOL')"
fi

echo "=============================================="
echo "DEX Swap Verification"
echo "=============================================="
echo "Timestamp: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo ""
echo "ClickHouse Server"
echo "  Host: $CH_HOST"
echo "  Database: $CH_DATABASE"
echo "  Latest Block: #${LATEST_BLOCK_NUM} (${LATEST_BLOCK_TS})"
echo ""
if [[ -n "$POOL" ]]; then
    echo "Pool Filter: $POOL"
fi
echo "RPC Endpoint: $RPC_ENDPOINT"
echo "Limit: $LIMIT"
echo "=============================================="
echo ""

# Query ClickHouse for latest swaps
QUERY="SELECT tx_hash, block_num, log_index, log_address, log_topic0, protocol, pool, input_contract, output_contract, timestamp FROM swaps WHERE block_num > 0 $POOL_FILTER ORDER BY block_num DESC, log_index DESC LIMIT $LIMIT FORMAT TabSeparated"

echo "Querying ClickHouse for latest $LIMIT swaps..."
echo ""

# Execute query and store results
RESULTS=$(eval "$CH_CMD --query \"$QUERY\"" 2>&1)

if [[ $? -ne 0 ]]; then
    echo -e "${RED}Error querying ClickHouse:${NC}"
    echo "$RESULTS"
    exit 1
fi

if [[ -z "$RESULTS" ]]; then
    echo -e "${YELLOW}No results found in ClickHouse${NC}"
    exit 0
fi

# Function to get transaction receipt via RPC
get_tx_receipt() {
    local tx_hash="$1"
    curl -s -X POST "$RPC_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionReceipt\",\"params\":[\"$tx_hash\"],\"id\":1}"
}

# Function to verify a swap log exists in receipt
verify_swap_in_receipt() {
    local receipt="$1"
    local expected_log_index="$2"
    local expected_log_address="$3"
    local expected_topic0="$4"
    local expected_block_num="$5"

    python3 -c "
import json, sys
receipt = json.loads('''$receipt''')
result = receipt.get('result')
if not result:
    print('NO_RECEIPT')
    sys.exit(0)

# Verify block number
rpc_block = int(result.get('blockNumber', '0x0'), 16)
expected_block = $expected_block_num
if rpc_block != expected_block:
    print(f'BLOCK_MISMATCH:{rpc_block}')
    sys.exit(0)

# Search for matching log
expected_idx = $expected_log_index
expected_addr = '${expected_log_address}'.lower()
expected_t0 = '${expected_topic0}'.lower()

for log in result.get('logs', []):
    log_idx = int(log.get('logIndex', '0x0'), 16)
    log_addr = log.get('address', '').lower()
    topics = log.get('topics', [])
    topic0 = topics[0].lower() if topics else ''

    if log_idx == expected_idx and log_addr == expected_addr and topic0 == expected_t0:
        print('MATCH')
        sys.exit(0)

print('LOG_NOT_FOUND')
"
}

# Process results
echo "Verifying swaps against RPC transaction receipts..."
echo ""
printf "%-4s | %-68s | %-10s | %-5s | %-12s | %-19s | %s\n" "Rank" "Tx Hash" "Block" "Log#" "Protocol" "Timestamp" "Status"
printf "%-4s-+-%-68s-+-%-10s-+-%-5s-+-%-12s-+-%-19s-+-%s\n" "$(printf '%0.s-' {1..4})" "$(printf '%0.s-' {1..68})" "$(printf '%0.s-' {1..10})" "$(printf '%0.s-' {1..5})" "$(printf '%0.s-' {1..12})" "$(printf '%0.s-' {1..19})" "$(printf '%0.s-' {1..15})"

MATCH_COUNT=0
MISMATCH_COUNT=0
NO_RECEIPT_COUNT=0
TOTAL_COUNT=0

while IFS=$'\t' read -r tx_hash block_num log_index log_address log_topic0 protocol pool input_contract output_contract ch_timestamp; do
    [[ -z "$tx_hash" ]] && continue
    TOTAL_COUNT=$((TOTAL_COUNT + 1))

    # Ensure tx_hash has 0x prefix for RPC call
    rpc_tx_hash="$tx_hash"
    if [[ "$rpc_tx_hash" != 0x* ]]; then
        rpc_tx_hash="0x${rpc_tx_hash}"
    fi

    # Get transaction receipt from RPC
    receipt=$(get_tx_receipt "$rpc_tx_hash")

    # Verify the swap log in receipt
    verify_result=$(verify_swap_in_receipt "$receipt" "$log_index" "$log_address" "$log_topic0" "$block_num")

    case "$verify_result" in
        MATCH)
            status="${GREEN}✓ MATCH${NC}"
            MATCH_COUNT=$((MATCH_COUNT + 1))
            ;;
        NO_RECEIPT)
            status="${YELLOW}? NO RECEIPT${NC}"
            NO_RECEIPT_COUNT=$((NO_RECEIPT_COUNT + 1))
            ;;
        BLOCK_MISMATCH*)
            rpc_block=$(echo "$verify_result" | cut -d: -f2)
            status="${RED}✗ BLOCK MISMATCH (RPC: $rpc_block)${NC}"
            MISMATCH_COUNT=$((MISMATCH_COUNT + 1))
            ;;
        LOG_NOT_FOUND)
            status="${RED}✗ LOG NOT FOUND${NC}"
            MISMATCH_COUNT=$((MISMATCH_COUNT + 1))
            ;;
        *)
            status="${RED}✗ ERROR${NC}"
            MISMATCH_COUNT=$((MISMATCH_COUNT + 1))
            ;;
    esac

    printf "%-4s | %-68s | %10s | %5s | %-12s | %-19s | " "$TOTAL_COUNT" "$tx_hash" "$block_num" "$log_index" "$protocol" "$ch_timestamp"
    echo -e "$status"
done <<< "$RESULTS"

echo ""
echo "=============================================="
echo "Summary"
echo "=============================================="
echo "Total checked: $TOTAL_COUNT"
echo -e "Matches: ${GREEN}$MATCH_COUNT${NC}"
echo -e "Mismatches: ${RED}$MISMATCH_COUNT${NC}"
if [[ $NO_RECEIPT_COUNT -gt 0 ]]; then
    echo -e "No receipt: ${YELLOW}$NO_RECEIPT_COUNT${NC}"
fi

if [[ $MISMATCH_COUNT -gt 0 || $NO_RECEIPT_COUNT -gt 0 ]]; then
    echo ""
    echo -e "${YELLOW}Note: Mismatches may occur due to:${NC}"
    echo "  - ClickHouse data not being fully synced to the latest block"
    echo "  - Recent transactions that haven't been indexed yet"
    echo "  - RPC node not having the transaction (pruned or different network)"
    echo "  - Different block heights between ClickHouse snapshot and RPC"
    exit 1
fi

echo ""
echo -e "${GREEN}All swaps verified successfully!${NC}"
exit 0
