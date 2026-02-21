---
name: release-conventions
description: Naming conventions for tags, releases, and SPKGs in the substreams-evm monorepo. Use when creating releases, tagging versions, or publishing packages.
license: Apache-2.0
compatibility:
  platforms: [claude-code, cursor, vscode, windsurf]
metadata:
  version: 1.0.0
  author: Pinax Network
  documentation: https://github.com/pinax-network/substreams-evm
---

# Release & Packaging Conventions

Naming conventions and release workflow for Pinax `substreams-*` monorepos.

## URL Pattern

```
https://github.com/pinax-network/substreams-<chain>/releases/download/<chain>-<type>-v<version>/clickhouse-<chain>-<type>-v<version>.spkg
```

Examples:
- `https://github.com/pinax-network/substreams-evm/releases/download/evm-contracts-v0.4.0/clickhouse-evm-contracts-v0.4.0.spkg`
- `https://github.com/pinax-network/substreams-evm/releases/download/evm-balances-v0.3.3/clickhouse-evm-balances-v0.3.3.spkg`
- `https://github.com/pinax-network/substreams-svm/releases/download/svm-dex-v0.2.5/clickhouse-svm-dex-v0.2.5.spkg`

## Tag Format

```
<chain>-<type>-v<version>
```

Examples: `evm-contracts-v0.4.0`, `evm-dex-v0.4.0`, `svm-dex-v0.2.5`

## Release Name

```
<chain>-<type> v<version>
```

Examples: `evm-contracts v0.4.0`, `evm-dex v0.4.0`

## SPKG Naming

Each crate produces up to 3 SPKGs. All share the same version number.

| Type | Pattern | Example |
|------|---------|---------|
| Base DB module | `<chain>-<type>-v<version>.spkg` | `evm-contracts-v0.4.0.spkg` |
| ClickHouse sink | `clickhouse-<chain>-<type>-v<version>.spkg` | `clickhouse-evm-contracts-v0.4.0.spkg` |
| PostgreSQL sink | `postgres-<chain>-<type>-v<version>.spkg` | `postgres-evm-contracts-v0.4.0.spkg` |

> **Key**: Engine prefix comes first (`clickhouse-` / `postgres-`), then chain+type.

## substreams.yaml Package Names

| Type | Pattern | Example |
|------|---------|---------|
| Base DB module | `<chain>_<type>` | `evm_contracts` |
| ClickHouse sink | `<chain>_clickhouse_<type>` | `evm_clickhouse_contracts` |
| PostgreSQL sink | `<chain>_postgres_<type>` | `evm_postgres_contracts` |

## SPKG Distribution

SPKGs must be placed in **two locations**:

1. `./spkg/` folder in the repo (so downstream modules can import them)
2. GitHub release assets (for external consumers / k8s templates)

## Release Workflow

1. **Bump version** in all relevant `substreams.yaml` files (base, clickhouse, postgres)
2. **Update import paths** in clickhouse/postgres `substreams.yaml` to reference the new base spkg version
3. **Build**: `cargo build --target wasm32-unknown-unknown --release -p <crate>`
4. **Pack** each module: `substreams pack` in the base, clickhouse, and postgres directories
5. **Copy SPKGs** to `./spkg/` (with correct naming) and remove old versions
6. **Commit** version bumps + spkg files
7. **Tag**: `git tag -a <chain>-<type>-v<version> -m "<chain>-<type> v<version>"`
8. **Push** commit and tag
9. **Create GitHub release** with the tag, attach all 3 SPKGs as assets

## Version Alignment

All 3 SPKGs for a given crate (base, clickhouse, postgres) **must share the same version**.

## Directory Structure

```
<chain>-<type>/
├── substreams.yaml          # Base DB module
├── src/lib.rs
├── Cargo.toml
├── clickhouse/
│   ├── substreams.yaml      # ClickHouse sink (imports base spkg)
│   ├── schema.sql
│   ├── Makefile
│   └── .gitignore           # !schema.sql to override root .gitignore
└── postgres/
    ├── substreams.yaml      # PostgreSQL sink (imports base spkg)
    ├── schema.sql
    ├── Makefile
    └── .gitignore           # !schema.sql to override root .gitignore
```
