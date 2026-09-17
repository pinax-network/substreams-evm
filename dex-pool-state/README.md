# Complete-block pool state

`map_pool_state` emits `dex.pool_state.v1.BlockPoolState` once per complete block,
including blocks without changes. It composes the raw V2 closing-reserve and V3
ordered-change modules at the same Substreams block. Consumers receive both
protocols before calculating any block-level result. Mismatched block identities
or timestamps fail instead of joining unrelated observations.

- `v2_pools` retains final successful Sync reserves, including zero reserves.
- `v3_pools` retains all ordered Initialize, Swap, Mint, Burn and invalid markers.

These protocol semantics remain distinct. V3 is not a standalone snapshot: it
requires independently verified initial state and all subsequent changes. Empty
arrays are complete-block observations, not a missing protocol response. The
package does not authenticate pools or tokens, apply decimals, calculate prices,
convert currencies, or infer economic equivalence between instruments.

Build and pack from the repository root:

```sh
cargo build --locked --release --target wasm32-unknown-unknown \
  -p uniswap-v2 -p uniswap-v3 -p dex-pool-state
substreams pack dex-pool-state/substreams.yaml
```

Local manifest imports bundle the exact built dependency modules into the package.
Consumers should pin the resulting package digest and verify complete block
continuity, contract identity and supported pool models before interpreting state.
