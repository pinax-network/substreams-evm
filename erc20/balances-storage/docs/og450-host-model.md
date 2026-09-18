# OG host model: five storage mismatches explained

The host-only Rust model reproduces all five sampled OG mismatches, including
both helper return values and the hourly stopping cursor. It now matches all
43 original read-only RPC controls and 49 fresh preview controls. The three
branches rejected by the initial model are now decoded; their original fixture
status and evidence remain unchanged. **OG remains unqualified for the
production storage module.**

A further 19 pool-gate controls distinguish ten finite numeric matches, six
recursive paths explicitly rejected by the host model, and three clock
underflow cases. The six recursive paths revert in RPC at both tested gas
budgets; they are not treated as zero balances.

The token is rank 411, `0xe18102869d32181aea317a40c5c4ce90ca591913`.
Its `balanceOf` adds pending hourly and daily rewards to the holder's raw root-0
balance. The helper is `0x1430c0bd0d023690f7aee3666daf265c498cc6a5`, with
selectors `e8e8fe04` and `7ac1f9f8`. Token and helper source lookups returned 404
from Sourcify; this work uses reviewed deployed bytecode and hash-pinned state.
Descriptive field names below do not claim verified Solidity source names.

| Actual RPC height | Hourly reward | Daily reward | Final balance |
| --- | ---: | ---: | ---: |
| 122288107 | 12931791375367022307 | 117088217914400390 | 1062462782039865649749 |
| 122288346 | 825762894943952709 | 4296282993261282764 | 371210601679873098541 |
| 122288436 | 2567321952608422165 | 0 | 3590497677060825519 |
| 122288829 | 634122505087826502 | 92333492716542292 | 212237419009570907473 |
| 122289015 | 645197321804747236 | 211168774963253239 | 223604192284746754801 |

Each fixture records its holder, block hash, raw storage words, runtime bindings,
fresh RPC expectations, and original failed survey row. The survey's `before`
rows name the changed block; the actual RPC height above is one block earlier.
No returned balance or helper output seeds the calculation.

The model reads holder records and cursors, hourly/daily total and user rates,
period rewards, the epoch, and cap fields. Zero period rates carry their previous
values. The initial hourly user rate falls back to holder field 5 when zero.
The hourly loop stops at the lesser of the current hour and the prior cursor
plus 168. The daily loop stops after three periods with positive total, user,
and reward inputs; a quotient that rounds down to zero still counts.

Both reward paths apply the same cap. It checks the holder's weekly activity,
then compares existing and pending value with the remaining cap. Value uses
two router `amountOut` quotes: OG → WBNB → USDT. Deployed router bytecode and
trace operands confirm the 9975/10000 fee constants and reserve1-to-reserve0
direction in both pools. All arithmetic preserves checked uint256 operations
and division rounding at the deployed steps.

The original controls cover raw balance changes and overflow, reward/rate
changes, initial and carried rates, zero-total skips, the three-period daily
limit, floor-to-zero contributions, weekly 168/169-hour boundaries, percentage
threshold equality, short-circuited and active overflows, cap clipping, reserve
changes, early exits, and clock changes. The original fixtures preserve the
three formerly unsupported controls: hourly or daily zero reward with a
positive total, and an initial zero daily user rate. Regression assertions now
require their captured outputs to match the extended model.

When a period's stored reward is zero and its carried total rate is positive,
the helper calculates a preview from the pool's raw balance. Its rate is
`min(1200 + 240 * floor(currentDay / 10), 3600) / 24`, with integer division.
The current day comes from the block timestamp and token epoch, even when
previewing an older period. The hourly preview burns
`floor(pool * rate / 100000)` and allocates 30% of that burn. The daily preview
repeats up to 24 hourly burns, allocating 15% per iteration. Each multiplication,
division, accumulation, and pool update preserves the deployed order.

Zero burn leaves the pool unchanged. A nonzero stored reward or zero carried
total skips preview without consuming the simulated pool. A zero user rate
does not skip preview, though it prevents that period's reward allocation.
Hourly and daily helpers each start from the original pool balance. The daily
initial rate falls back to holder field 6 only when its stored rate is zero
and holder field 8 is positive; otherwise zero carries until a later nonzero
user rate arrives.

