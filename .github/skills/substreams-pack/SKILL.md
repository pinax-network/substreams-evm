---
name: substreams-pack
description: Instructions for building, packing, and distributing Substreams packages (.spkg) in the substreams-evm monorepo. Use when building crates, running `substreams pack`, or copying .spkg files.
license: Apache-2.0
compatibility:
  platforms: [claude-code, cursor, vscode, windsurf]
metadata:
  version: 1.0.0
  author: Pinax Network
  documentation: https://github.com/pinax-network/substreams-evm
---

# Substreams Pack & Release

Instructions for building, packing, and copying `.spkg` files in this monorepo.

## Overview

The `./spkg/` directory is the central distribution point for all Substreams packages. Downstream modules import their dependencies from `../spkg/*.spkg`, so packages **must** be copied there after packing.

## Dependency Chain

Packages must be built **bottom-up** — a package's `.spkg` dependencies must exist in `./spkg/` before it can be packed.

```
Level 0 — Leaf crates (no in-repo .spkg dependencies):
  ├── erc20-transfers/       → erc20-transfers-v*.spkg
  ├── erc20-tokens/          → erc20-tokens-v*.spkg
  ├── erc20-balances/        → erc20-balances-v*.spkg
  ├── native-transfers/      → evm-native-transfers-v*.spkg
  └── native-balances/       → evm-native-balances-v*.spkg

Level 1 — DB layer (imports Level 0 .spkg files):
  ├── db-evm-transfers/      → evm-transfers-v*.spkg
  └── db-evm-balances/       → evm-balances-v*.spkg

Level 2 — Sink layer (imports Level 1 .spkg files):
  ├── db-evm-transfers-clickhouse/  → evm-clickhouse-transfers-v*.spkg
  ├── db-evm-transfers-postgres/    → evm-postgres-transfers-v*.spkg
  ├── db-evm-balances-clickhouse/   → evm-clickhouse-balances-v*.spkg
  └── db-evm-balances-postgres/     → evm-postgres-balances-v*.spkg
```

## Step-by-Step: Full Rebuild

### 1. Regenerate Protobuf Bindings (if .proto files changed)

```bash
cd proto && make protogen
```

This runs `substreams protogen --exclude-paths sf/substreams,google` to regenerate `proto/src/pb/*.rs` from `proto/v1/*.proto`.

### 2. Build All Rust Crates

```bash
cargo build --target wasm32-unknown-unknown --release
```

Or build specific crates:

```bash
cargo build --target wasm32-unknown-unknown --release \
  -p erc20-tokens \
  -p erc20-balances \
  -p db-evm-transfers \
  -p db-evm-balances
```

### 3. Pack Level 0 (leaf crates)

```bash
cd erc20-transfers && substreams pack && cp erc20-transfers-v*.spkg ../spkg/
cd erc20-tokens    && substreams pack && cp erc20-tokens-v*.spkg ../spkg/
cd erc20-balances  && substreams pack && cp erc20-balances-v*.spkg ../spkg/
```

> **Note:** `native-transfers` and `native-balances` are typically stable and rarely need repacking.

### 4. Pack Level 1 (DB layer)

These import Level 0 `.spkg` files from `../spkg/`, so Level 0 must be packed first.

```bash
cd db-evm-transfers && substreams pack && cp evm-transfers-v*.spkg ../spkg/
cd db-evm-balances  && substreams pack && cp evm-balances-v*.spkg ../spkg/
```

### 5. Pack Level 2 (Sink layer)

These import Level 1 `.spkg` files from `../spkg/`, so Level 1 must be packed first.

```bash
cd db-evm-transfers-clickhouse && substreams pack && cp evm-clickhouse-transfers-v*.spkg ../spkg/
cd db-evm-transfers-postgres   && substreams pack && cp evm-postgres-transfers-v*.spkg ../spkg/
cd db-evm-balances-clickhouse  && substreams pack && cp evm-clickhouse-balances-v*.spkg ../spkg/
cd db-evm-balances-postgres    && substreams pack && cp evm-postgres-balances-v*.spkg ../spkg/
```

## Package Name Reference

| Directory | Package Name | .spkg Filename |
|---|---|---|
| `erc20-transfers/` | `erc20_transfers` | `erc20-transfers-v*.spkg` |
| `erc20-tokens/` | `erc20_tokens` | `erc20-tokens-v*.spkg` |
| `erc20-balances/` | `erc20_balances` | `erc20-balances-v*.spkg` |
| `native-transfers/` | `evm_native_transfers` | `evm-native-transfers-v*.spkg` |
| `native-balances/` | `evm_native_balances` | `evm-native-balances-v*.spkg` |
| `db-evm-transfers/` | `evm_transfers` | `evm-transfers-v*.spkg` |
| `db-evm-balances/` | `evm_balances` | `evm-balances-v*.spkg` |
| `db-evm-transfers-clickhouse/` | `evm_clickhouse_transfers` | `evm-clickhouse-transfers-v*.spkg` |
| `db-evm-transfers-postgres/` | `evm_postgres_transfers` | `evm-postgres-transfers-v*.spkg` |
| `db-evm-balances-clickhouse/` | `evm_clickhouse_balances` | `evm-clickhouse-balances-v*.spkg` |
| `db-evm-balances-postgres/` | `evm_postgres_balances` | `evm-postgres-balances-v*.spkg` |

## Import Map

Which `.spkg` files each package imports from `../spkg/`:

| Package | Imports |
|---|---|
| `db-evm-transfers` | `erc20-transfers`, `erc20-tokens`, `evm-native-transfers`, `substreams-database-change`, `substreams-sink-sql-protodefs` |
| `db-evm-balances` | `erc20-balances`, `evm-native-balances`, `substreams-database-change`, `substreams-sink-sql-protodefs` |
| `db-evm-transfers-clickhouse` | `evm-transfers` |
| `db-evm-transfers-postgres` | `evm-transfers` |
| `db-evm-balances-clickhouse` | `evm-balances` |
| `db-evm-balances-postgres` | `evm-balances` |

## Common Scenarios

### Proto changed (e.g., added a field)

Rebuild everything from proto down:

```bash
cd proto && make protogen
cargo build --target wasm32-unknown-unknown --release
# Then pack Level 0 → copy → Level 1 → copy → Level 2 → copy
```

### Only erc20-balances logic changed (no proto change)

```bash
cargo build --target wasm32-unknown-unknown --release -p erc20-balances -p db-evm-balances
cd erc20-balances && substreams pack && cp erc20-balances-v*.spkg ../spkg/
cd db-evm-balances && substreams pack && cp evm-balances-v*.spkg ../spkg/
cd db-evm-balances-clickhouse && substreams pack && cp evm-clickhouse-balances-v*.spkg ../spkg/
cd db-evm-balances-postgres && substreams pack && cp evm-postgres-balances-v*.spkg ../spkg/
```

### Only db-evm-transfers Rust code changed

```bash
cargo build --target wasm32-unknown-unknown --release -p db-evm-transfers
cd db-evm-transfers && substreams pack && cp evm-transfers-v*.spkg ../spkg/
cd db-evm-transfers-clickhouse && substreams pack && cp evm-clickhouse-transfers-v*.spkg ../spkg/
cd db-evm-transfers-postgres && substreams pack && cp evm-postgres-transfers-v*.spkg ../spkg/
```

## Version Bumping

Versions are defined in each `substreams.yaml` under `package.version`. When bumping:

1. Update the version in `substreams.yaml` of the changed package
2. Update the import version in any **downstream** `substreams.yaml` that references it
3. Rebuild and repack following the dependency chain above
