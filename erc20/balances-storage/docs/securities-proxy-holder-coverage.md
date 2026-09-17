# Three more proxy balance layouts

The [42-token fixture](../tests/fixtures/bsc-securities-proxy-layouts.json) adds
three contracts from the original top-50 BSC RPC stream. Rankings count balance
rows, not market capitalization. All layouts remain explicit test parameters;
production defaults remain `[]`.

| Rank | Contract | Reviewed implementation | Balance base |
| ---: | --- | --- | --- |
| 20 | `0x205812cdbed920aff76c6580abd681a46d11efc7` | SecuritiesToken | ERC-7201 ERC20 namespace |
| 24 | `0xce24439f2d9c6a2289f741120fe202248b666666` | StablecoinV2 | 51 |
| 27 | `0x02fca66c1d1afb4e2a7884261eb00f63598a7436` | SecuritiesToken | ERC-7201 ERC20 namespace |

## Getter and dependency qualification

The [qualification record](evidence/securities-proxy-qualification.json) retains
verified source provenance, historical runtime bindings, storage layouts, getter
traces, read-only state overrides and dependency boundary checks. The two
implementation runtimes match verified source at blocks 122288005 and 122289029.
All three proxy getters and both implementations pass raw mapping-word controls
for 0, 1, 123 and uint256 max.

The SecuritiesToken proxies have identical runtime bytes, including the immutable
beacon address `0x156d6dce9a4f6139a3406f1f021f1a4880de93a3`. Rank 27 has verified
proxy source; rank 20 is bound to it through byte-for-byte runtime equality.
Both delegate to `0xcfed6c4679297ea4889f8183bc057b4a86c64e46`, whose inherited
`balanceOf` reads the mapping rooted at
`0x52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00`.

The beacon itself did not have verified source in the Sourcify response. Its
small runtime is retained in the evidence. The reviewed `implementation()` path
dispatches to PC 110, reads slot 1 at PC 113, masks it to an address and returns.
That path has no caller, timestamp, block-number or external-call dependency.
Independent slot overrides return zero, `0x11…11` and `0xff…ff` as expected.
The fixture pins the proxy, beacon and implementation runtimes, and the beacon's
implementation pointer. It also verifies the EIP-1967 beacon mirror against the
embedded immutable address. Changing that mirror stops processing, even though
this proxy runtime uses the immutable address.

SecuritiesToken also exposes `balanceOfUI`, which applies a scheduled multiplier.
The RPC reference calls `balanceOf`, so storage output must remain raw. At the
canonical setup block, independent overrides demonstrate raw **123** alongside
UI **246** with an effective next multiplier of 2, and raw **123** alongside UI
**369** with a future next multiplier and current multiplier of 3. These are
read-only simulations, not transactions. No balance correction was needed.

The transparent StablecoinV2 proxy has verified source and delegates to
`0xbef21313c69c009fd7d9510a8d3a481a32473dfc`. Its inherited `balanceOf` directly
returns mapping 51. Pause and frozen-holder flags restrict transfers rather than
transforming that getter. An independent override with both flags set still
returns the raw balance of **123**. The current proxy admin is nonzero; the
reference's zero caller follows the implementation path. Admin writes are not
silently ignored by the profile.

Reviewed non-balance configuration includes namespaced allowances, AccessControl
role data, initialization, UI schedules, compliance/pause-manager pointers, and
ordinary metadata headers for SecuritiesToken. StablecoinV2 includes allowances,
permit nonces, frozen flags, authorization nonces and administrative scalars.
Unreviewed role-enumeration array elements, long dynamic metadata and other
unknown writes stop processing. These profiles do not claim all possible future
administrative operations.

## Actual WASM and retained holders

The [WASM audit](evidence/securities-proxy-rpc.json) checks **1,024 consecutive
blocks, 122288006–122289029**. All **83,853 emitted balances** match canonical
hash-pinned RPC, including **11,848 zeros**. Every configured token emits rows;
the three additions contribute **1,564 checks**. [Per-token counts](evidence/securities-proxy-rpc-token-counts.json)
retain the coverage of each profile.

The [first audit](evidence/securities-proxy-rpc-incomplete.json) stopped on an RPC
transport failure after 586 checked blocks and 51,544 successful comparisons,
with zero mismatches. Its complete WASM capture was retained. The final audit
rechecks every row of that same capture with two workers, revalidates runtime
boundaries and canonical headers, and records the original report and capture
digests. The failed run is not presented as a pass or added to the final totals.

The [holder replay](evidence/securities-proxy-holder-coverage.json) covers **452
consecutive blocks, 122288006–122288457**. All **66,051 reference observations
across 42 tokens** match initialized state, with zero unknown or incorrect
initialized values. The three additions contribute **1,243 observations**,
including six carried-forward observations without a new balance emission.

Initialization uses **25,489 stored-holder checkpoints** (50,978 balance/storage
RPC reads), 3,001 computed initial balances and 19/184 holders at the previously
qualified deployments. Processing makes no balance RPC calls and never uses
reference amounts to repair candidate state. The checkpoint covers only holders
observed in the bounded reference window, not every holder on the chain.

Cold replay retains **24,117 unknown observations**, including **12,889 nonzero
values**, with zero incorrect known values. The three additions account for
486 unknown observations, including 248 nonzero values. A verified checkpoint
or complete history is still required for stored balances predating the stream.
These ranges overlap earlier validation; their counts are not additive history.

## Rust regressions and remaining coverage

The [captured cases](../tests/fixtures/securities-proxy/cases.json) preserve actual
transactions and headers for all three proxies, with **13 independent RPC
expectations**. Before saving each fixture, its output was checked against the
full captured block. Filtering retains transactions touching proxy dependencies
as well as the token. Live audits and holder replay use full blocks.

Three new Rust regressions cover those captured balances, token UI-slot updates
versus beacon implementation updates, and rejection of unreviewed role-array or
transparent-proxy admin writes. **130 workspace library/binary tests pass**;
Clippy with warnings denied, workspace WASM compilation and targeted formatting
also pass. Package, runtime, layout and fixture digests are in
[artifacts](evidence/securities-proxy-artifacts.json).

The balance algorithm is unchanged. There remains one RPC-free `map_events`
using shared `evm.balances.v1.Events`; no extra map/cache, protobuf, generated
binding or token default is added. The WASM is identical to the preceding batch.

**Eight top-50 candidates remain unqualified**: ranks 16, 19, 23, 29, 30, 33, 41
and 46, whose source lookups did not provide verified implementations. Getter
controls alone do not establish their complete semantics. Global holder
enumeration, production bootstrap and identical raw event-row coverage also
remain open.

Reproduce the stream audit with `audit-rpc --start 122288006 --blocks 1024`, the
42-token fixture and an appropriate Substreams endpoint. Reproduce holder state
with `holder-coverage`, the digest-matching `out/top50-1024/report.json` reference,
full Extended blocks 122288006–122288457 and that fixture. Use fresh output
directories. All executable tooling and tests are Rust.
