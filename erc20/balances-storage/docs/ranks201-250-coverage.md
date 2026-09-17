# BSC ranks 201–250: 34 additional qualified profiles

The next 50 candidates come from the original immutable RPC ranking at blocks
**122288006–122289029**, inclusive. This batch qualifies **34** of them and adds
them to the prior 199-profile fixture. The
[combined fixture](../tests/fixtures/bsc-ranks201-250-layouts.json) contains
**233 profiles**. Sixteen candidates in this batch and LBP (rank 143) remain
outside the qualified set. This is a bounded interval and observed-holder
campaign, not support for all BSC tokens or a complete holder census.

Production remains one RPC-free `map_events`, shared `evm.balances.v1.Events`,
caller-qualified layouts and default `[]`. These additions use existing rules;
they do not change production logic or package bytes. Tests and diagnostics are
Rust. The [machine-readable summary](evidence/ranks201-250-summary.json) records
every candidate, qualification outcome, source digest and cohort count.

## Qualification and captured tests

The [initial survey](evidence/ranks201-250-survey.json) sampled 50 candidates and
found 48 candidate mappings, with 2,594 independent matching RPC checks. That is
discovery evidence: only one candidate had exact raw event-row parity, and
sample matches alone do not qualify a getter or its other storage fields.

The [33-profile qualification](evidence/ranks201-250-qualification.json) contains:

- 22 historical-runtime-bound source reviews: ranks **201, 202, 206, 211, 213,
  215, 221, 223, 225, 226, 229, 230, 231, 232, 234, 235, 237, 243, 244, 247, 249,
  250**. Reviewed getters return a direct mapping word; ranks 225, 247 and 249
  expose a public mapping. Independent zero, one, 123 and maximum-word controls
  match, as do the complete interval's emitted balances.
- 11 exact reviewed runtime families with freshly checked dependencies: ranks
  **205, 207, 216, 217, 218, 219, 227, 228, 233, 242, 245**. Six use the reviewed
  minimal proxy family, four use the reviewed beacon family, and one is direct.
  Matching a proxy shell is insufficient without matching its implementation.

DDN's daily and monthly fee records each occupy six words. These are explicit,
source-reviewed non-balance records; no unrestricted unknown-write ignore was
added. The captured interval does not contain a witness that fails when the
record width is reduced from six to five, so the fixture makes no such claim.

The [initial qualification failure](evidence/ranks201-250-qualification-initial.json)
is preserved. The review helper initially specified rank 237's `RoleData` as two
words; the source guard rejected it. Its address set occupies two words and its
admin role a third. The corrected profile permits those three explicit words.
Changes to the set's separately stored dynamic contents remain unqualified.

The 33 filtered Extended-block fixtures retain every transaction that writes
the target token. Their output is checked against the complete block before
filtering. The Rust regression checks **84 independently RPC-verified balances**
across all 33 tokens. It does not use the mapper's own results as expected values.

## swkeyDAO2: stored shares divided by a protected factor

Rank **241**, `0x009b797edf9acc666a36020c4509c6095de51408`, uses an otherwise
familiar proxy shell with a different implementation. Reusing the earlier
profile fails with `unqualified proxy implementation`; that failure is retained
in the [division review](evidence/swkey-divisor-review.json).

The pinned implementation is `0x7589b8d477961e9a0995688c18a4974a254dd08e`, with
runtime hash `0x4c6326bbd14525a76a0a2c22d5df9836398d830197b180c8924154dcf3b97cca`.
Verified implementation source was unavailable. Historical bytecode review and
traces identify scalar **9** as the divisor, mapping **10** as raw shares, and
nested mapping **11** as allowances. In the getter, `SLOAD` at PCs 3056 and 3081
reads the divisor and shares; the SafeMath helper reaches unsigned `DIV` at
6106 after rejecting zero. The successful implementation getter has no other
storage, caller, clock or external-contract dependency. The reviewed transparent
proxy's admin exception still applies.

**140 independent state controls** cover five holders, four positive divisors
and seven raw values, including boundaries and uint256 maximum. Every successful
result equals `floor(raw / divisor)`. Five zero-divisor calls revert. Four
non-admin caller controls with a maximum allowance leave balances unchanged.
The allowance getter reads the nested root-11 key at PC 4293; this provides the
basis for accepting those non-balance writes. The initial strict scan's 16
unresolved blocks are retained separately from the corrected scan.

At the parent block, holder `0x124763985d17ce87d1791015f632dc04fd5bc30f` has raw
shares `21866433874190392251570809700755167808344226008734491572399455801909401933200`
but RPC balance `100736701226292376`. All **28 boundary observations** across
14 holders match after division; ten differ from the raw word. The
[qualified layout](evidence/swkey-qualification.json) pins both runtime dependencies
and the divisor at both boundaries, and the complete 1,024-block projection
rejects any intervening protected change. No divisor change occurred.

