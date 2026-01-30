# EVM PostgreSQL Balances

ERC-20 & Native balances for EVM blockchains stored in PostgreSQL.

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

This streams ERC-20 and native balance data from Ethereum mainnet to your PostgreSQL database.

## Querying PostgreSQL

### Using Docker exec

```bash
# Connect to psql shell
docker exec -it substreams-postgres psql -U dev-node -d dev-node

# Run a single query
docker exec substreams-postgres psql -U dev-node -d dev-node -c "SELECT * FROM erc20_balances LIMIT 10;"
```

### Using psql directly (if installed locally)

```bash
psql "postgresql://dev-node:insecure-change-me-in-prod@localhost:5432/dev-node"
```

### Example Queries

```sql
-- Count all balances
SELECT COUNT(*) FROM erc20_balances;
SELECT COUNT(*) FROM native_balances;

-- Get top 10 ERC-20 balances for a specific token (e.g., USDT)
SELECT address, balance 
FROM erc20_balances 
WHERE contract = '0xdac17f958d2ee523a2206206994597c13d831ec7'
ORDER BY balance DESC 
LIMIT 10;

-- Get all ERC-20 balances for an address
SELECT contract, balance, timestamp 
FROM erc20_balances 
WHERE address = '0x...your_address...';

-- Get native ETH balance for an address
SELECT balance, timestamp 
FROM native_balances 
WHERE address = '0x...your_address...';

-- List all tables
\dt

-- Describe a table
\d erc20_balances
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
| `START_BLOCK` | `23145742` | Starting block number |
| `STOP_BLOCK` | `23145745` | Ending block number |
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
| `blocks` | Block metadata (number, hash, timestamp) |
| `erc20_balances` | Latest ERC-20 token balances per address/contract |
| `native_balances` | Latest native currency (ETH) balances per address |

### ERC-20 Balances Schema

```sql
CREATE TABLE erc20_balances (
    block_num   INTEGER NOT NULL,
    block_hash  TEXT NOT NULL,
    timestamp   TIMESTAMP NOT NULL,
    contract    TEXT NOT NULL,      -- Token contract address
    address     TEXT NOT NULL,      -- Holder address
    balance     NUMERIC NOT NULL,   -- Balance in wei
    PRIMARY KEY (contract, address)
);
```

### Native Balances Schema

```sql
CREATE TABLE native_balances (
    block_num   INTEGER NOT NULL,
    block_hash  TEXT NOT NULL,
    timestamp   TIMESTAMP NOT NULL,
    address     TEXT PRIMARY KEY,   -- Holder address
    balance     NUMERIC NOT NULL    -- Balance in wei
);
```

## How Upserts Work

The schema uses PostgreSQL `RULE`s to handle balance updates. When the same address receives multiple balance changes within a block, only the latest value is stored:

```sql
-- Automatically converts INSERT to UPDATE when key exists
CREATE RULE upsert_erc20_balances AS
    ON INSERT TO erc20_balances
    WHERE EXISTS (SELECT 1 FROM erc20_balances WHERE contract = NEW.contract AND address = NEW.address)
    DO INSTEAD UPDATE ...
```

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
docker exec substreams-postgres psql -U dev-node -d dev-node -c "SELECT 1;"
```

### Reset everything

```bash
docker compose down -v
docker compose up -d
make setup
```

## License

[MIT](../LICENSE)
