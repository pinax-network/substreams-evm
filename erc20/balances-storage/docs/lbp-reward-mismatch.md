# LBP: pending rewards change balances without holder writes

Rank 143, `0x88886f0fd371dff856291badced45922bc888888`, remains **unqualified**.
Its runtime-bound source implements `balanceOf(holder)` as a raw mapping word
plus `hashrate.pendingRewards(holder)`, except for explicit protocol addresses.
The dependency is `0x5e3cbc82d020be91a989eb747934104e9ab585fe`.

## Why the first samples missed the mismatch

The eight addresses observed in the original window all returned raw balances.
The main holder with positive hashrate shares,
`0x00e3ea08fd8cbad955ec5d2292ad637670c31524`, is the pool and is exempt from
the pending-reward branch. Address-shaped preimages, logs and callers in that
window did not expand the set beyond eight addresses.

[Global state checks](evidence/lbp-global-review.json) show trading is open,
32 nodes exist, and both reward accumulators are positive and change during the
window. The hypothesis that rewards are globally disabled or zero is false.
The two failed getter probes in that diagnostic target internal fields without
public ABI getters; they are retained and are not treated as zero values.

## Historical-holder counterexamples

[Transfer-log discovery](evidence/lbp-historical-holders.json) over the preceding
100,000 blocks, **122188005–122288004**, finds 48 candidate holders.
Historical `balanceOf`, raw storage and `pendingRewards` calls at canonical
block **122288005** show **33 raw/RPC mismatches**. These non-exempt holders'
public balances equal raw storage plus pending rewards.

The [retained-holder check](evidence/lbp-retained-drift.json) examines their raw
balance keys across every captured block from **122288006 through 122289029**,
then checks their final canonical RPC balances. All **33 holders have no
persisted balance-word writes and unchanged raw words**, yet their public
balances change. A correct initial RPC checkpoint followed by balance writes
alone therefore becomes stale.

For holder `0x000bc2d60dd3153696832b672dfa58ca24877c89`, in raw integer units:

| Value | Block 122288005 | Block 122289029 |
| --- | ---: | ---: |
| Stored balance | 45,138,408,565,149 | 45,138,408,565,149 |
| Pending rewards | 159,231,065,713,174,153 | 160,971,372,459,665,186 |
| RPC balance | 159,276,204,121,739,302 | 161,016,510,868,230,335 |
| Holder balance writes during the window | — | 0 |

A captured Rust regression initializes all 33 holders with their correct RPC
balances, replays the complete canonical clock interval without holder balance
writes, and verifies the audit reports 33 value mismatches at the final block.
Reference observations do not repair state or turn the outcome into parity.

## Remaining implementation work

This behavior cannot be represented by the current raw-mapping or pinned
constant/divisor rules. The source's reward calculation depends on holder
hashrate, node status, user indices, changing global accumulators, emission
state and block time. Qualification must account for those dependencies and
updates to untouched holders; adding storage ignore rules or emitting raw
words would conceal incorrect balances.

No LBP profile is included in the 149 qualified layouts. These counterexamples
are separate from their passing audits. They establish a concrete remaining
gap; they do not establish complete holder enumeration, full reward-model
support or completion of the BSC work.