The 49 fresh controls test all three resolved branches, tiny pool rounding,
pool multiplication overflow, repeated previews, mixed stored and previewed
rewards, zero-total skips, zero-user preview followed by recovery, three-day
stopping, and clock boundaries at days 0, 9, 10, 99, 100, 109, 110, and 365.
Eleven fresh traces also provide ten internal preview returns, binding both
the reward and remaining pool rather than only the final holder balance.

The strict fixture decoder rejects missing words, unreviewed runtime hashes,
and changed stored dependency addresses. Holder-record byte 20 is a uint8
field; it is not assumed to be a boolean. The sampled clock source is helper
slot 1: the reviewed EXP/DIV sequence uses byte offset zero. All observed
SLOAD values in the five successful traces are bound to freshly captured raw
words in the evidence report.

Implementation and captured regressions:

- [Host arithmetic](../tools/src/og_model.rs) and [raw-state decoder](../tools/src/og_model/fixture.rs)
- [Rust regressions](../tools/src/og_model/tests.rs)
- [Historical snapshots](../tests/fixtures/og-model/historical.json) and [RPC controls](../tests/fixtures/og-model/controls.json)
- [Preview snapshots and controls](../tests/fixtures/og-model/preview.json)
- [Initial evidence](evidence/og450-host-model.json) and [preview evidence and bytecode excerpts](evidence/og450-preview-model.json)
- [Pool-gate controls](../tests/fixtures/og-model/recursive.json), [1024-block pool scan](../tests/fixtures/og-model/pool-range.json), and [pool-gate evidence](evidence/og450-recursive-model.json)

Run the focused checks with:

```sh
cargo test --locked -p erc20-balances-storage-tools --lib og_model::tests
```

This adds no production layout, protobuf, module, or dependency. It does not
solve retained state, holder enumeration, changed runtime qualification, or
network portability. The fixture decoder conservatively requires a complete
initialized snapshot and limits its daily horizon to 1000 periods. Numeric
overflow rejection is tested; arbitrary router revert ABI parity is outside
this model's scope.

The pool's nonzero hourly rate can still return its raw balance when both
helper entry gates exit. Hourly exits for a zero hourly rate, current hour zero,
or an hourly cursor at or beyond the current hour. Daily exits for either zero
rate, current day zero, or a daily cursor at or beyond the current day. A
nonzero hourly rate therefore requires all four raw pool cursor words. The
successful zero-time traces call the cursor getters before their zero-time
exits. For an underflowing clock, requiring those words is conservative: the
clock getter reverts before the cursor call. The fixture decoder preserves the
original time inputs while planning an empty period horizon, allowing the
model to apply early exits before checked subtraction.

If either pool helper proceeds beyond those gates, it calls the same token's
`balanceOf(pool)` before entering its period loop. That call sees unchanged
state and repeats. Zero period totals, user rates, and rewards do not avoid the
call. Bounded traces show repeated pool-balance calls followed by out-of-gas
propagation; RPC calls at 1 million and 2 million gas revert. The host model
continues to reject these paths explicitly rather than simulate a gas budget.

The actual pool record and all four cursors were unchanged throughout the
original ranking window, blocks 122288006–122289029 inclusive. Both reward
rates were zero, and canonical RPC `balanceOf(pool)` equaled the independently
captured raw balance in all 1024 blocks. State, stored dependency addresses,
and balances were read at each canonical hash; token/helper runtime checks
were performed at the window boundaries. This establishes the sampled window,
not a permanent invariant.

Two investigation failures remain recorded. The first recursive call-trace
attempt returned an RPC JSON decoding error; its cause was not established.
Later bounded traces succeeded. The first error comparator then incorrectly
required `error.data` to be `0x`; the provider returned code 3, `execution
reverted`, with that field omitted. The corrected evidence preserves the
omission and classifies these as guarded reverts, without inventing a numeric
result. Production state retention and qualification remain separate work.
