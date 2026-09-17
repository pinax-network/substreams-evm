# Ten additional BSC proxy and array layouts

Ranks **157, 158, 160, 165, 169, 177, 180, 186, 192 and 195** now pass the
historical qualification campaign. The [combined fixture](../tests/fixtures/bsc-more-qualified-layouts.json)
contains **188 profiles**. The interval is **122288006–122289029**, inclusive.
Twelve candidates from the reviewed top 200 remain unqualified.

Production remains one RPC-free `map_events` with shared
`evm.balances.v1.Events`, caller-qualified layouts and default parameters `[]`.
The package and WASM digests are unchanged. This extension adds qualification
fixtures, evidence and Rust regressions without changing ingestion logic.

## Getter, proxy and field qualification

The [qualification evidence](evidence/more-qualified-qualification.json) binds
verified source to the historical runtime and records each getter, explicit
storage fields, dependency identities and independent controls.

| Rank | Token symbol | Balance root and reviewed dependency |
| --- | --- | --- |
| 157 | babysuica | Mapping 3; fee-address array at 8 |
| 158 | TELEBTC | Mapping 51; ERC-1967 implementation |
| 160 | STBL | OpenZeppelin ERC-7201 ERC20 namespace; ERC-1967 implementation |
| 165 | USDDD | Same ERC20 namespace; ERC-1967 implementation |
| 169 | ARTX | Mapping 51; ERC-1967 implementation |
| 177 | LUNA | Mapping 5; beacon, beacon's delegate and token implementation |
| 180 | SpaceX Meme | Mapping 1; shareholder-address array at 28 |
| 186 | IN | Same ERC20 namespace; ERC-1967 implementation |
| 192 | APR | Same ERC20 namespace; ERC-1967 implementation |
| 195 | MERL | Same ERC20 namespace; delayed-initialization ERC-1967 proxy |

All getters return the holder's stored balance unchanged. Namespace profiles
include the reviewed ERC20 struct binding, not just an observed slot. Explicit
compiler storage fields exclude reserved gaps; other namespaces and unreviewed
writes remain unresolved. The delayed proxy's implementation is already
initialized at both boundaries; no deployment baseline is assumed.

The LUNA outer proxy has a different runtime from previously reviewed bridge
instances. Its own runtime-bound source implements the same beacon forwarding
path. The beacon, beacon delegate implementation and token implementation match
the reviewed family and are independently checked at both historical boundaries.
Protected dependency changes still stop processing, even without token activity.

All **40 balance-word controls** pass for zero, one, 123 and uint256 maximum.
Another **82 scalar-field controls** confirm the seven ERC-1967 getters remain
unchanged when their reviewed non-balance scalar fields are overridden.

## Address arrays and remaining removal gap

The [array evidence](evidence/more-qualified-address-lists.json) records **11 fee
array appends** and **one shareholder array append** across all 1,024 blocks.
Existing append validation verifies each length transition, element position,
ordinal order, empty old element and canonical address word. Unknown writes
are not discarded.

Captured regressions at blocks **122288022** and **122288619** reproduce the
unresolved-storage failures when their explicit array configurations are
removed. The reviewed configurations produce the independently captured RPC
balances. Eleven captured cases cover all ten tokens and both array cases.

**SpaceX Meme also contains a swap-and-pop removal path.** No removal occurs in
this test window. Removals remain unsupported and fail closed; the summary
records this separately from passing append behavior. A captured removal and
corresponding validation support are still required before claiming that path.

## RPC and holder evidence

The [native scan](evidence/more-qualified-native-scan.json) and
[actual packaged-WASM audit](evidence/more-qualified-rpc.json) both produce
**613 balances** across the complete interval. All **613 RPC checks**, including
**56 zeros**, match. Canonical clock evidence covers **812 empty output blocks**
and 212 blocks with output. Every configured token emits.

The [native holder audit](evidence/more-qualified-holder-coverage.json) initializes
**218 observed holders** with **436 explicit balance/storage RPC reads**. All
**998 reference observations** match, with no initialized unknowns or mismatches.
A cold consumer still has **370 unknown observations**, including **127 nonzero
values**. Those gaps require initialization or earlier history.

The [independent actual-WASM holder replay](evidence/more-qualified-wasm-holders.json)
also matches all 998 observations, including **385 without a new event**. Its
final canonical RPC snapshot matches **all 218 retained holders**, including
**117 zeros**. Reference observations never repair retained state; processing
makes no balance RPC calls.

All **205 workspace Rust library/binary tests pass**, together with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
Repackaging preserves the exact package and WASM used by the previous cohorts.

## Combined scope

The [combined evidence](evidence/more-qualified-summary.json) totals **103,022
emitted RPC checks**, including **14,524 zeros**, and **168,800 initialized holder
observations**, all matching. These combine disjoint 140-, 1-, 8-, 29- and
10-token WASM runs on the same package and canonical interval. A combined native
replay of all 188 profiles matches their complete row/value union in every block.
No new combined 188-token WASM run is claimed.

Remaining candidate ranks are **143, 151, 167, 168, 172, 174, 179, 189, 194, 196,
198 and 199**. [LBP's reward-driven holder mismatches](lbp-reward-mismatch.md)
remain unresolved; these passing profiles do not establish support for that
behavior. Source-visible array removal is also retained as outstanding work.

This is bounded token and observed-holder evidence, not global holder enumeration
or universal ERC-20 support. The requested network sequence remains
[Ethereum, Base, HyperEVM and Arc](network-expansion.md) after the BSC work, with
separate historical runtime, Extended-block and holder qualification.
