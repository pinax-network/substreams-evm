# Final BSC proxy candidates and log-only review

The [99-profile fixture](../tests/fixtures/bsc-final-proxy-layouts.json) adds
4Stock and CAP to the previous 97 contracts from the original BSC top-100
capture. These additions use reviewed historical bytecode and independent RPC
controls; verified source was unavailable.

| Rank | Token | Contract | Balance storage |
| --- | --- | --- | --- |
| 56 | 4Stock | `0xd270d4e1ec6e6e0d28c0ecb8be966ec75997ffff` | Mapping 1 through a pinned ERC-1167 implementation |
| 85 | CAP | `0x99991c6aabba5a096f24f250b73580f5179b9999` | ERC-7201-style namespace `0x52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00` through a pinned ERC-1967 implementation |

[Qualification evidence](evidence/final-proxy-qualification.json) records each
layout, implementation runtime, executed getter path, boundary hashes, standard
getter traces and read-only controls. Both getters decode the holder address,
load the configured mapping word and return it unchanged. Four raw-word controls
(zero, one, 123 and uint256 maximum) and four address/caller controls pass per
token. CAP's other configured fields are the traced allowance mapping and
supply/name/symbol/owner scalars.

## 4Stock reward accounting

Mapping 36 initially appeared to be another balance candidate because sampled
values agreed with `balanceOf`. Overriding that mapping does not change the
getter; overriding mapping 1 does. The reviewed getter reads mapping 1 at
implementation PC 9263. This is why sample agreement does not promote a layout.

The initial scan with only standard ERC-20 fields failed on **92 blocks** due
to unclassified reward bookkeeping. These were extraction errors, not observed
incorrect balance values. The explicit additions are:

- Mapping 36 fields 0–3, holding reward shares and accrual/payment accounting.
  The transfer path reads the actual mapping-1 balance and adjusts these fields;
  the payment path calls a separate reward token and updates fields 1–3.
- Scalar 25, which packs a helper address and flags. The captured transfer sets
  and clears bit 168 at PCs 17099 and 17114.
- Scalars 37, 40 and 47, used by the reviewed share-total, accumulation and
  reward-payment paths. These descriptions follow bytecode operations; they
  are not claimed source-level variable names.

Eight additional controls set all these bookkeeping words to zero or maximum
while independently varying the balance word through zero, one, 123 and maximum.
The getter always returns the raw mapping-1 word. Existing `other_mapping_words`
and explicit scalar fields handle the writes; no production mapper change is
needed. The final two-layout native scan passes **1,024 consecutive blocks,
122288006–122289029**, emitting **225 4Stock** and **124 CAP** balances.

The review deliberately limits mapping 36 to four words. Its adjacent membership
flag and the slot-35 address list were not exercised in this window and remain
unsupported. A transaction that writes those fields still fails. This is bounded
runtime/layout qualification, not a claim that every transfer or administrative
path in either contract has been tested.

Four [captured cases](../tests/fixtures/final-proxies/cases.json) retain **11
independent historical RPC expectations**. Rust regressions verify those values,
reproduce failures when reviewed fields are removed, and reject the adjacent
unreviewed membership field. A captured reward-only transaction updates
bookkeeping and pays the separate reward token, while correctly producing no
4Stock balance update. Fixtures preserve the original header and all
transactions relevant to the configured token; their output was checked against
the full block before saving.

## Remaining log-only candidate

Rank 70, `0xe0150e5020e6326448502d0e3f03afd971a85fd8`, remains outside the
99-profile coverage claim. The [creation and activity review](evidence/log-only-creation-review.json)
now identifies why it has reference rows without storage updates:

- Before block **121118665**, canonical RPC returns empty code and nonce zero.
  The captured successful CREATE installs the reviewed 2,264-byte runtime with
  **no storage writes**. Its constructor only copies and returns that runtime.
- The runtime's `balanceOf` reads mapping 0. Its only executable `SSTORE`, at
  PC 856, writes the nested allowance mapping at base 1. The executable runtime
  contains no external call, delegate call, contract creation or self-destruct.
- Selector `a9059cbb` has no dispatched transfer method. It enters a fallback
  that parses packed 51-byte records and emits Transfer logs without changing
  balances. The captured block **122288749** contains **159 nonzero Transfer
  logs**, while independent canonical RPC rechecks return **zero for all 298
  reference participants**.

Two captured Rust regressions preserve the constructor and log-only behavior.
The raw mapping projection emits no rows, leaving a measurable row-coverage gap;
it does not infer state from log amounts or treat all absent writes as zero.
An explicit immutable-zero mapping profile and its event/holder handling remain
follow-up work. The original activity evidence and incomplete coverage are
retained rather than relabeled as full parity.

## Final validation

The actual packaged WASM passes **95,971 historical RPC comparisons**, including
**13,416 zeros**, across all 99 configured tokens and the 1,024-block interval.
There are **zero value mismatches**. The two additions contribute 349 checks.
The WASM binary matches the preceding verified import scan and has no RPC
imports; the package digest changed because it embeds the updated README.

Initialized-holder replay passes **157,123 reference observations**, including
**4,364 carried-forward matches**, with no unknown or incorrect initialized
values. The additions contribute 635 observations and 36 carried-forward
matches. Setup verifies **51,186 stored-holder checkpoints** using **102,372
balance/storage RPC reads**. Processing makes **zero balance RPC calls** and
1,024 header verification calls. Formula/deployment baselines remain separately
counted, and reference observations never repair retained state.

Cold replay still has **56,788 unknown observations**, including **29,933 nonzero
values**, with no incorrect known values. The additions account for 250 unknown
observations, including 36 nonzero values. These are retained coverage limits;
the tests do not establish complete holder enumeration or identical raw output
rows without initialization.

**163 workspace Rust library/binary tests pass**, along with Clippy with warnings
denied, workspace WASM compilation, targeted formatting and diff checks. A Clippy
warning in the new test assertion was corrected and the affected tests and
checks rerun. No production mapper logic changed in this batch.

Results and digests are retained in the [RPC audit](evidence/final-proxy-rpc.json),
[per-token counts](evidence/final-proxy-rpc-token-counts.json),
[holder replay](evidence/final-proxy-holder-coverage.json), and
[artifact identities](evidence/final-proxy-artifacts.json). Local output
directories retain the complete raw captures, RPC checks and holder observations.
The overlapping prior BSC reports must not be summed as independent history.

The [network expansion sequence](network-expansion.md) remains Ethereum, Base,
HyperEVM and Arc after the BSC review. Each needs its own chain identity,
Extended-block qualification, token ranking, runtime review, packaged-WASM audit
and retained-holder replay. No result here qualifies another network or proves
complete global holder enumeration.
