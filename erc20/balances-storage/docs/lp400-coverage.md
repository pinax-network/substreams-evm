# Three Cake-LP profiles: balance-independent metadata

Three additional candidates from the sampled BSC RPC stream bring the explicit
test configuration to **383 of the first 400 profiles**. Seventeen remain
unqualified. Ranking measures observations in that stream, not market value.
The interval is **122288006–122289029**, inclusive.

| Rank | Token contract | Balance root |
| ---: | --- | ---: |
| 353 | `0x29e8f7b1195cae4f761815bdec29200e15566983` | 1 |
| 355 | `0x528f7144a1ad78e05c7226ffbcbef9ee9223f382` | 1 |
| 361 | `0xb72973bdf9abf343be53ff9ac47c3a8305a5c8ee` | 1 |

All three return symbol `Cake-LP` and share the reviewed runtime hash
`0x60e4bcb14447615ab7c14fda2c2d70ca4191570e8841c75618e627c8f72662f8`.
These are explicit contract/runtime qualifications, not general support for
every contract with this symbol. Verified source is not claimed.

## Initial rejection and getter review

The [initial inspection](evidence/lp400-full-initial-inspection.json) preserves
nine rejected blocks per token when no metadata roots were configured. The
first unresolved write was slot 12, the LP operation lock. This was a refusal
to emit incomplete output, not a demonstrated incorrect RPC balance value.
The [persisted-write review](evidence/lp400-full-write-review.json) retains
the fixed-field and balance writes from all nine failing blocks, including
lock transitions from 1 to 0 and back to 1.

The [qualification](evidence/lp400-full-qualified.json) binds the complete valid
`balanceOf` bytecode path to the historical runtime. After selector and ABI
validation, it hashes the address with root 1, reads one word, and returns that
word unchanged. There is no secondary read, external call, holder-dependent
value branch, or caller/time dependency in the valid balance getter. The older
ABI decoder masks the high address bits; three independent dirty-address
controls confirm this behavior.

The [field review](evidence/lp400-full-fields.json) includes executed paths and
trace hashes for all configured metadata roots:

| Root | Meaning | Reviewed representation |
| ---: | --- | --- |
| 0 | Total supply | uint256 |
| 2 | Allowances | Nested owner/spender mapping |
| 3 | Domain separator | bytes32 |
| 4 | Permit nonces | Address mapping |
| 5, 6, 7 | Factory, token0, token1 | Low 160 bits |
| 8 | Reserves and last timestamp | uint112, uint112, uint32 |
| 9, 10 | Cumulative prices | uint256 |
| 11 | Last reserve product | uint256 |
| 12 | Operation lock | Mint entry requires 1, then writes 0 |

Ten fixed fields and two mapping roots are independently checked with zero and
maximum-word overrides. All **72 metadata controls** preserve a separately
overridden balance of 123. The metadata getters also return their expected
values and packing. For the lock, zero and maximum both produce the exact
`Pancake: LOCKED` revert while `balanceOf` remains 123. The lock trace qualifies
the entry guard only; later mint execution is not evidence about balances.
Another **60 raw-word controls** cover five addresses per token with zero,
one, 123 and uint256 maximum. Every value matches RPC.

The first six-field trial covered metadata written in this window. The final
configuration additionally reviews allowance, nonce, domain and address fields
so ordinary approval/permit writes are not omitted merely because they were
absent from the sample. The final expanded configuration was independently
replayed and audited. Unknown roots still stop processing. No production
algorithm change, extra module or new protobuf is needed.

## Packaged output and holder state

The [packaged RPC audit](evidence/lp400-full-rpc.json),
[native holder replay](evidence/lp400-full-holder-coverage.json) and
[packaged holder replay](evidence/lp400-full-wasm-holders.json) pass:

| Measurement | Three new profiles |
| --- | ---: |
| Emitted balances independently checked against RPC | 54 |
| Emitted zeros | 0 |
| Initialized observed holders | 36 |
| Parent-checkpoint balance/storage reads | 72 |
| Initialized reference observations | 108 |
| Matches without a new balance event | 54 |
| Final holders checked against RPC | 36 |
| Final zero balances | 30 |
| Cold unknown observations | 54 |
| Cold unknown nonzero observations | 0 |

Canonical clocks cover all 1,024 blocks, including empty outputs. RPC reads are
hash-pinned. Processing makes no balance RPC calls, and reference rows never
repair retained state. Cold unknown observations remain unknown even when the
reference value is zero. The final snapshot checks every initialized observed
holder, not every possible holder.

Four Rust regressions in `src/lp400_tests.rs` retain three captured transaction
fixtures with six independent RPC expectations, reproduce the initial lock
rejection, compare all 72 metadata controls and 60 raw-word controls, and reject
an unreviewed nested mapping. Null-address raw balances are still checked;
their 12 public-output cases are excluded consistently with `erc20/balances`.
All **274 workspace Rust library/binary tests** pass, together with Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.

The [combined capture](evidence/lp400-full-combined.json) runs
[`bsc-lp400-full-layouts.json`](../tests/fixtures/bsc-lp400-full-layouts.json).
Every protobuf event field matches for **109,235 previously RPC-verified
emitted balances**, including **15,167 zeros**. There are **382 emitting
profiles**; hLBP retains separate older mint evidence. Identical event histories
preserve **179,025 initialized observations**, including **69,791 without a new
event**. This reuses immutable RPC evidence with fresh canonical headers. It
does not establish a new full-383 holder checkpoint or global enumeration.

The production artifacts remain unchanged:

- SPKG SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`
- WASM SHA-256: `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`

The [summary](evidence/lp400-full-summary.json) lists the seventeen remaining
candidates. This qualification does not establish global holder enumeration,
production reflection support or all future metadata paths. Ethereum, Base,
HyperEVM and Arc still need independent qualification after BSC. The interface
remains one RPC-free `map_events`, shared `evm.balances.v1.Events`, explicit
layouts and default parameters `[]`.

Reproduce emitted checks with the Rust `audit-rpc` command,
`tests/fixtures/lp400-full/layouts.json`, start 122288006 and 1024 blocks. Use
`holder-coverage` with the original ranking/reference and complete Extended
block directory for native holder comparison. Use fresh output directories and
preserve unsuccessful evidence.
