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
Level 0 — Leaf crates (event extraction, no in-repo .spkg dependencies):
  ├── erc20/          → erc20-v*.spkg
  ├── erc1155/        → erc1155-v*.spkg
  ├── native/         → native-v*.spkg
  ├── blocks/         → blocks-v*.spkg
  ├── dex/            → dex-v*.spkg
  └── dex-nfts/       → dex-nfts-v*.spkg

Level 1 — DB crates (imports Level 0 .spkg files, outputs DatabaseChanges):
  ├── evm-transfers/  → evm-transfers-v*.spkg
  ├── evm-balances/   → evm-balances-v*.spkg
  ├── evm-contracts/  → evm-contracts-v*.spkg
  ├── evm-dex/        → evm-dex-v*.spkg
  ├── evm-nfts/       → evm-nfts-v*.spkg
  └── evm-supply/     → evm-supply-v*.spkg
```

> **Note:** There are no separate ClickHouse/PostgreSQL directories. Sink configuration (SQL schemas, engine selection) is defined in the `substreams.yaml` within each DB crate.

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
  -p erc20 \
  -p db-evm-transfers \
  -p db-evm-balances
```

### 3. Pack Level 0 (leaf crates)

```bash
cd erc20   && substreams pack && cp erc20-v*.spkg ../spkg/
cd erc1155 && substreams pack && cp erc1155-v*.spkg ../spkg/
cd native  && substreams pack && cp native-v*.spkg ../spkg/
cd blocks  && substreams pack && cp blocks-v*.spkg ../spkg/
cd dex     && substreams pack && cp dex-v*.spkg ../spkg/
```

### 4. Pack Level 1 (DB crates)

These import Level 0 `.spkg` files from `../spkg/`, so Level 0 must be packed first.

```bash
cd evm-transfers && substreams pack && cp evm-transfers-v*.spkg ../spkg/
cd evm-balances  && substreams pack && cp evm-balances-v*.spkg ../spkg/
cd evm-contracts && substreams pack && cp evm-contracts-v*.spkg ../spkg/
cd evm-dex       && substreams pack && cp evm-dex-v*.spkg ../spkg/
cd evm-nfts      && substreams pack && cp evm-nfts-v*.spkg ../spkg/
cd evm-supply    && substreams pack && cp evm-supply-v*.spkg ../spkg/
```

## Package Name Reference

| Directory | Package Name | .spkg Filename |
|---|---|---|
| `erc20/` | `erc20` | `erc20-v*.spkg` |
| `erc1155/` | `erc1155` | `erc1155-v*.spkg` |
| `native/` | `native` | `native-v*.spkg` |
| `blocks/` | `blocks` | `blocks-v*.spkg` |
| `dex/` | `dex` | `dex-v*.spkg` |
| `dex-nfts/` | `dex_nfts` | `dex-nfts-v*.spkg` |
| `evm-transfers/` | `evm_transfers` | `evm-transfers-v*.spkg` |
| `evm-balances/` | `evm_balances` | `evm-balances-v*.spkg` |
| `evm-contracts/` | `evm_contracts` | `evm-contracts-v*.spkg` |
| `evm-dex/` | `evm_dex` | `evm-dex-v*.spkg` |
| `evm-nfts/` | `evm_nfts` | `evm-nfts-v*.spkg` |
| `evm-supply/` | `evm_supply` | `evm-supply-v*.spkg` |

## Common Scenarios

### Proto changed (e.g., added a field)

Rebuild everything from proto down:

```bash
cd proto && make protogen
cargo build --target wasm32-unknown-unknown --release
# Then pack Level 0 → copy → Level 1 → copy → Level 2 → copy
```

### Only erc20 logic changed (no proto change)

```bash
cargo build --target wasm32-unknown-unknown --release -p erc20 -p db-evm-transfers -p db-evm-balances
cd erc20 && substreams pack && cp erc20-v*.spkg ../spkg/
cd evm-transfers && substreams pack && cp evm-transfers-v*.spkg ../spkg/
cd evm-balances && substreams pack && cp evm-balances-v*.spkg ../spkg/
```

### Only evm-transfers Rust code changed

```bash
cargo build --target wasm32-unknown-unknown --release -p db-evm-transfers
cd evm-transfers && substreams pack && cp evm-transfers-v*.spkg ../spkg/
```

## Version Bumping

Versions are defined in each `substreams.yaml` under `package.version`. When bumping:

1. Update the version in `substreams.yaml` of the changed package
2. Update the import version in any **downstream** `substreams.yaml` that references it
3. Rebuild and repack following the dependency chain above
