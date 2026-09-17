# LBP reward-state diagnostic and regression

LBP remains **unqualified for the production mapper**. A host-only Rust model
now explains all 33 known counterexamples from the
[holder mismatch investigation](lbp-reward-mismatch.md). It reproduces the
runtime-bound integer reward calculation using initialized storage, persisted
updates and block time. No production map, store, layout or protobuf changes
are included in this work.

## Historical evidence

The [manifest](../tests/fixtures/lbp-rewards/manifest.json) binds the original
RPC capture and the new offline fixture by SHA-256. Both LBP and its hLBP reward
dependency were checked against their verified source runtime at the initial
and final blocks. The fixture builder rechecked the initial runtime and final
canonical header. The original diagnostic also checked canonical headers at
every sampled height. The collector rejects persisted code changes to either
contract and enforces old/new continuity for each tracked storage word.

| Measurement | Result |
| --- | ---: |
| Initial checkpoint block | 122288005 |
| Replay interval, inclusive | 122288006–122289029 |
| Known counterexample holders | 33 |
| Initial storage reads | 171 |
| Captured tracked storage updates | 110 |
| Raw holder balance updates | 0 |
| Computed block/holder observations in the original diagnostic | 33,792 |
| Independently RPC-validated observations | 594 |
| Validation calls (`balanceOf` plus `pendingRewards`) | 1,188 |
| Balance or pending-reward mismatches in those samples | 0 |
| RPC reads used to advance replay state | 0 |

The 594 observations cover the initial checkpoint, every 64th block starting
at the first replay block, and the final block: 18 heights × 33 holders.
**The other computed observations were not independently RPC-validated.**
These observations are separate from the 199-profile production audit totals.

The offline test applies all 110 persisted updates across the complete 1,024
block clock sequence, then checks each of the 594 captured RPC observations.
The compact replay journal retains each source Extended block's hash, parent,
timestamp and file digest. It is derived from `persist::collect_block`; it is
not a fresh packaged-WASM reward replay. RPC expectations never enter or
repair the model's state. Negative controls prove that dropping global writes
or freezing the preview clock produces mismatches. The earlier regression
that detects 33 stale raw-balance holders remains in place.

## Independent arithmetic controls

[Thirty-four simulated-state cases](../tests/fixtures/lbp-rewards/controls.json)
make 68 read-only `eth_call` requests against the same historical contract
runtimes at canonical block 122288005. State overrides affect only each call's
simulation; no transactions are submitted. Each stored expectation comes from
RPC, including five expected Solidity arithmetic-panic responses.

The cases cover:

- Closed trading, unchanged/future update timestamps, an empty participant
  tree, static rewards only, node rewards only, and exhausted/limited cap debt.
- Days zero and one; immediately before, at and after a day boundary;
  multiple-day integration; day 10,000, day 10,001 and the cutoff crossing.
- User indices above live accumulators, zero-share non-nodes, zero/dead
  addresses, and all eight runtime-resolved LBP exemptions.
- Overflow of the static accumulator, holder multiplication, and raw-plus-
  pending balance addition. Exemptions and reward-free early exits skip
  otherwise invalid arithmetic, matching the contracts.

The decay calculation ports the verified source's PRBMath fixed-point
logarithm/exponent path. Repeated multiplication or floating-point exponentials
would change rounding. The adjacent-day decay and division order also follow
the deployed source. The [PRBMath MIT notice](../tools/src/lbp_rewards/PRBMath-LICENSE)
is retained with the port; [upstream license](https://github.com/PaulRBerg/prb-math/blob/main/LICENSE.md).

## What production support still needs

This is a bounded arithmetic and replay diagnostic, not a general reward-token
implementation. Production support still needs a durable, explicitly
initialized registry of holders and their reward fields, retained global
dependency state, historical runtime/exemption binding, and deterministic
updates for holders affected only by a clock or global change. New holders,
checkpoints, restarts and chain rewinds must preserve those guarantees.

A possible extension is a Substreams store feeding the existing `map_events`.
That would retain one map but add persistent state and requires a design
decision. No such store has been added. Without persistence, this token must
remain explicitly unsupported; treating missing state as zero or emitting only
raw mapping writes is incorrect.

The model does not enumerate all holders, simulate claims or transactions,
prove all future runtime versions, or establish production output parity for
LBP. The qualified fixture remains at 199 profiles and excludes LBP.

Run the captured regressions without RPC or credentials:

```sh
cargo test --locked -p erc20-balances-storage-tools --lib lbp_rewards
```
