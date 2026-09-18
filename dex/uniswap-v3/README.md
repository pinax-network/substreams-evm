# Uniswap V3

`map_events` retains the existing transaction/event output.
The shared signed-integer conversion now preserves valid negative ticks; previously
it rejected every negative value, and existing callers substituted zero. This also
corrects negative tick/range values in the V4 and Kyber Elastic event extractors.

`map_pool_changes` emits a `uniswap.v3.BlockPoolChanges` envelope for every complete
block, including blocks without relevant changes. Block number/hash, parent hash
and nanosecond timestamp allow consumers to enforce continuity. For each pool,
Initialize, Swap, Mint and Burn changes are ordered by canonical block log index;
reverted calls and failed transactions are excluded. BASE receipt and EXTENDED
trace representations are never read together.

This output supports reconstruction of closing price and concentrated liquidity
from an independently verified initial state. It is not a standalone snapshot:
Mint/Burn events after the last Swap can alter active liquidity and initialized
tick ranges. Consumers must apply every change before calculating a block close.
Amounts remain unscaled decimal integers; a Burn has a negative liquidity delta.
Zero changes are preserved, and consumer freshness rules must decide whether a
change supplies a new observation.

A known topic with an invalid encoding/value produces an explicit `invalid` change
for that pool. Consumers must invalidate that pool's cached state, rather than
silently carry it forward. An unrelated malformed event does not hide valid changes
from other pools. Consumers still own factory/pool/token verification; a matching
ABI topic does not establish that a contract is an official V3 pool.

The module performs no token classification, decimal scaling, USD conversion,
liquidity qualification or pricing. A verified baseline, tick-range completeness,
market identity and reorg/gap handling remain downstream responsibilities.
