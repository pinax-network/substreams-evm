# BSC candidates ranked 151–200

This cohort adds **29 qualified profiles**, bringing the
[combined fixture](../tests/fixtures/bsc-ranks151-200-layouts.json) to **178**.
The test interval remains **122288006–122289029**, inclusive. The original RPC
capture and ranking are unchanged. The other 21 candidates in this cohort,
plus reward-bearing LBP from the previous cohort, remain unqualified.

Production still has one RPC-free `map_events`, shared `evm.balances.v1.Events`,
caller-qualified layouts and default parameters `[]`. This extension changes
qualification fixtures and Rust regressions; it requires no ingestion change.

## Reviewed layouts

The [qualification evidence](evidence/ranks151-200-qualification.json) records
each profile, historical getter controls and source/storage-field review.

| Basis | Ranks |
| --- | --- |
| Runtime-bound verified source; direct balance mapping | 153, 154, 156, 161, 162, 163, 164, 166, 170, 171, 173, 175, 176, 178, 181, 182, 183, 184, 185, 190, 191, 193, 197, 200 |
| Previously reviewed canonical minimal proxy; independently pinned implementation | 152, 188 |
| Previously reviewed beacon family; independently pinned beacon and implementation | 155, 187 |
| Previously reviewed clone with its own pinned zero-word fallback | 159 |

The 24 direct getters return the address's balance word unchanged. Their
explicit scalar and mapping fields are checked against compiler storage layout.
Dynamic data outside those reviewed fields remains unqualified; a new unresolved
write still stops processing. Matching outer proxy bytecode alone is insufficient:
each reused family passes independent historical dependency checks at both
boundaries. Deployment assumptions from another instance are removed.

All 29 profiles pass **116 read-only balance-word controls**, covering zero,
one, 123 and uint256 maximum. The complete native replay emits **1,811 balances**
without unresolved writes across all 1,024 blocks. All 29 tokens emit.

## Why ARC initially mismatched

The [original survey](evidence/ranks151-200-survey.json) retains its one value
mismatch. The BSC token named **ARC**, rank 159 at
`0x32b133ca38c9b410a053f2bcfeea83831c3bcfe0`, returns **1,000,000,000 raw units**
when the holder's mapping word is zero. Its getter reads this fallback from
scalar slot 8. Nonzero balance words are returned unchanged.

The [diagnosis](evidence/ranks151-200-fallback-diagnosis.json) preserves the
original failing observation. The clone and implementation runtimes match the
reviewed family, and historical checks independently confirm this instance's
fallback at both boundaries. Configuring that existing rule resolves the
mismatch. Any persisted change to the fallback remains protected.

Of its **63 initialized holders**, **60** have zero raw storage and a positive
RPC balance. A captured Rust regression verifies all 63 and reproduces the
incorrect zero result when the fallback is removed. Its 111 reference
observations pass; 87 require initialized state without a new matching event.
This token symbol is unrelated to the requested **Arc network** qualification.

## Actual WASM and retained holders

The [packaged-WASM audit](evidence/ranks151-200-rpc.json) matches all **1,811
emitted balances**, including **196 zeros**, against canonical historical RPC.
Complete canonical clock evidence covers **606 empty output blocks** and 418
blocks with output. The package and WASM digests are unchanged.

The [native holder replay](evidence/ranks151-200-holder-coverage.json) initializes
**1,151 observed holders** using **2,302 explicit balance/storage RPC reads**.
All **2,891 reference observations** match, with no initialized unknowns or
mismatches. Cold-start gaps remain explicit: **1,051 unknown observations**,
including **565 nonzero values**, cannot be reconstructed from new writes alone.

The [independent replay of actual WASM](evidence/ranks151-200-wasm-holders.json)
matches all 2,891 observations, including **1,081 without a new event**. Its final
canonical RPC snapshot matches **all 1,151 retained holders**, including **408
zeros**. Reference rows never repair retained state. Processing makes no balance
RPC calls; initialization and verification are measured separately.

Thirty captured block cases cover all 29 profiles and the original ARC mismatch
block. The workspace has **203 passing Rust library/binary tests**. Clippy with
warnings denied, workspace WASM compilation, formatting and diff checks pass.

## Combined scope and next candidates

The [combined evidence](evidence/ranks151-200-summary.json) totals **102,409
emitted RPC checks**, including **14,468 zeros**, and **167,802 initialized holder
observations**, all matching. These totals combine disjoint 140-, 1-, 8- and
29-token actual-WASM runs on the identical package and canonical interval.
A combined native replay of all 178 layouts matches their full row/value union
in every block. This is not a new combined 178-token WASM capture.

The summary preserves every remaining candidate. Untested proxy implementations,
unreviewed fields and missing direct-mapping observations are not promoted by
their sampled values. The separate [LBP reward counterexamples](lbp-reward-mismatch.md)
remain unresolved: 33 holders change balance without raw balance-word writes.

These are bounded token and holder results, not complete global holder
enumeration or universal ERC-20 support. Holders need a qualified checkpoint or
full-history replay. After the BSC candidate review, the requested sequence is
[Ethereum, Base, HyperEVM and Arc](network-expansion.md), with separate runtime,
Extended-block and holder evidence for each network.
