#!/bin/bash

# Script to verify ERC20 balances between ClickHouse and RPC calls
# Usage: ./verify-balances.sh [OPTIONS]

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
CONTRACT="${CONTRACT:-0xdac17f958d2ee523a2206206994597c13d831ec7}"
LIMIT="${LIMIT:-30}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Verify ERC20 balances between ClickHouse and RPC calls"
    echo ""
    echo "Options:"
    echo "  --ch-host HOST        ClickHouse host (default: localhost)"
    echo "  --ch-port PORT        ClickHouse port (default: 9000)"
    echo "  --ch-user USER        ClickHouse user (default: default)"
    echo "  --ch-password PASS    ClickHouse password"
    echo "  --ch-database DB      ClickHouse database (default: default)"
    echo "  --rpc-endpoint URL    Ethereum RPC endpoint (default: https://eth.llamarpc.com)"
    echo "  --contract ADDR       ERC20 contract address (default: USDT)"
    echo "  --limit NUM           Number of top holders to check (default: 30)"
    echo "  -h, --help            Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  CH_HOST, CH_PORT, CH_USER, CH_PASSWORD, CH_DATABASE"
    echo "  RPC_ENDPOINT, CONTRACT, LIMIT"
    echo ""
    echo "Example:"
    echo "  $0 --ch-host ch-node890h.riv.eosn.io --ch-password 'YOUR_PASSWORD' \\"
    echo "     --ch-database 'mainnet:evm-balances@v0.3.0' --contract '0xdac17f958d2ee523a2206206994597c13d831ec7'"
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
        --contract)
            CONTRACT="$2"
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

# Get latest block timestamp from ClickHouse
LATEST_BLOCK_QUERY="SELECT max(block_num), max(timestamp) FROM blocks FORMAT TabSeparated"
LATEST_BLOCK_DATA=$(eval "$CH_CMD --query \"$LATEST_BLOCK_QUERY\"" 2>/dev/null)
LATEST_BLOCK_NUM=$(echo "$LATEST_BLOCK_DATA" | cut -f1)
LATEST_BLOCK_TS=$(echo "$LATEST_BLOCK_DATA" | cut -f2)

# Fetch token metadata from RPC (decimals, symbol, name)
rpc_call() {
    local data="$1"
    curl -s -X POST "$RPC_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_call\",\"params\":[{\"to\":\"$CONTRACT\",\"data\":\"$data\"},\"latest\"],\"id\":1}" | jq -r '.result // empty'
}

# decimals() = 0x313ce567
DECIMALS_HEX=$(rpc_call "0x313ce567")
if [[ -n "$DECIMALS_HEX" && "$DECIMALS_HEX" != "null" ]]; then
    DECIMALS=$(python3 -c "print(int('${DECIMALS_HEX}', 16))")
else
    DECIMALS=18
fi

