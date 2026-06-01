# `dex-swaps-pools`

Legacy **key-value store** of DEX pool metadata for EVM chains.

Functionally ~1:1 with [`evm-dex-foundational-store`](../evm-dex-foundational-store), but:

- Output is a **legacy Substreams key-value store** (`updatePolicy: set_if_not_exists`,
  `valueType: proto:dex.foundational_store.v1.Pool`) instead of foundational `SinkEntries`.
- Pool data is extracted **directly from a single `sf.ethereum.type.v2.Block`** using
  [`substreams-abis`](https://github.com/pinax-network/substreams-abis) decoders — it does
  **not** import per-protocol spkg packages.

## Store layout

| | |
|---|---|
| **key** | hex-encoded pool address (lowercase, no `0x`) |
| **value** | `dex.foundational_store.v1.Pool { tokens[], factory }` |
| **policy** | `set_if_not_exists` — first writer wins (mirrors foundational `if_not_exist: true`) |

See [keys in stores](https://docs.substreams.dev/reference-material/manifest-and-components/keys-in-stores).

Payload is intentionally restricted to shared metadata: `tokens[]` plus `factory` when
available. Protocol-specific init fields (Aerodrome `stable`, TraderJoe `bin_step`, Kyber
`swap_fee_units`/`tick_distance`, …) remain on the original pool-creation events.

## Protocols covered

Uniswap v1–v4, Aerodrome, Kyber Elastic, TraderJoe, Balancer v3, Bancor, SunPump, CurveFi
(factory deployments + direct constructor deployments).

## Build

```bash
make build   # compile wasm
make pack    # -> ../spkg/dex-swaps-pools-v0.1.0.spkg
make gui     # run against ENDPOINT
```
