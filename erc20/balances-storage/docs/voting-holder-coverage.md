# Voting-token holder coverage

The [89-profile fixture](../tests/fixtures/bsc-voting-layouts.json) adds SENTIS
(rank 71, `0x8fd0d741e09a98e82256c63f25f90301ea71a83e`) and STAR
(rank 93, `0x8fce7206e3043dd360f115afa956ee31b90b787c`) to the preceding 87.
Their verified `balanceOf` implementations return ordinary balance mappings,
at slots 5 and 0 respectively. Voting history updates do not alter this getter.

[Qualification evidence](evidence/voting-qualification.json) binds verified
source to the historical runtime, records compiler storage layouts, getter
traces and deployment identities, and independently checks `clock()` at
deployment and both ranked-window boundaries. [Sixteen read-only RPC controls](evidence/voting-getter-controls.json)
include zero, one, a nontrivial value and the full uint256 maximum, with and
without modified voting-history lengths. All return the configured balance word.

## The uncovered write and its fix

Both tokens passed the ordinary-transfer window after configuring their scalar
and mapping fields. However, SENTIS's constructor mint at **51,995,162** and
STAR's later mint at **58,597,372** stopped with `unresolved storage`: the mapper
did not understand their dynamic voting checkpoint arrays. Those original
errors remain in the qualification evidence and captured Rust cases. This was
an extraction failure, not a sampled balance-value disagreement.

The new optional `voting_checkpoints` layout describes reviewed OpenZeppelin
`Trace208` histories. SENTIS uses block numbers, with total checkpoints at slot
15 and delegate checkpoint mappings at slot 14. STAR uses timestamps, at slots
10 and 9. These fields remain separate from balances and ordinary ignored slots.

For an append, the mapper requires a persisted length increase of exactly one,
the corresponding final array element, zero initial contents, and correct
execution order. It validates packed uint48 clock values and old/new continuity.
Only one element per array can change for a given clock. A timestamp history may
instead overwrite the last entry across consecutive blocks in the same second;
its old and new clock must both match the block timestamp. Array indices are
bounded by the number of distinct clock keys that the reviewed insertion rule
can have produced. This is a semantic constraint from the pinned code, not an
arbitrary configured range of storage to ignore.

Unknown writes, missing append witnesses, malformed lengths, wrong clocks,
discontinuous writes, and reverted changes cannot authorize an array element.
The configuration rejects roots shared with balances or other protected fields.
The production module remains one RPC-free `map_events`, with the shared
`evm.balances.v1.Events` and default parameters `[]`.

The real historical mint fixtures exercise total-checkpoint arrays. No delegate
vote events were found in the separate 100,000-block scan ending at 122289029
for [SENTIS](evidence/voting-sentis-recent-activity.json) or
[STAR](evidence/voting-star-recent-activity.json).
Delegate arrays and the same-timestamp overwrite branch are verified against
the source and covered by Rust scenarios; live captures of those branches are
not claimed.

## Empty block delivery in the audit tool

The first three-block STAR WASM audit was retained as
[incomplete](evidence/voting-star-mint-incomplete.json). Substreams successfully
delivered all three blocks, but its JSONL renderer omitted two empty Events
messages, and the old validator required a JSON row for every block.

The Rust capture tool now requires the original delivery count to equal the
requested range. For sparse output it captures clocks for the same package and
parameters, checks every block number and canonical hash, and verifies parent
continuity before representing absent Events as empty lists. The original sparse
capture and all digests remain available. Gaps, duplicates, partial blocks,
wrong hashes, forks and missing delivery receipts fail. Empty lists never create
zero holder balances. Standalone incomplete JSONL files still fail validation.

The final STAR audit checks all three blocks, including the two confirmed empty
outputs. Its [clocks](evidence/voting-star-mint-clocks.txt) and
[normalized Events](evidence/voting-star-mint-events.jsonl) are retained.

## Holder replay and validation

The [initialized-holder replay](evidence/voting-holder-coverage.json) covers
**1,024 consecutive blocks, 122288006–122289029**. All **154,131 reference
observations** match, including **4,315 carried-forward matches**, with no
unknown or incorrect initialized balances. SENTIS contributes 291 observations
and STAR 224; their additions include seven carried-forward matches.

Setup independently verifies **50,474 stored-holder checkpoints** with
**100,948 balance/storage RPC reads**. Existing formula and validated deployment
baselines remain separately counted. Processing uses zero balance RPC calls and
1,024 header verification calls. Reference results never repair candidate state.

Cold replay retains **55,578 unknown observations**, including **29,551 nonzero
values**, with no incorrect known values. The two additions account for 198
unknown observations, including 136 nonzero values. A verified checkpoint or
full history is still required; these observed holders do not represent every
holder globally.

Seven [captured Rust cases](../tests/fixtures/voting/cases.json) retain seven
independent emitted-balance expectations and three additional STAR holder reads.
The initial mint amounts sum to independent RPC total supply: **1e27 integer
units** for SENTIS and **1e18** for STAR's sampled mint. STAR's supply transition
was located by bisection between zero and positive observations; this establishes
the recorded adjacent transition, not global monotonicity or the first-ever
mint. A Rust replay preserves that holder's balance through empty output before
and after the mint, using a measured pre-mint checkpoint.

The final packaged WASM matches all **94,238 emitted balances**, including
**13,223 zeros**, across all 89 tokens in the ranked window. The two additions
contribute 310 checks. Both separate historical mint audits also match RPC;
the STAR audit accounts for all three blocks and confirms two empty outputs.

**154 workspace Rust library/binary tests pass**, along with Clippy with warnings
denied, workspace WASM compilation and formatting/diff checks. The
[WASM import scan](evidence/voting-wasm-imports.json) permits only the four
reviewed output/logging/panic/empty-output imports and finds no RPC imports.

The initial full WASM audit [stopped on an RPC transport failure](evidence/voting-rpc-incomplete.json)
after 62,003 checks, with no mismatches. It remains incomplete. Final audit results,
per-token coverage and exact artifact digests are recorded in
[the RPC report](evidence/voting-rpc.json),
[token counts](evidence/voting-rpc-token-counts.json),
[SENTIS mint audit](evidence/voting-sentis-mint-rpc.json),
[STAR mint audit](evidence/voting-star-mint-rpc.json), and
[artifact identities](evidence/voting-artifacts.json).

Eleven of the original top 100 candidates remain unqualified: ranks **55, 56,
62, 67, 69, 70, 72, 79, 85, 89 and 92**. The remaining work includes runtime
semantics and non-balance field review where verified source is unavailable,
and stronger activity evidence for the log-only zero-balance candidate. These
overlapping windows must not be added together as independent history.
