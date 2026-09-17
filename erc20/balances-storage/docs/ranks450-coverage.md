# Nineteen more BSC profiles: 412 of the first 450

Nineteen candidates from ranks 401–450 pass separate packaged RPC and
initialized-holder checks, bringing the explicit test configuration to
**412 profiles**. Ranking measures observations in the sampled RPC stream,
not market capitalization. The tested interval remains **122288006–122289029**,
inclusive, with complete clocks for all 1,024 blocks.

| Cohort | Profiles | Emitted RPC checks | Emitted zeros | Initialized observations | Carry-forward matches | Final holders | Final zeros |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Five direct runtime matches | 5 | 74 | 23 | 143 | 69 | 84 | 52 |
| APM zero-word fallback | 1 | 16 | 0 | 25 | 9 | 11 | 0 |
| Thirteen proxy profiles | 13 | 270 | 31 | 356 | 86 | 167 | 64 |
| **New profiles total** | **19** | **360** | **54** | **524** | **164** | **262** | **116** |

All completed gates have **zero value mismatches**. Independent parent
checkpoints initialize 262 observed token/holder pairs with 524 balance/storage
reads. Actual production-WASM output drives retained state; processing makes
no balance RPC calls, and reference rows never initialize or repair that state.
Every initialized holder is checked against fresh RPC at the final block.

## What changed

[APM](apm450-coverage.md) corrects two plain-mapping mismatches using the existing
`zero_balance` rule: a zero mapping word returns scalar slot 7, currently
7,000,000,000 raw units. Slot 7 remains protected against changes, including
changes restored within a block. Nine unchanged nonzero observations need an
independent holder checkpoint; absent writes never become inferred zeros.

The [five direct profiles](direct450-coverage.md) use exact previously reviewed
runtime bytes with fresh candidate-specific getter and metadata controls.
Their layouts include the reviewed allowance roots and the four-word nested
record used by 145-NO. The fifth record word and unknown roots still reject.

The [thirteen proxy profiles](family450-coverage.md) bind each candidate's
forwarder, effective implementation and beacon dependencies. Runtime similarity
alone is insufficient. AAVE's initially unexpected transparent-proxy admin
read was separately reviewed: canonical caller behavior, nonzero admin at the
boundaries, actual-admin and zero-admin reverts, and rejection of persisted
admin changes/restoration without holder writes. No new production rules,
map modules or protobuf types were introduced.

Sixteen new Rust regressions retain 22 captured transaction fixtures with
60 independent RPC expectations, 380 successful raw/fallback controls and
116 metadata controls. They cover the original mismatches, exact override
payloads, null-address filtering, unknown roots, record widths and protected
implementation/admin/fallback dependencies. Earlier failed investigations and
diagnostic setup attempts remain linked from their cohort reports.

All **308 workspace Rust library/binary tests** pass, along with Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.
The separate OG model adds three host tests for five historical mismatches,
40 supported RPC controls and three explicitly unsupported controls.

## Combined replay and limits

The [fresh combined capture](evidence/ranks450-combined.json) uses
[`bsc-ranks450-layouts.json`](../tests/fixtures/bsc-ranks450-layouts.json).
Every protobuf event field matches for **109,830 previously RPC-verified
emitted balances**, including **15,236 zeros**. There are **411 emitting
profiles**; hLBP retains separate older mint evidence. Identical event histories
preserve **179,886 initialized observations**, including **70,057 without a
new event**.

This binds the verified 393-profile capture and the three new cohort captures
to fresh canonical headers. It reuses immutable RPC evidence and does not
establish a new full-412 holder checkpoint or global enumeration. Without their
bounded checkpoints, the new cohorts retain **162 unknown observations,
including 54 nonzero balances**. Raw event counts therefore do not establish
identical cold stream coverage.

The production package remains
`f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81` and its WASM
remains `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`.
The interface is one RPC-free `map_events`, shared `evm.balances.v1.Events`,
explicit verified layouts and default parameters `[]`.

The [summary](evidence/ranks450-summary.json) lists all 38 unqualified profiles
among the first 450. Seven are from the original first 400: LBP, TITAN, ORD,
YBC, 钻石, BabyDoge and 10SET. Thirty-one remain from ranks 401–450, including
OG. The [OG host model](og450-host-model.md) is a separate experiment and does
not add a production layout. Reflection and other calculated getters still
need complete dependency initialization, affected-holder output, restart and
rewind handling. Ethereum, Base, HyperEVM and Arc require independent
qualification after BSC.
