# BabyDoge and 10SET reflection balances

BabyDoge (rank 372) and 10SET (rank 377) have source-bound reflection getters.
An excluded holder reads `_tOwned` directly; other holders divide `_rOwned`
by a rate derived from global supplies and the complete ordered exclusion list.
A matching direct-mapping sample on an excluded holder does not establish the
getter used by ordinary holders. Neither token is added to the production
layout fixture by this work.

The Rust [host model](../tools/src/reflection.rs) implements that reviewed
calculation with integer division, the deployed supply-fallback order and
expected reverts. Inputs are independently initialized storage words. Missing
global state, array elements or member balances are errors, never assumed zero.
Conflicting snapshots of the same account are rejected. Excluded holders bypass
the global calculation, including invalid or unavailable global supplies.

## Historical evidence

The [historical report](evidence/reflection-historical.json) binds verified
source bytes to the actual runtime at every sampled canonical block. It covers
the original BSC window, with parent checkpoint 122288005 and final block
122289029. The holder set is the addresses observed in the original RPC stream.

| Token | Observed holders | Snapshots | Storage reads | Fresh getter checks | Excluded / ordinary checks | Matches in original reference |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| BabyDoge | 19 | 11 | 1,122 | 209 | 22 / 187 | 34 |
| 10SET | 12 | 10 | 430 | 120 | 0 / 120 | 33 |

All **329 independent getter checks** match. Treating the token-word mapping
as the balance would disagree in **177** of those observations. BabyDoge has
12 excluded entries and 10SET has one throughout these snapshots; all entries
and their reflected/token words are initialized independently.

## Retained state and passive holder changes

The [host replay](evidence/reflection-replay.json) starts from those parent
checkpoints and applies persisted storage changes across **1,024 consecutive
Extended blocks**. Block hashes form a continuous chain and the endpoints are
bound to fresh canonical headers. Reverted writes are excluded, known old words
must be continuous, and persisted token-code changes are rejected.

It matches **298 subsequent initialized observations** and **1,407 independently
read storage words**, with **zero balance RPC calls during processing** and no
reference values inserted into state. The replay consumes 32 persisted
BabyDoge writes and 54 10SET writes. This is a host diagnostic consuming raw
Extended storage changes, not the production `map_events` output.

The replay finds **20 10SET holder/block balance changes with no write to that
holder's reflected balance, token balance or exclusion flag**. Twelve such
transitions receive **24 fresh RPC checks**, one on each adjacent block boundary.
Every check matches. BabyDoge has no such transition in this interval.

This explains why retaining only the latest emitted direct balance is
insufficient for reflection holders: the global rate can change their amounts
while their own storage stays constant. Production support still needs complete
dependency initialization, affected-holder output, and restart/rewind handling.
The bounded 31-holder checkpoint is not a complete global holder enumeration.

## Controlled paths and regressions

The [read-only controls](evidence/reflection-controls.json) exercise 24 cases
on each original deployed runtime: **48 matching outcomes**, including **10
expected reverts**. They cover ordinary/excluded branches, low-byte boolean
decoding, global-rate changes, integer truncation, ordered exclusion subtraction,
duplicate entries, underflow fallbacks, empty supplies, uint256 boundaries and
the early return for excluded holders. They modify simulated storage only;
neither deployed code is replaced and no transaction is sent.

Captured [Rust tests](../tools/src/reflection/tests.rs) replay the independent
historical inputs, controlled outcomes and 12 passive transitions. They also
reject missing or inconsistent initialization. These model tests do not claim
packaged-WASM reflection support.

Initial diagnostic runs are retained locally: one rejected a case-sensitive
source-address comparison before reading token state, and two completed their
comparisons but returned a failing process status because their custom report
status was not recognized by the run wrapper. Canonical address normalization
and the existing success status resolved those harness issues; fresh final
runs passed. No balance mismatch was discarded by those corrections.
