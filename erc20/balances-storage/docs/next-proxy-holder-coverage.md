# Additional BSC proxies and deployments

This records the 87-profile batch. The subsequent
[voting-token review](voting-holder-coverage.md) adds SENTIS and STAR, with
checkpoint-array handling and historical mint regressions, bringing the test
set to 89. The evidence below retains the earlier artifacts.

The [87-profile fixture](../tests/fixtures/bsc-next-proxy-layouts.json) adds eleven
contracts from the same ranked RPC capture, using the existing single RPC-free
`map_events` and shared `evm.balances.v1.Events`. This batch changes qualified
layouts and Rust regressions; the mapper WASM is unchanged.

| Ranks | Implementation | Balance base | Qualification |
| --- | --- | ---: | --- |
| 59, 61, 74, 77, 84, 86, 96, 97 | FlapTaxTokenV3 | 51 | Canonical ERC-1167 clones; verified implementation source |
| 65 | TokenV2 | 51 | Canonical clone; existing reviewed implementation, separately pinned new deployment |
| 57 | GMToken | 51 | Reviewed BeaconProxy source, beacon getter bytecode/controls, verified implementation source |
| 64 | BTRToken | 201 | Reviewed ERC1967Proxy source and verified implementation source |

[Qualification evidence](evidence/next-proxy-qualification.json) includes exact
contracts, runtime identities, compiler storage layouts, source digests, getter
traces/controls and deployment identities. Source is bound to historical runtime.
All implementation code and proxy/beacon pointers are independently verified at
the historical boundaries. The complete 1,024-block preflight passes with no
unresolved configured writes.

FlapTaxTokenV3's transfer taxes and packed pool state affect how transfers update
storage. Its `balanceOf` still returns `_balances[account]` directly. Permit
nonces, pool membership and initialization fields are reviewed separately.
BTR's pause/quota/whitelist state and GMToken's compliance/pause managers likewise
do not transform the getter's stored amount. [Read-only controls](evidence/next-proxy-getter-controls.json)
confirm 16 raw-balance/global-field combinations. The beacon's source was
unavailable; two caller controls and its executed bytecode verify the masked
implementation read from slot 1, independently of owner slot 0.

Metadata payloads are explicitly bounded to their reviewed storage words, not
an arbitrary ignore range. Three FlapTaxTokenV3 clones use three metadata words;
the other five use two. GMToken has two-word name payloads at both the inherited
and override fields. The TokenV2 clone uses two metadata words. Header/length and
payload digests agree at both qualification boundaries. Unreviewed payload
extension, enumerable role arrays and other unknown storage still stop
processing. The source getter and actual writes, not token-family labels alone,
determine the layout.

## Deployment and initial holders

Two contracts were created during this window:

| Rank | Contract | First CREATE block | Initial emitted holders | Initial supply in integer units |
| ---: | --- | ---: | ---: | ---: |
| 59 | `0x8a119a3087799041bace3d906695baddee9e7777` | 122288475 | 7 | 1000000000000000000000000000 |
| 65 | `0x07367aeb2e969249bc2803fa2d7385db00628777` | 122288326 | 23 | 100000000000000000000000000 |

Both have empty code and zero nonce immediately before deployment. The profiles
pin the exact block hash, CREATE call, code-change ordinal and runtime. The
mapper verifies first persisted storage writes start at zero, then applies all
later writes in the same block.

All **30 original unresolved RPC responses** were empty `0x` returns at the
pre-deployment block hash: seven for rank 59 and 23 for rank 65. The
[classification record](evidence/next-proxy-predeployment-rpc.json) retains those
responses as **unavailable before verified CREATE**, not valid zero balances or
value mismatches. Initialization uses validated EVM creation state, never an
invented pre-deployment RPC result.

[Independent initial-supply checks](evidence/next-proxy-initial-supply.json)
confirm that the sum of all captured holder balances equals RPC `totalSupply`,
the stored supply word and `maxSupply` for both deployments. An initial test
assumed both families minted one billion tokens. The TokenV2 clone actually
configures 100 million, unlike its previously reviewed sibling; the regression
now uses its independently captured RPC supply. This was a test assumption,
not missing balances.

## WASM and holder replay

Across **1,024 consecutive blocks, 122288006–122289029**, the
[actual WASM audit](evidence/next-proxy-rpc.json) matches all **93,928 emitted
balances**, including **13,209 zeros**, with zero RPC mismatches. All 87
configured tokens emit rows; the eleven additions contribute **2,187 checks**.
[Per-token counts](evidence/next-proxy-rpc-token-counts.json) retain their coverage.

The [initialized-holder replay](evidence/next-proxy-holder-coverage.json) matches
all **153,616 reference observations** with zero unknown or incorrect initialized
values. **4,308 matches** carry forward without a new emission. The eleven
additions contribute **3,253 observations**, including **408 carried-forward
matches**.

Setup verifies **50,361 stored-holder checkpoints** with **100,722 balance/storage
RPC reads**. It also initializes 3,001 balances from the previously qualified
formula, 710/211 holders at the two earlier deployments, and **71/60 holders**
at the new rank-59/rank-65 deployments. Those new deployment totals include
holders first observed later in the window, not only the 7/23 creation-block
emissions. Processing makes zero balance RPC calls and 1,024 separately counted
header verification calls; reference values never repair candidate state.

Cold replay preserves **55,380 unknown observations**, including **29,415 nonzero
values**, with zero incorrect known values. The additions contribute 658 unknown
observations, including 191 nonzero values. A checkpoint or full history remains
necessary for stored balances that predate the stream. Checkpoints cover only
the bounded reference's observed holders, not every holder globally.

## Regressions and remaining work

The [eleven captured Rust cases](../tests/fixtures/next-proxies/cases.json) contain
**64 independent historical RPC balance expectations**. Transaction subsets
retain the original headers and are checked against full-block token output
before saving. Additional regressions verify complete initial supply, preserve
pre-deployment unavailability and reject storage immediately beyond each
reviewed two- or three-word metadata payload.

**140 workspace library/binary tests pass**, along with Clippy with warnings
denied, workspace WASM compilation and formatting/diff checks. Artifact and
fixture digests are recorded [here](evidence/next-proxy-artifacts.json). The
runtime has no RPC imports, no new map/cache/protobuf and no built-in token list;
production parameters remain `[]`. All executable validation tooling is Rust.

Thirteen candidates from the original top 100 remain unqualified: ranks **55,
56, 62, 67, 69, 70, 71, 72, 79, 85, 89, 92 and 93**. Two need voting checkpoint
array review; the others need further bytecode/field qualification or broader
activity evidence.

The [rank-70 activity review](evidence/next-proxy-zero-only-activity.json) explains
its weak sample: one successful call emits **159 transfer logs but performs no
token storage writes**. The RPC reference observes 298 holders with zero
balances; the storage mapper emits no balance rows. Nonzero event amounts are
not evidence of a corresponding stored balance. Its independently tested getter
does read mapping 0, but real nonzero activity and emitted-row coverage remain
unproven. The token is not counted among the 87 qualified replay profiles.

These windows overlap prior reports; their totals are not additive independent
history. Reproduce using the 87-profile fixture, `audit-rpc --start 122288006
--blocks 1024 --workers 2`, and `holder-coverage` with the original digest-matching
RPC capture and all consecutive Extended blocks in `out/top50-full-holder-blocks`.
