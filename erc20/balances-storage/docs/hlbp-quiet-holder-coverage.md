# hLBP holder coverage without balance changes

The later [captured mint audit](hlbp-active-holder-coverage.md) qualifies hLBP
and expands the fixture to 199 profiles. This report preserves the earlier quiet
audit's original scope, counts and qualification status.

Rank 189, **hLBP** (`0x5e3cbc82d020be91a989eb747934104e9ab585fe`), now has
independent getter and initialized-holder evidence. It remains outside the
**188 active qualified profiles** because no actual changed-balance case has
been captured. This follow-up does not resolve
[LBP's external pending-reward mismatch](lbp-reward-mismatch.md).

## Getter and bookkeeping

The [getter qualification](evidence/hlbp-quiet-qualification.json) binds verified
source to historical runtime
`0xe65e2ea2fa1e0264e23eec7dcabed2ba497ca87f918a7c141ecc8313fe794662`.
Its sole implemented `balanceOf` returns `_balances[account]` at mapping 0.
There is no pending-reward term in this getter. Independent call tracing reads
one storage word, and read-only overrides of zero, one, 123 and uint256 maximum
return exactly those values.

The original discovery scan found changes under mapping 9, which source
identifies as `userIndex`, not balances. The explicit reviewed layout treats
this and the other declared bookkeeping mappings separately. Scalar roots are
2, 3, 4, 6, 7, 8, 12 and 17; non-balance mappings are rooted at 1, 5, 9, 10, 11,
13, 14, 15 and 16. No arbitrary storage ranges or inferred layouts are enabled.

All 1,024 complete Extended blocks in **122288006–122289029** validate with
**zero emitted balance rows**. A broader search sampled the largest reference
holder at 33 historical heights, 65,536 blocks apart, and found no different
balance. That sampling does not rule out changes between samples. The search
did not produce a changed-balance regression, so it did not promote the profile.
The original search report and unsuccessful qualification-wrapper result remain
preserved; the holder audits below are independent runs.

## Independent holder evidence

The [native audit](evidence/hlbp-quiet-native-holders.json) initializes four
reference holders with **eight explicit canonical RPC balance/storage reads**.
All **88 reference observations** match. A cold consumer has **88 unknown
observations**, including **22 nonzero values**, because it receives no balance
events during this interval. An absent update cannot establish a zero balance.

The [actual packaged-WASM replay](evidence/hlbp-quiet-wasm-holders.json) independently
confirms empty output for **all 1,024 blocks** using complete canonical clocks.
Initialized state matches all 88 historical reference observations without a
new event. The final canonical RPC snapshot checks all four retained holders:
**three zeros and one nonzero balance**, with no mismatches. Reference values
never repair state, and replay makes no balance RPC calls.

This reuses the original historical RPC reference capture; it is not 88 new
RPC balance queries. Checkpoint and final-snapshot reads are separate. The
actual package digest remains
`d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`.

The captured Rust regression at block **122288019** verifies that reviewed
bookkeeping produces no balance event and rejects the same block when required
non-balance mapping qualification is removed. It also preserves the difference
between initialized matches and cold unknowns, including the nonzero holder.
The complete campaign audits bind the earlier checkpoint to the intervening
blocks; the small fixture is not claimed as a full-history replay.

Production code, package bytes, the qualified-profile count and the single
RPC-free `map_events` interface remain unchanged. This is useful holder evidence
for a quiet token, not emitted-row parity or global holder enumeration.

All **211 workspace Rust library/binary tests pass**, along with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
