# EVM PostgreSQL Transfers

ERC-20, Native transfers & WETH Events for EVM blockchains stored in PostgreSQL.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [substreams-sink-sql](https://github.com/streamingfast/substreams-sink-sql)
- [Substreams CLI](https://substreams.streamingfast.io/getting-started/installing-the-cli)

## Quick Start

### 1. Start PostgreSQL

```bash
docker compose up -d
```

This starts a PostgreSQL 16 container with:
- **User**: `dev-node`
- **Password**: `insecure-change-me-in-prod`
- **Database**: `dev-node`
- **Port**: `5432`

### 2. Setup Schema

```bash
make setup
```

This creates all tables, indexes, and upsert rules in the database.

### 3. Run the Sink

```bash
make dev
```

This streams ERC-20 and native transfer data from Ethereum mainnet to your PostgreSQL database.

## Querying PostgreSQL

### Using Docker exec

```bash
# Connect to psql shell
docker exec -it substreams-postgres-transfers psql -U dev-node -d dev-node

# Run a single query
docker exec substreams-postgres-transfers psql -U dev-node -d dev-node -c "SELECT * FROM erc20_transfers LIMIT 10;"
```

### Using psql directly (if installed locally)

```bash
psql "postgresql://dev-node:insecure-change-me-in-prod@localhost:5432/dev-node"
```

### Example Queries

```sql
-- Count all transfers
SELECT COUNT(*) FROM erc20_transfers;
SELECT COUNT(*) FROM transactions;

-- Get recent ERC-20 transfers for a specific token (e.g., USDT)
SELECT "from", "to", amount, timestamp 
FROM erc20_transfers 
WHERE log_address = '0xdac17f958d2ee523a2206206994597c13d831ec7'
ORDER BY timestamp DESC 
LIMIT 10;

-- Get all ERC-20 transfers from an address
SELECT log_address, "to", amount, timestamp 
FROM erc20_transfers 
WHERE "from" = '0x...your_address...';

-- Get native transactions for an address
SELECT tx_hash, tx_value, timestamp 
FROM transactions 
WHERE tx_from = '0x...your_address...' OR tx_to = '0x...your_address...';

-- List all tables
\dt

-- Describe a table
\d erc20_transfers
```

## Docker Compose Commands

```bash
# Start PostgreSQL
docker compose up -d

# View logs
docker compose logs -f postgres

# Stop PostgreSQL (keeps data)
docker compose down

# Stop and remove all data (reset)
docker compose down -v

# Check container status
docker compose ps
```

## Configuration

### Environment Variables

You can customize the PostgreSQL connection by setting environment variables in the Makefile or exporting them:

| Variable | Default | Description |
|----------|---------|-------------|
| `ENDPOINT` | `eth.substreams.pinax.network:443` | Substreams endpoint |
| `START_BLOCK` | `24226215` | Starting block number |
| `STOP_BLOCK` | `+100` | Ending block number |
| `PG_DSN` | `psql://dev-node:...@localhost:5432/dev-node?sslmode=disable` | PostgreSQL connection string |

### Customizing Block Range

```bash
# Sync a specific block range
make dev START_BLOCK=20000000 STOP_BLOCK=20001000

# Use a different network endpoint
make dev ENDPOINT=base.substreams.pinax.network:443
```

## Schema

### Tables

| Table | Description |
|-------|-------------|
| `blocks` | Block metadata |
| `transactions` | Native transactions with value transfers |
| `calls` | Internal calls with native value transfers |
| `erc20_transfers` | ERC-20 token Transfer events |
| `erc20_approvals` | ERC-20 token Approval events |
| `weth_deposit` | WETH Deposit events |
| `weth_withdrawal` | WETH Withdrawal events |
| `usdc_mint` | USDC Mint events |
| `usdc_burn` | USDC Burn events |
| `usdt_issue` | USDT Issue events |
| `usdt_redeem` | USDT Redeem events |
| `steth_token_rebased` | stETH TokenRebased events |
| `steth_shares_burnt` | stETH SharesBurnt events |
| `steth_transfer_shares` | stETH TransferShares events |
| `steth_external_shares_burnt` | stETH ExternalSharesBurnt events |
| `block_rewards` | Block rewards (mining/staking) |
| `withdrawals` | Validator withdrawals (post-Shanghai) |
| `selfdestructs` | Selfdestruct events |
| `genesis_balances` | Genesis balances (block 0) |
| `dao_transfers` | DAO hard fork transfers |

## How Inserts Work

The substreams uses `create_row` to insert transfer events. Each event is uniquely identified by the combination of block_num, tx_index, and log_index (or similar keys depending on the event type).

## Troubleshooting

### Connection refused

Make sure PostgreSQL is running:
```bash
docker compose ps
docker compose logs postgres
```

### Permission denied

Check that the credentials match:
```bash
docker exec substreams-postgres-transfers psql -U dev-node -d dev-node -c "SELECT 1;"
```

### Reset everything

```bash
docker compose down -v
docker compose up -d
make setup
```

## License

[MIT](../LICENSE)