# symbol() = 0x95d89b41
SYMBOL_HEX=$(rpc_call "0x95d89b41")
if [[ -n "$SYMBOL_HEX" && "$SYMBOL_HEX" != "null" && ${#SYMBOL_HEX} -gt 2 ]]; then
    TOKEN_SYMBOL=$(python3 -c "
import codecs
h = '${SYMBOL_HEX}'[2:]
if len(h) >= 128:
    offset = int(h[:64], 16) * 2
    length = int(h[offset:offset+64], 16)
    data = h[offset+64:offset+64+length*2]
    print(codecs.decode(data, 'hex').decode('utf-8', errors='replace').strip())
else:
    print(codecs.decode(h, 'hex').decode('utf-8', errors='replace').strip('\\x00'))
")
else
    TOKEN_SYMBOL="unknown"
fi

# name() = 0x06fdde03
NAME_HEX=$(rpc_call "0x06fdde03")
if [[ -n "$NAME_HEX" && "$NAME_HEX" != "null" && ${#NAME_HEX} -gt 2 ]]; then
    TOKEN_NAME=$(python3 -c "
import codecs
h = '${NAME_HEX}'[2:]
if len(h) >= 128:
    offset = int(h[:64], 16) * 2
    length = int(h[offset:offset+64], 16)
    data = h[offset+64:offset+64+length*2]
    print(codecs.decode(data, 'hex').decode('utf-8', errors='replace').strip())
else:
    print(codecs.decode(h, 'hex').decode('utf-8', errors='replace').strip('\\x00'))
")
else
    TOKEN_NAME="unknown"
fi

echo "=============================================="
echo "ERC20 Balance Verification"
echo "=============================================="
echo "Timestamp: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo ""
echo "ClickHouse Server"
echo "  Host: $CH_HOST"
echo "  Database: $CH_DATABASE"
echo "  Latest Block: #${LATEST_BLOCK_NUM} (${LATEST_BLOCK_TS})"
echo ""
echo "Token Metadata"
echo "  Contract: $CONTRACT"
echo "  Name: $TOKEN_NAME ($TOKEN_SYMBOL)"
echo "  Decimals: $DECIMALS"
echo ""
echo "RPC Endpoint: $RPC_ENDPOINT"
echo "Limit: $LIMIT"
echo "=============================================="
echo ""

# Query ClickHouse for top holders
QUERY="SELECT address, balance, formatReadableQuantity(floor(balance / pow(10, $DECIMALS))) AS balance_formatted, timestamp FROM erc20_balances FINAL WHERE contract = lower('$CONTRACT') ORDER BY balance DESC LIMIT $LIMIT FORMAT TabSeparated"

echo "Querying ClickHouse for top $LIMIT holders..."
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

# ERC20 balanceOf function signature: balanceOf(address) = 0x70a08231
BALANCE_OF_SIG="0x70a08231"

# Function to make RPC call for balance
get_rpc_balance() {
    local address="$1"
    # Pad address to 32 bytes (remove 0x prefix, left-pad with zeros)
    local padded_address=$(echo "$address" | sed 's/0x//' | awk '{printf "%064s\n", $0}' | tr ' ' '0')
    local data="${BALANCE_OF_SIG}${padded_address}"
    
    local response=$(curl -s -X POST "$RPC_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_call\",\"params\":[{\"to\":\"$CONTRACT\",\"data\":\"$data\"},\"latest\"],\"id\":1}")
    
    local result=$(echo "$response" | jq -r '.result // empty')
    
    if [[ -z "$result" || "$result" == "null" ]]; then
        echo "0"
        return
    fi
    
    # Convert hex to decimal (remove 0x prefix)
    # Use bc for large number handling
    local hex_value=$(echo "$result" | sed 's/0x//')
    if [[ -z "$hex_value" || "$hex_value" == "0x" ]]; then
        echo "0"
        return
    fi
    
    # Convert hex to decimal using Python (handles large numbers)
    local decimal_value=$(python3 -c "print(int('$hex_value', 16))" 2>/dev/null || echo "0")
    echo "$decimal_value"
}

# Process RPC balance: format and calculate error in one Python call
process_rpc_balance() {
    local rpc_balance="$1"
    local ch_balance="$2"
    local decimals="$3"
    python3 -c "
import math
rpc = $rpc_balance
ch = $ch_balance
decimals = $decimals
# Format balance
val = math.floor(rpc / (10 ** decimals))
if val >= 1e12:
    formatted = f'{val/1e12:.2f} trillion'
elif val >= 1e9:
    formatted = f'{val/1e9:.2f} billion'
elif val >= 1e6:
    formatted = f'{val/1e6:.2f} million'
elif val >= 1e3:
    formatted = f'{val/1e3:.2f} thousand'
else:
    formatted = f'{val:.2f}'
# Calculate error
if rpc == 0:
    error = 'inf' if ch != 0 else '0.00'
else:
    error = f'{abs(ch - rpc) / rpc * 100:.2f}'
print(f'{formatted}\t{error}')
"
}

# Process results
echo "Comparing balances..."
echo ""
printf "%-4s | %-44s | %-14s | %-14s | %-8s | %-19s | %s\n" "Rank" "Address" "ClickHouse" "RPC" "Error %" "Timestamp" "Status"
printf "%-4s-+-%-44s-+-%-14s-+-%-14s-+-%-8s-+-%-19s-+-%s\n" "$(printf '%0.s-' {1..4})" "$(printf '%0.s-' {1..44})" "$(printf '%0.s-' {1..14})" "$(printf '%0.s-' {1..14})" "$(printf '%0.s-' {1..8})" "$(printf '%0.s-' {1..19})" "$(printf '%0.s-' {1..10})"

MATCH_COUNT=0
MISMATCH_COUNT=0
TOTAL_COUNT=0
INF_COUNT=0
ERRORS=""

while IFS=$'\t' read -r address ch_balance ch_formatted ch_timestamp; do
    [[ -z "$address" ]] && continue
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    
    # Get RPC balance
    rpc_balance=$(get_rpc_balance "$address")
    
    # Process RPC balance (format + error) in single Python call
    rpc_data=$(process_rpc_balance "$rpc_balance" "$ch_balance" "$DECIMALS")
    rpc_formatted=$(echo "$rpc_data" | cut -f1)
    pct_error=$(echo "$rpc_data" | cut -f2)
    
    # Compare balances (treat <0.01% error as a match)
    if [[ "$ch_balance" == "$rpc_balance" ]]; then
        status="${GREEN}✓ MATCH${NC}"
        MATCH_COUNT=$((MATCH_COUNT + 1))
    elif [[ "$pct_error" != "inf" ]] && python3 -c "exit(0 if float('$pct_error') < 0.01 else 1)"; then
        status="${GREEN}≈ MATCH${NC}"
        MATCH_COUNT=$((MATCH_COUNT + 1))
    else
        status="${RED}✗ MISMATCH${NC}"
        MISMATCH_COUNT=$((MISMATCH_COUNT + 1))
        if [[ "$pct_error" == "inf" ]]; then
            INF_COUNT=$((INF_COUNT + 1))
        else
            [[ -z "$ERRORS" ]] && ERRORS="$pct_error" || ERRORS="$ERRORS,$pct_error"
        fi
    fi
    
    # Format error display
    [[ "$pct_error" == "inf" ]] && pct_display="∞" || pct_display="${pct_error}%"
    
    printf "%-4s | %-44s | %14s | %14s | %8s | %-19s | " "$TOTAL_COUNT" "$address" "$ch_formatted" "$rpc_formatted" "$pct_display" "$ch_timestamp"
    echo -e "$status"
done <<< "$RESULTS"

echo ""
echo "=============================================="
echo "Summary"
echo "=============================================="
echo "Total checked: $TOTAL_COUNT"
echo -e "Matches: ${GREEN}$MATCH_COUNT${NC}"
echo -e "Mismatches: ${RED}$MISMATCH_COUNT${NC}"

if [[ $MISMATCH_COUNT -gt 0 ]]; then
    if [[ $INF_COUNT -gt 0 ]]; then
        echo -e "Infinity errors (div by 0): ${YELLOW}$INF_COUNT${NC}"
    fi
    # Calculate median error (only for non-infinity errors)
    if [[ -n "$ERRORS" ]]; then
        MEDIAN_ERROR=$(python3 -c "
import statistics
errors = [$ERRORS]
print(f'{statistics.median(errors):.2f}')
")
        echo -e "Median error (mismatches): ${YELLOW}${MEDIAN_ERROR}%${NC}"
    fi
fi

if [[ $MISMATCH_COUNT -gt 0 ]]; then
    echo ""
    echo -e "${YELLOW}Note: Mismatches may occur due to:${NC}"
    echo "  - ClickHouse data not being fully synced to the latest block"
    echo "  - Recent transactions that haven't been indexed yet"
    echo "  - Different block heights between ClickHouse snapshot and RPC 'latest'"
    exit 1
fi

echo ""
echo -e "${GREEN}All balances verified successfully!${NC}"
exit 0
