# YBC reward arithmetic from storage

YBC remains outside the production layouts. A host-only Rust model now explains
all six known raw-word mismatches and matches both `calculateReward` and
`balanceOf` for 24 independently initialized historical snapshots. This resolves
the sampled arithmetic discrepancy; it does not implement production reward
state or establish complete holder coverage.

The [evidence manifest](evidence/ybc-reward-model.json) binds the historical
inputs, expectations, reviewed runtimes and read-only controls. The
[model](../tools/src/ybc_rewards.rs) uses checked uint256 arithmetic and explicit
state inputs. It makes no RPC calls. Captured expectations live in
[`tests/fixtures/ybc-rewards`](../tests/fixtures/ybc-rewards).

## Reviewed calculation

The token's verified source establishes that `balanceOf` adds the first word
returned by its external reward dependency to the raw balance. The second word
is the stopping hour, as corrected in the [earlier trace report](ybc-reward-trace.md).
The dependency's source remains unavailable: two metadata gateway requests
returned HTTP 429 and a third timed out. Those failures are retained. Its model
comes from historical runtime bytecode and the recovered execution, followed by
fresh controls against the deployed contracts.

The modeled path performs these operations in the deployed order:

1. Return zero reward and stopping hour when the user's static rate is below
   `1,000,000,000`. Otherwise calculate the current hour from launch time and
   return early for an unopened or already claimed cycle.
2. Visit at most 240 consecutive hours from the saved last cycle. A no-burn
   hour skips rate updates and rewards. Other hours carry forward nonzero total
   and user rates, preserving integer division at each hour.
3. When a recorded hourly reward is zero and the total rate is nonzero, preview
   the burn against the remaining pool balance. The daily parameter is divided
   by 24 before multiplication; two thirds of the burn form the reward.
4. Convert accumulated reward through the pool reserves, apply the user's
   `wbnbValue * 13 / 10` cap after previous LP rewards, and convert the remaining
   cap back to token units when necessary. A positive conversion requires
   nonzero reserves; a zero amount returns zero before that check.
5. Add the resulting reward to the raw balance with overflow checks.

Missing initialized hours, fields or burn flags are rejected. Explicit initial
rates must equal their first hourly storage words. The cap-equals-claimed case
returns zero reward but preserves the stopping hour; cap-below-claimed returns
two zero words. Neither case can be replaced by a generic zero-reward shortcut.

## Historical observations and controls

At canonical blocks **122288005** and **122289029**, fresh storage reads initialize
12 previously sampled holders at each boundary. The capture reads **3,410 storage
words** and makes **48 independent getter calls**. All 24 reward tuples and all
24 balances match. Six balances differ from their raw words and are explained
by the calculated reward. RPC expectations never initialize model state.

At block 122288005, **30 read-only state-override cases** compare both getters,
for **60 calls**. All outcomes match, including **17 expected reverts**: 15
arithmetic panics and two insufficient-liquidity errors. Controls cover hourly
rate carry, skipped burns, missing recorded rewards, pool depletion, rounding,
caps, reserve orientation, the 240-hour limit, early exits and overflow. The
captured tests verify revert payloads as well as successful values. No
transaction was submitted.

The token, reward dependency and pool runtimes are bound to each historical
capture. The pool's own reward path is excluded from these initialized snapshots
by checking its static rate is below threshold; its public balance therefore
equals its raw word in these cases. A pool with its own nonzero reward requires
additional modeling and evidence.

Run the offline captured regressions from the repository root:

```sh
cargo test --locked -p erc20-balances-storage-tools --lib ybc_rewards
```

## Remaining production work

This is a two-boundary snapshot model, not a consecutive-block reward replay or
a packaged-WASM qualification. Durable initialization, dependency-change guards,
affected-holder discovery when only time or shared state changes, restarts and
rewinds remain unresolved. Recursive pool rewards and untested runtime branches
also remain outside the evidence. No token is promoted, and no production map,
store, protobuf or default parameter changes are introduced.
