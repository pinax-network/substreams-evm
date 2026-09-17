# BSC top-350 follow-up: remaining direct getters and proxies

This follow-up qualifies 13 more profiles from the original top 350 RPC-stream
candidates, bringing the explicit test configuration to **345 profiles** in
[`bsc-pending350-layouts.json`](../tests/fixtures/bsc-pending350-layouts.json).
LBP, TITAN, ORD, YBC and 钻石 remain outside production qualification.

The subsequent [XVS width follow-up](xvs-uint96.md) adds a 346th profile and a
new audited package. The measurements and artifact statements below retain
this 13-token cohort's original scope.

The production interface stays one RPC-free `map_events`, the shared
`evm.balances.v1.Events` protobuf, explicit caller-qualified layouts and default
parameters `[]`. These profiles require no production mapper change. All
executable diagnostics and regression tests are Rust.

## Qualification

The [qualification report](evidence/pending350-qualification.json) records the
complete reviewed getter paths, independent word/account/caller controls and
explicit storage fields. [Fresh canonical runtime checks](evidence/pending350-runtime-checks.json)
bind the tokens and their dependencies at both interval boundaries.

| Rank | Token | Qualification detail |
| --- | --- | --- |
| 303 | SPARK | Unconditional slot-0 balance mapping |
| 312 | MOMO | Unconditional slot-0 balance mapping |
| 319 | 金麟人生 | Slot 0; first CREATE validated at block 122288321 |
| 321 | 牛来金库 | Unconditional slot-3 balance mapping |
| 325 | NVB2 | Unconditional slot-0 balance mapping |
| 326 | HIGH | Pinned minimal proxy; verified SynapseERC20 implementation, slot 101 |
| 327 | Tracker | Slot-0 balance; separately reviewed dividend bookkeeping |
| 332 | 招财牛 | Pinned minimal proxy; slot 0 and independent scalar 8 |
| 333 | AUT | Unconditional slot-0 balance mapping |
| 339 | O | Unconditional slot-5 balance mapping |
| 341 | TAKE | Verified implementation and namespaced ERC-20 balance mapping |
| 342 | STBU | Verified historical source and slot-6 getter path |
| 345 | IRYS | Verified implementation and namespaced ERC-20 balance mapping |

Tracker's coincident slot-12 values are separate bookkeeping. Its `balanceOf`
reads only mapping root 0. The report reviews scalar roots 7, 10 and 15 and
mapping roots 8, 9, 12 and 17; 42 read-only field controls leave the balance
unchanged. Unreviewed array or membership writes remain strict errors.

招财牛's scalar 8 is exposed by selector `0xc5c03af3`. Its getter reads that
scalar directly; the complete balance getter reads only mapping root 0. Six
independence controls cover zero, 123 and maximum scalar words with balance
words zero and one. No source field name or reentrancy semantics are inferred.

For 金麟人生, parent code and nonce are absent and the creation hash is pinned.
The mapper validates the CREATE, code and nonce evidence before a holder replay
can initialize empty storage. Its predeployment `balanceOf` response remains
empty, not a claimed RPC zero. The original failed exploratory check is retained
in the [initial investigation](ranks301-350-investigation.md).

All 13 profiles pass a complete native replay of **1,024 consecutive Extended
blocks**, 122288006 through 122289029, with no unresolved writes.
The [initial strict write review](evidence/pending350-initial-write-review.json)
preserves the rejected deployment and unknown bookkeeping writes before these
explicit rules were added; it does not represent a qualified configuration.

## Packaged RPC and holder evidence

The [actual packaged-WASM audit](evidence/pending350-rpc.json) matches all
**295 emitted balances**, including **21 zeros**, with historical RPC. Complete
canonical clocks account for empty-output blocks.

The [native holder replay](evidence/pending350-holder-coverage.json) initializes
165 existing observed holders using **330 explicit balance/storage reads**.
Nine more holders receive a validated empty-storage baseline at the new token's
CREATE. The [actual-WASM holder replay](evidence/pending350-wasm-holders.json)
then confirms:

- **517 initialized reference observations**, with no mismatches.
- **222 matches without a new balance event**.
- All **174 final holder balances** match fresh RPC, including **77 zeros**.
- Processing performs no balance RPC calls; reference rows never repair state.
- Cold state preserves **206 unknown observations**, including **41 nonzero**.

The [fresh combined capture](evidence/pending350-combined.json) preserves every
protobuf event field for **108,507 previously RPC-verified emitted balances**,
including **15,057 zeros**. There are 344 emitting profiles; hLBP is quiet in
this interval and retains its separate older mint evidence. Identical combined
event histories preserve evidence for **177,744 initialized observations** and
**69,238 matches without a new event**.

The combined comparison reuses immutable RPC evidence with fresh canonical
headers and runtime checks. It does not create a new full-345 checkpoint or
global holder snapshot. The separately initialized 13-token cohort supplies
the new holder evidence.

Thirteen captured transaction fixtures retain **29 independently checked
balances** in [Rust regression tests](../src/pending350_tests.rs). All
**252 workspace library/binary tests** pass, as do Clippy with warnings denied,
workspace WASM compilation, scoped formatting and diff checks. Production SPKG
and WASM hashes are unchanged from the previous cohort.

These are bounded runtime and interval qualifications. A checkpoint covers only
the observed holder set. Neither a matching emitted row nor a final observed
holder snapshot establishes complete global holder enumeration.

## Why 钻石 remains excluded

The [external getter diagnostic](evidence/diamond-external-getter.json) confirms
that rank 338, `0x26bfefbad1bc1f6979ad92e544171f0b600c8888`, conditionally adds
an external contract's uint256 return to its raw slot-0 balance. A call trace
binds selector `0xf40f0f52` to the external proxy and its implementation. The
token and dependency themselves bypass the contribution.

Sixteen fresh historical boundary checks cover all eight holders in its
38-row reference stream. All match raw storage: those sampled contributions
are zero. Twenty individual dependency-word overrides also leave the raw
balance unchanged; these unsuccessful activation attempts are preserved.

Twelve read-only external-return controls demonstrate the missing behavior:
for ordinary holders a simulated return of 17 changes balances 0 and 1 to 17
and 18, and maximum raw balance plus one reverts with checked overflow. The
two exempt holders retain their raw balances. These are simulated effects,
not reported historical mismatches.

A complete external computation and retained state model is still needed.
Raw-word equality in the sampled interval is insufficient to promote this token.
LBP, TITAN, ORD and YBC retain their previously documented limitations. After
the BSC review, [Ethereum, Base, HyperEVM and Arc](network-expansion.md) need
independent token and holder qualification; current RPC prerequisite probes do
not establish parity on those networks.
