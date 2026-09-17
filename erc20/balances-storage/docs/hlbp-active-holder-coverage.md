# hLBP mint and retained-holder qualification

hLBP, rank **189** (`0x5e3cbc82d020be91a989eb747934104e9ab585fe`), now has a
captured balance change in addition to its prior quiet-holder evidence. The
[combined fixture](../tests/fixtures/bsc-hlbp-layouts.json) contains **199 qualified
profiles** from the reviewed top 200. Reward-bearing LBP (143) remains excluded.
This does not resolve [LBP's pending-reward mismatch](lbp-reward-mismatch.md).

The same source-qualified hLBP layout works unchanged. Production remains one
RPC-free `map_events`, shared `evm.balances.v1.Events`, explicit caller-qualified
layouts and default `[]`. This addition changes test profiles, fixtures and
evidence; it does not change ingestion logic or package bytes.

## Captured mint and source binding

The earlier search sampled only the preceding 2,097,152 blocks. A wider
historical search found adjacent differing balances at **104727183–104727184**.
Its binary partition preserved observations on both sides; it does not claim
the earliest deployment or earliest balance change.

The [new qualification evidence](evidence/hlbp-active-qualification.json) binds
the same verified source to runtime
`0xe65e2ea2fa1e0264e23eec7dcabed2ba497ca87f918a7c141ecc8313fe794662`
at the changed-balance block. The implemented `balanceOf` returns the raw
`_balances[account]` mapping word at root zero. Fresh overrides of zero, one,
123 and uint256 maximum return exactly those words. Historical runtime checks
also pass at both ends of the new interval.

Transaction
`0x55856d9fda4c5be5193561c7d775e823c3d6e499da44aab9da963daf61c50b0c`
mints to `0x00e3ea08fd8cbad955ec5d2292ad637670c31524` at block **104727184**.
Canonical RPC balances change from **0** to
**10737390507980558866879696** base units. The captured non-reverted `Transfer`
from the zero address has that exact amount. The Extended block's storage
projection emits the same single balance.

The Rust fixture retains complete transactions that write the token and verifies
their output equals the full captured block. It matches the independent RPC
amount and reproduces rejection when the reviewed `registeredLp` bookkeeping
mapping at root five is removed. The [earlier quiet regression](hlbp-quiet-holder-coverage.md)
still covers no-output bookkeeping and the difference between initialized
balances and cold unknowns.

## Separate active and quiet intervals

| Evidence | Changed-balance interval | Original quiet interval |
| --- | --- | --- |
| Blocks, inclusive | 104727168–104727231 | 122288006–122289029 |
| Consecutive blocks | 64 | 1,024 |
| WASM balance rows | 1 | 0 |
| Canonically confirmed empty blocks | 63 | 1,024 |
| Initialized reference observations | 4 | 88 |
| Initialized matches without a new event | 3 | 88 |
| Final retained holders checked | 4 | 4 |
| Final zero balances | 3 | 3 |

The [active WASM audit](evidence/hlbp-active-rpc.json) matches its sole emitted
balance to historical RPC, with zero mismatches. The
[native holder replay](evidence/hlbp-active-holder-coverage.json) uses **eight
explicit checkpoint balance/storage reads** for four observed holders. All four
reference observations match. Cold state has three unknown zero-value
observations; the mint teaches it only the changed holder's balance.

The [independent active WASM replay](evidence/hlbp-active-wasm-holders.json)
matches all four observations and all four final canonical RPC balances.
Reference rows never repair retained state and processing performs no balance
RPC calls. The complete 64-block capture binds the checkpoint, intervening
updates and final snapshot. The small unit-test fixture is not presented as a
full-history replay.

These are two distinct intervals and checkpoints. Their four-holder sets are
not added into a claim of eight unique holders, and the active interval is not
folded into the original 1,024-block aggregate. A captured mint does not establish
all possible transfer, burn, referral or global-holder paths.

## Combined 199-profile regression

The [new combined WASM capture](evidence/hlbp-combined.json) runs all 199 profiles
over the original 1,024-block interval. **198 profiles emit**; hLBP remains quiet
there. Every protobuf event field matches the prior 198-profile capture, with
**103,647 previously RPC-verified balances** and hLBP's canonically confirmed
empty output. Artifact digests, layouts, package identity and fresh canonical
headers are checked. This comparison performs no repeated balance RPC calls.

The combined event histories preserve evidence for **169,854 initialized holder
observations** and **66,208 initialized carry-forward matches** in that interval.
The count now includes hLBP's 88 quiet observations, previously reported
separately. It excludes the four observations from the older active interval.
The **14,573 emitted zero checks** are unchanged. There is no new full-199 holder
checkpoint, final-holder snapshot or claim of global enumeration.

All **216 workspace Rust library/binary tests pass**, together with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
Repacking preserves the exact audited artifacts:

- SPKG: `d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`.
- WASM: `861a879353a7fc7163a26c80f671fc3011e9ca3ee32a65dfc0badd99ac9794b6`.

LBP remains the unresolved candidate from this top-200 review: public balances
depend on changing external rewards even when a holder's raw balance word never
changes. Its captured counterexamples remain negative tests. The subsequent
network sequence remains [Ethereum, Base, HyperEVM and Arc](network-expansion.md),
with separate qualification and holder evidence for each network.
