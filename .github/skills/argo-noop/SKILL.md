---
name: argo-noop
description: Submit noop sink workflows via the Argo Dock API to validate Substreams packages against a chain. Use when running noop sinks, validating spkgs, or submitting Argo workflows.
license: Apache-2.0
compatibility:
  platforms: [claude-code, cursor, vscode, windsurf]
metadata:
  version: 1.0.0
  author: Pinax Network
  documentation: https://argo.token-api.service.dev.pinax.network/
---

# Argo Noop Workflow

Submit noop sink workflows to validate Substreams packages against a blockchain.

## API

**Base URL**: `https://argo.token-api.service.dev.pinax.network`

**Swagger**: `GET /swagger.json`

## Workflow

### 1. Get the chain's head block

```bash
curl -s https://argo.token-api.service.dev.pinax.network/api/networks/<chain_id>
```

Use `headBlockNumber` as the `range-stop`.

### 2. Submit the noop workflow

```bash
curl -s -X POST https://argo.token-api.service.dev.pinax.network/api/workflows/submit \
  -H "Content-Type: application/json" \
  -d '{
    "templateName": "token-noop",
    "parameters": [
      {"name": "range-start", "value": "0"},
      {"name": "range-stop", "value": "<head_block_number>"},
      {"name": "endpoint", "value": "<chain>-substreams-tier1-prod.kan-sst2.pinax.io:80"},
      {"name": "spkg", "value": "https://github.com/pinax-network/substreams-<chain_standard>/releases/download/<tag>/clickhouse-<chain_standard>-<type>-v<version>.spkg"},
      {"name": "substreams-workers", "value": "50"},
      {"name": "other-args", "value": "--plaintext --noop-mode"}
    ]
  }'
```

### 3. Monitor

Check workflow status:
```bash
curl -s https://argo.token-api.service.dev.pinax.network/api/workflows/<workflow_name>
```

## Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `range-start` | Start block | `0` |
| `range-stop` | Stop block (use head block from networks API) | `40877365` |
| `endpoint` | Substreams tier1 endpoint | `unichain-substreams-tier1-prod.kan-sst2.pinax.io:80` |
| `spkg` | URL to the SPKG file (GitHub release asset) | `https://github.com/pinax-network/substreams-evm/releases/download/evm-contracts-v0.4.0/clickhouse-evm-contracts-v0.4.0.spkg` |
| `module` | Module name (default: `db_out`) | `db_out` |
| `substreams-workers` | Parallel workers | `50` |
| `other-args` | Extra CLI args | `--plaintext --noop-mode` |

## Endpoint Pattern

```
<chain>-substreams-tier1-prod.kan-sst2.pinax.io:80
```

Examples:
- `unichain-substreams-tier1-prod.kan-sst2.pinax.io:80`
- `eth-substreams-tier1-prod.kan-sst2.pinax.io:80`

## Templates

List available templates:
```bash
curl -s https://argo.token-api.service.dev.pinax.network/api/workflows/templates
```

Current templates: `token-noop`, `token-backfill-dev`, `token-backfill-prod-a`, `token-backfill-prod-b`, `token-backfill-prod-c`
