# AR, ARZ and ARS: bytecode review of sparse discovery candidates

Three additional candidates from the original BSC RPC stream bring the explicit
configuration to **380 of the first 400 profiles**. Twenty remain unqualified.
The interval is **122288006–122289029**, inclusive. Ranking measures observations
in that stream, not market value.

| Rank | Symbol | Token contract | Balance root |
| ---: | --- | --- | ---: |
| 351 | AR | `0x19943c027d8deb171e91315cfcb446b9dcc16d25` | 5 |
| 354 | ARZ | `0x403d4d10b6de8d0cc9429ff17b689ea1b0c770b7` | 5 |
| 364 | ARS | `0xe9e6c90c5b4d94b12e3ffff7b0b74de7b177fb51` | 5 |

Each initial discovery sample contained only one nonzero holder, below the
unchanged two-holder threshold. The eight sampled raw-word comparisons per
token matched RPC, but were insufficient for automatic discovery. These were
coverage gaps, not historical value mismatches.

## Independent getter review

The [qualification](evidence/sparse400-qualified.json) preserves the discovery
results and source inventory, which did not provide verified source for these
contracts. Qualification instead uses manual review of the complete valid
`balanceOf` bytecode path, historical runtime binding, independent controls and
the strict native replay. It does not claim source verification.

All three paths validate the nonpayable call, selector, argument length and
canonical address, then calculate the slot-5 mapping key, perform one `SLOAD`
and return the raw word through ABI encoding. The reviewed getter body has the
same bytes at different code offsets. There is no holder-dependent value branch,
secondary storage read, external call, caller/origin dependency or clock read
on the valid getter path. Rejection edges for call value, a short address
argument and a noncanonical address use `PUSH1 0; DUP1; REVERT`.

The [read-only controls](evidence/sparse400-controls.json) contain **56 raw-word
checks** across zero, dead, maximum, token-contract and sampled holder addresses
(deduplicated when the sampled holder is the token). Zero, one, 123 and uint256
maximum all match RPC. Nine invalid-call checks exercise the reviewed rejection
edges. Fresh runtime checks cover the interval's parent and final blocks.

The [initial helper failures](evidence/sparse400-initial-controls.json) are
preserved. The endpoint omitted `error.data` for an empty revert, while the first
helper required `0x`. The revised check requires error code 3, a revert message
and absent or empty data, bound to the reviewed empty-revert bytecode. No balance
mismatch was discarded.

The profiles configure **only the runtime hash and balance root**. No metadata
slot is ignored. All 1,024 complete Extended blocks pass the native mapper;
any later unresolved metadata write still stops processing. These additions
need no production algorithm change.

## Packaged output and holders

The [packaged RPC audit](evidence/sparse400-rpc.json),
[native holder replay](evidence/sparse400-holder-coverage.json) and
[packaged holder replay](evidence/sparse400-wasm-holders.json) pass:

| Measurement | Three new profiles |
| --- | ---: |
| Emitted balances independently checked against RPC | 54 |
| Emitted zeros | 27 |
| Initialized observed holders | 36 |
| Parent-checkpoint balance/storage reads | 72 |
| Initialized reference observations | 108 |
| Matches without a new balance event | 54 |
| Final holders checked against RPC | 36 |
| Final zero balances | 31 |
| Cold unknown observations | 54 |
| Cold unknown nonzero observations | 18 |

Canonical clocks cover empty-output blocks. RPC reads are hash-pinned;
processing uses no balance RPC calls and reference rows never repair retained
state. Final checks cover every initialized observed holder.

Three captured transaction fixtures retain six independent RPC expectations in
`src/sparse400_tests.rs`. Additional regressions check all 56 raw getter values
against synthetic Extended storage writes. The public event output includes
the 44 non-null-address cases and excludes the twelve null-address cases,
matching `erc20/balances`'s address filter. The first test incorrectly expected
an event for the null address; correcting that expectation preserves the raw
value assertions and requires no production change.
All **270 workspace Rust library/binary tests** pass, together with Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.

The [combined capture](evidence/sparse400-combined.json) runs
[`bsc-sparse400-layouts.json`](../tests/fixtures/bsc-sparse400-layouts.json).
Every protobuf event field matches for **109,181 previously RPC-verified emitted
balances**, including **15,167 zeros**. There are **379 emitting profiles**;
hLBP retains separate older mint evidence. Identical event histories preserve
**178,917 initialized observations**, including **69,737 without a new event**.
This reuses immutable RPC evidence with fresh canonical headers; it does not
establish a new full-380 holder checkpoint or global enumeration.

The production artifacts remain unchanged:

- SPKG SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`
- WASM SHA-256: `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`

The [summary](evidence/sparse400-summary.json) lists all twenty remaining
candidates. This qualification does not prove future metadata paths, global
holder enumeration or production reflection support. Ethereum, Base, HyperEVM
and Arc still need independent qualification after BSC. The interface remains
one RPC-free `map_events`, shared `evm.balances.v1.Events`, explicit layouts and
default parameters `[]`.

Reproduce emitted checks with the Rust `audit-rpc` command,
`tests/fixtures/sparse400/layouts.json`, start 122288006 and 1024 blocks. Use
`holder-coverage` with the original ranking/reference and full Extended block
directory for native holder comparison. Use fresh output directories and
preserve unsuccessful evidence.
