---
name: argo-workflows
description: Submit noop and backfill workflows via the Argo Dock API. Use when running noop sinks, backfilling ClickHouse, validating spkgs, or submitting Argo workflows.
license: Apache-2.0
compatibility:
  platforms: [claude-code, cursor, vscode, windsurf]
metadata:
  version: 1.1.0
  author: Pinax Network
  documentation: https://argo.token-api.service.dev.pinax.network/
---

# Argo Workflows

Submit noop and backfill workflows via the Argo Dock API.

## API

**Base URL**: `https://argo.token-api.service.dev.pinax.network`

**Swagger**: `GET /swagger.json`

**UI (VPN)**: `https://argo.riv-dev1.pinax.io/`

## Templates

| Template | Use |
|----------|-----|
| `token-noop` | Validate an SPKG against a chain (no DB writes) |
| `token-backfill-prod-a` | Backfill ClickHouse cluster A |
| `token-backfill-prod-b` | Backfill ClickHouse cluster B |
| `token-backfill-prod-c` | Backfill ClickHouse cluster C |
| `token-backfill-dev` | Backfill dev cluster |

## Noop Workflow

Use to validate an SPKG processes correctly before backfilling.

### Steps

1. **Get head block**:
   ```bash
   curl -s https://argo.token-api.service.dev.pinax.network/api/networks/<chain_id>
   ```

2. **Submit**:
   ```bash
   curl -s -X POST https://argo.token-api.service.dev.pinax.network/api/workflows/submit \
     -H "Content-Type: application/json" \
     -d '{
       "templateName": "token-noop",
       "parameters": [
         {"name": "range-start", "value": "0"},
         {"name": "range-stop", "value": "<head_block>"},
         {"name": "endpoint", "value": "<chain>-substreams-tier1-prod.kan-sst2.pinax.io:80"},
         {"name": "spkg", "value": "<full_github_release_spkg_url>"},
         {"name": "substreams-workers", "value": "50"},
         {"name": "other-args", "value": "--plaintext --noop-mode"}
       ]
     }'
   ```

3. **Monitor**: `GET /api/workflows/<workflow_name>`

### Tips
- Use **50 workers** for noop (speed matters, no DB contention)
- Always use the **full GitHub release URL** for spkg

## Backfill Workflow

Use after noop succeeds to write data to ClickHouse.

### Steps

1. **Get head block** from `/api/networks/<chain_id>`

2. **Calculate range-stop**: Round to nearest 10,000, several hours ahead of head to account for processing time

3. **Calculate blocks-per-job**: Divide total range by ~200 to get ~200 parallel jobs

4. **Validate params** (must pass with no errors/warnings):
   ```
   GET /api/workflows/templates/<template>/validate?parameters=<url_encoded_json>
   ```
   Parameters is a URL-encoded JSON object with key-value pairs (not an array).

5. **Get confirmation** from the team before submitting

6. **Submit**:
   ```bash
   curl -s -X POST https://argo.token-api.service.dev.pinax.network/api/workflows/submit \
     -H "Content-Type: application/json" \
     -d '{
       "templateName": "token-backfill-prod-b",
       "parameters": [
         {"name": "range-start", "value": "0"},
         {"name": "range-stop", "value": "<stop_block>"},
         {"name": "blocks-per-job", "value": "<calculated>"},
         {"name": "spkg", "value": "<full_github_release_spkg_url>"},
         {"name": "endpoint", "value": "<chain>-substreams-tier1-prod.kan-sst2.pinax.io:80"},
         {"name": "substreams-workers", "value": "1"},
         {"name": "other-args", "value": "--plaintext"},
         {"name": "ch-database", "value": "<chain>:<module>@v<version>"},
         {"name": "table-prefix", "value": "backfill"}
       ]
     }'
   ```

### Key Differences from Noop
- **1 worker** (data already cached from noop run)
- No `--noop-mode` in other-args
- Must set `ch-database` in format: `<chain>:<module>@v<version>` (e.g. `unichain:evm-contracts@v0.4.0`)
- `table-prefix` typically `backfill`

### ch-database Format

```
<chain>:<module>@v<version>
```

The chain must match the endpoint, and module/version should match the SPKG. The validator will warn on mismatches.

Examples:
- `unichain:evm-contracts@v0.4.0`
- `eth:evm-transfers@v0.4.0`
- `base:evm-dex@v0.4.0`

## Endpoint Pattern

```
<chain>-substreams-tier1-prod.kan-sst2.pinax.io:80
```

## SPKG URL Pattern

Always use the full GitHub release URL:
```
https://github.com/pinax-network/substreams-<chain_standard>/releases/download/<tag>/clickhouse-<chain_standard>-<type>-v<version>.spkg
```