Existing `balance_divisor` support handles this case without a production code
change. A captured regression matches two independent RPC balances and
reproduces the raw-share mismatch when division is omitted. A mutation of that
captured block confirms a persisted divisor change stops processing and demands
holder reinitialization. This does not qualify rebasing across divisor changes.

## Emitted balances and retained holders

| Evidence over 1,024 blocks | 33 direct/family profiles | swkeyDAO2 |
| --- | ---: | ---: |
| Emitted WASM balances checked against RPC | 1,427 | 32 |
| Emitted zero balances checked | 138 | 0 |
| Initialized observed holders | 863 | 14 |
| Checkpoint balance/storage RPC reads | 1,726 | 28 |
| Initialized reference observations | 2,332 | 64 |
| Matches without a new balance event | 905 | 32 |
| Final retained holders checked against RPC | 863 | 14 |
| Final zero balances | 345 | 9 |
| Cold unknown observations | 865 | 32 |
| Cold unknown nonzero observations | 252 | 0 |

The [33-profile WASM audit](evidence/ranks201-250-rpc.json) and
[swkeyDAO2 WASM audit](evidence/swkey-rpc.json) have zero mismatches. Separate
[33-profile holder](evidence/ranks201-250-wasm-holders.json) and
[swkeyDAO2 holder](evidence/swkey-wasm-holders.json) replays apply those actual
outputs over complete canonical clocks. Reference rows never repair retained
state; processing makes no balance RPC calls. Every final initialized holder
matches canonical historical RPC. Cold unknowns remain unknown, including the
252 nonzero observations; they are not counted as zero balances.

The [combined 233-profile capture](evidence/ranks201-250-combined.json) checks
every protobuf field against the prior 199-profile output and the two new,
disjoint audited cohorts. **232 profiles emit** in this window; hLBP remains
quiet and retains its separate older mint evidence. The combined stream preserves
**105,106 previously verified emitted balances**, including **14,711 zero checks**,
and the same event histories supporting **172,250 initialized observations** and
**67,145 initialized carry-forward matches**. This check binds evidence digests,
layouts, the unchanged package and fresh canonical headers. It does not repeat
those balance RPC calls or create a fresh full-233 checkpoint/final snapshot.

All **223 workspace Rust library/binary tests** pass, together with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
Repacking the unchanged production package preserves the audited artifacts:

- SPKG: `d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`.
- WASM: `861a879353a7fc7163a26c80f671fc3011e9ca3ee32a65dfc0badd99ac9794b6`.

## Remaining candidates and preserved failures

The [mismatch diagnostic](evidence/ranks201-250-mismatches.json) preserves both
raw-mapping counterexamples. YBC (rank **238**) adds `staticReward` from
`calculateReward(address)` on another contract to its direct mapping balance.
Of 24 boundary observations for 12 holders, six differ from the raw word; all
24 equal raw balance plus the independently returned static reward. Unlike the
separate LBP counterexample, this particular YBC sample has no observed public
balance change without a candidate-word write. No YBC production layout is
qualified.

YBC's reward dependency is `0xe8058da568eb94194588bc9bae53a5e584fe7471`, pinned
at both boundaries to runtime hash
`0xb3d6938e8c9477247d16e146b97c511d1c0c232b3b76394f204a31d239b38c1f`.
Its source is unavailable. The detailed trace fails with RPC code `-32008`,
`Response is too big`, exceeding the 167,772,160-byte provider limit. The
[first incomplete diagnostic](evidence/ranks201-250-mismatch-initial.json) and
later explicit trace failure are retained. No detailed YBC storage trace or
complete reward model is claimed.

Later follow-up: [compact YBC tracing](ybc-reward-trace.md) recovers the execution
and independently checks its storage reads. It also corrects the old
`dynamic_reward` label: the second reward tuple word is a stopping hour. The
original response-limit failure above remains historical evidence; YBC is still
unqualified because its complete reward model and holder-output behavior remain
unresolved.

Ranks **203, 204, 208, 209, 210, 212, 214, 220, 222, 224, 236, 238, 239, 240,
246, 248** remain unqualified. Next reviews include voting histories, holder
arrays, new proxy dependencies and source-unavailable getters. LBP (143) remains
outside production despite its separate host-only reward model. After BSC,
the requested sequence is [Ethereum, Base, HyperEVM and Arc](network-expansion.md),
with separate runtime, Extended-block and holder evidence for each network.
