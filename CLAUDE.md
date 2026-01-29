# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust Substreams project for indexing EVM blockchain data from multiple chains (Ethereum, Base, BSC, Polygon, Arbitrum, Optimism, Avalanche, Tron). It extracts token transfers, balances, and DEX trading activity into structured Protocol Buffer messages for data warehouse integration.

## Build Commands

All modules use the same Makefile pattern. Run from within each module directory:

```bash
# Build WASM module
make build                    # cargo build --target wasm32-unknown-unknown --release

# Package for deployment
make pack                     # substreams pack

# Generate protobuf code (in proto/ directory)
make protogen                 # substreams protogen

# Test with interactive GUI
make gui                      # substreams gui with development settings

# Test with no-op sink (throughput testing)
make noop                     # substreams-sink-noop

# Production mode
make prod                     # substreams gui with production-mode flag
```

For ClickHouse sink modules, additional commands:
```bash
make dev                      # Run with local ClickHouse
make setup                    # Initialize ClickHouse schema
```

## Architecture

**Rust workspace** with 18+ member crates compiled to WebAssembly (`wasm32-unknown-unknown`).

### Module Categories

- **Transfer modules** (`erc20-transfers/`, `native-transfers/`): Extract token transfer events
- **Balance modules** (`erc20-balances/`, `native-balances/`): Track token balances
- **DEX modules** (`dex/uniswap-v1/` through `dex/uniswap-v4/`, `dex/balancer/`, `dex/curvefi/`, `dex/bancor/`, `dex/cow/`, `dex/sunpump/`): Extract swap events from DEX protocols
- **ClickHouse sinks** (`clickhouse-evm-transfers/`, `clickhouse-evm-balances/`, `clickhouse-evm-dex/`, `clickhouse-blocks/`): Convert events to ClickHouse database format

### Key Patterns

**Handler functions** use the `#[substreams::handlers::map]` attribute:
```rust
#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, Error>
```

**Event matching** uses ABI decoders from `substreams-abis`:
```rust
if let Some(event) = events::Transfer::match_and_decode(log) {
    // process transfer
}
```

**Address encoding**: Supports both EVM hex addresses and Tron Base58Check encoding via `common::handle_encoding_param()` and `common::bytes_to_string()`.

### Shared Code

- `common/`: Address encoding (Tron Base58Check), BigInt conversions, transaction/log creation helpers
- `proto/`: Protocol Buffer definitions in `proto/v1/*.proto`

## Dependencies

Key workspace dependencies (from Cargo.toml):
- `substreams = "0.7.0"`
- `substreams-ethereum = "0.11.1"`
- `substreams-abis` (Pinax GitHub)
- `substreams-database-change = "3.0.0"`
- `prost = "0.13"`

Rust toolchain: 1.88 (specified in `rust-toolchain.toml`)

## Environment Variables

Each Makefile supports:
- `ENDPOINT`: Substreams endpoint (default varies by module, e.g., `eth.substreams.pinax.network:443`)
- `START_BLOCK`: Starting block number
- `STOP_BLOCK`: Stop block (use `+N` for relative offset)
- `PARALLEL_JOBS`: Parallel workers (default 500)
