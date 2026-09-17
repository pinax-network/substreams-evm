# BSC ranks 251–300: 33 more qualified layouts

The subsequent [17-candidate follow-up](pending300-coverage.md) completes this
cohort's bounded qualification and adds captured DIA/ETZ holder-list activity.
This report retains the original 33-token measurements and pending status.

The explicit test set now contains **279 qualified profiles**, with 278 emitting
in the original 1,024-block window. This adds 33 contracts from ranks 251–300;
17 in this cohort still need review. The production interface remains one
RPC-free `map_events`, the shared `evm.balances.v1.Events` output and default
parameters `[]`. No production token list or extra map/store was added.

## Selection and qualification

Candidates come from the original immutable RPC ranking, not a new capture:
blocks **122288006–122289029** inclusive, reference digest
`ebfff9a998618b3e7e9f7802d20d49c9890f7295de189315393e636a65fc88d9`.
The 50 candidates account for 2,593 reference observations. The exploratory
survey sampled 316 unique active blocks and checked 2,668 values without RPC
errors or value mismatches. It still reports `coverage_gap`: matching values
do not qualify unknown storage writes or supply unchanged-holder rows.

[Qualification evidence](evidence/ranks251-300-qualification.json) records:

- 18 direct/public mapping getters reviewed in verified source bound to the
  historical runtime, including their scalar, allowance and multiword records.
- 15 previously reviewed runtime families, with fresh holder controls and
  historical dependency checks. These include two upgradeable proxies and
  ten minimal proxies; shell similarity alone does not qualify a dependency.
- Zero, one, 123 and maximal uint256 word controls for every added contract.
- A complete native scan of all 1,024 Extended blocks with no unresolved writes.

A separate compact-inspector pass covers all 50 candidates, including those
still pending. It binds attributed storage/call instruction positions to each
historical runtime and records the canonical probe, trace digest and all four
controls in the candidate summary. All controls match the sampled raw word;
none automatically promote the remaining 17 profiles.

The added ranks are **251, 252, 255, 256, 257, 259, 260, 261, 262, 263, 264,
268, 269, 270, 271, 274, 275, 276, 277, 279, 280, 281, 282, 283, 284, 285,
286, 291, 294, 295, 296, 297 and 299**. Their explicit configuration is
[bsc-ranks251-300-layouts.json](../tests/fixtures/bsc-ranks251-300-layouts.json).
Bookkeeping rules do not make arbitrary future writes acceptable; unreviewed
writes or protected runtime/dependency changes still reject processing.

## Actual packaged output and holders

The [fresh 33-token packaged-WASM audit](evidence/ranks251-300-rpc.json) checks
**1,128 emitted balances**, including **125 zeros**, against canonical
historical RPC. All match. Complete block-clock evidence covers the 737 empty
output blocks as well as blocks with events.

The [holder replay](evidence/ranks251-300-wasm-holders.json) initializes **736
observed holders** at the canonical parent using **1,472 balance/storage reads**.
It then consumes the actual packaged events without processing balance RPC
calls or inserting reference values into retained state:

- **1,724 reference observations match**, including **596** with no new event.
- All **736 final holder balances** match a fresh RPC snapshot; **189 are zero**.
- Cold state still has **581 unknown observations**, **297 nonzero**. Unknown
  history is preserved as unknown rather than treated as zero or repaired from
  the reference stream.

The [combined 279-profile capture](evidence/ranks251-300-combined.json) matches
every protobuf event field against the earlier 246-profile capture plus the
new audited events. Its **106,749 emitted balances**, including **14,871 zeros**,
reuse immutable, independently verified RPC evidence after fresh canonical
header and runtime checks. The combined comparison makes no new balance RPC
calls and does not create a full-279 checkpoint or final holder snapshot.
Identical event histories preserve evidence for **174,900 initialized
observations** and **68,152 matches without a new event**. hLBP remains quiet
in this interval and has separately documented older activity.

The unchanged production SPKG digest is
`f13ce00a08e3610ed607a168c1f2afac6fb92ac63561f945e97a55d7b4801503`.
Thirty-three captured transaction fixtures retain **123 independently checked
balances** for Rust regression tests. The tests bind block identities and compare
all captured projected balances with the RPC expectations.

Validation passes **243 workspace Rust library/binary tests**, Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.

## Remaining coverage

[Candidate status and provenance](evidence/ranks251-300-summary.json) retain all
50 candidates. Ranks **253, 254, 258, 265, 266, 267, 272, 273, 278, 287, 288,
289, 290, 292, 293, 298 and 300** remain outside this fixture. Their successful
exploratory word controls do not replace review of getter implementations,
proxy dependencies and non-balance storage.

LBP (143), TITAN (203), ORD (209) and YBC (238) also remain unqualified.
[YBC's recovered reward trace](ybc-reward-trace.md) now resolves the earlier
provider response-size obstacle and corrects a mislabeled tuple field. It does
not yet provide a complete reward model or affected-holder emission rule.

These are bounded per-contract tests, not global holder enumeration or universal
BSC support. The next network sequence remains [Ethereum, Base, HyperEVM and
Arc](network-expansion.md), each with independent chain, runtime, Extended-block
and holder evidence.
