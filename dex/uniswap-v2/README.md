# Uniswap V2 extraction

`map_events` retains the transaction-level event output. `map_pool_closes`
provides a separate `uniswap.v2.BlockPoolCloses` output for state consumers.

The latter emits an envelope for every complete block, including its number,
hash, parent hash, and timestamp with nanoseconds. It selects the last successful
`Sync` for each pool by the canonical block log index, including reserve changes
from liquidity operations. Pools are sorted by address for deterministic output.
Failed transactions and reverted calls cannot set a closing observation.

A matching `Sync` with malformed topics/data or a noncanonical uint112 value
produces `invalid: true` with empty reserve strings. Any such event invalidates
that pool for the entire block, including when a later Sync is well formed.
Consumers must discard its cached reserves. The marker does not invent zero
liquidity or prevent unrelated pools from being extracted.

Reserve values remain unsigned integer strings without floating-point conversion.
Zero reserves are preserved: consumers must observe an emptied pool. Only pools
that changed in this block are listed. An absent pool does not mean zero reserves
and does not supply a new observation. A consumer that retains earlier reserves
must retain their original observation time and handle gaps and reorgs.

This is raw event extraction. A contract with a matching event signature is not
automatically an authentic Uniswap pool. Consumers must independently verify pool
origin, contract code, token ordering, token identity and decimals. This module
does not establish USD prices, liquidity quality or a TWAP, and it does not expose
a full pool-state snapshot for a consumer joining mid-stream.

## Build and validate

From the repository root:

```sh
cargo test --locked -p uniswap-v2
cargo build --locked --release --target wasm32-unknown-unknown -p uniswap-v2
substreams pack dex/uniswap-v2/substreams.yaml
```

The unit tests cover final-state selection after an intrablock round trip, empty
liquidity, exact uint112 reserves, unordered input logs, reverted calls, failed
transactions, malformed Sync invalidation, empty blocks and missing metadata. No live-source coverage is
implied by these synthetic tests.
