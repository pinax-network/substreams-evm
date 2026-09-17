# OG host model: five storage mismatches explained

The host-only Rust model reproduces all five sampled OG mismatches, including
both helper return values and the hourly stopping cursor. It also matches 40
read-only RPC controls. Three controls exercise branches that the model
explicitly rejects because their arithmetic has not been decoded. **OG remains
unqualified for the production storage module.**

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

The 40 matching controls cover raw balance changes and overflow, reward/rate
changes, initial and carried rates, zero-total skips, the three-period daily
limit, floor-to-zero contributions, weekly 168/169-hour boundaries, percentage
threshold equality, short-circuited and active overflows, cap clipping, reserve
changes, early exits, and clock changes. The fixtures also preserve the three
unsupported controls: hourly or daily zero reward with a positive total, and
an initial zero daily user rate. The pool's own nonzero hourly reward rate is
another explicit unsupported dependency because it requires recursion.

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
- [Compact evidence and bytecode excerpts](evidence/og450-host-model.json)

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

The next bounded step is to decode the hourly and daily zero-reward preview
bodies at helper PCs 11119 and 6559, and the initial daily rate branch at
3852–3881, then add raw-state and clock controls. Production state retention
and qualification remain separate work.
