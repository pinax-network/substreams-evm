# BSC TOPS and StandardToken clone holder coverage

Two more profiles from the immutable top-250 ranking are qualified: **TOPS
(236)** and the **StandardToken clone (220)** at
`0x4bafbe9f5fdf15b4bd6210e55e100d2266b169e3`. The
[combined fixture](../tests/fixtures/bsc-tops-clone-layouts.json) contains
**241 profiles**. Eight candidates from ranks 201–250 and LBP (143) remain
outside that fixture. The [summary](evidence/tops-clone-summary.json) records
the remaining addresses and binds the reports below.

Production behavior is unchanged: one RPC-free `map_events`, shared
`evm.balances.v1.Events`, explicit caller-qualified layouts and default `[]`.
This batch adds qualification evidence and Rust regressions, without a
production store, protobuf change or token allowlist.

## TOPS getter and LP bookkeeping

The [qualification](evidence/tops-qualification.json) binds TOPS's verified
source to the runtime at both ends of **122288006–122289029**, inclusive. Its
getter directly returns the holder's word in balance mapping 5. The trace and
four independent word overrides—zero, one, 123 and uint256 maximum—agree.
Scalar, packed-struct and mapping fields are reviewed against the source
storage layout. Multiword struct fields are included only within their
declared sizes.

The LP-provider address list at root 28 uses the existing witnessed append
rule. No list writes occur in the original 1,024-block interval. A
[separate historical search](evidence/tops-list-activity.json) finds a
registration at **119571736**, where the list grows from **759 to 760**.
Adjacent canonical observations bind the transition; this is not an
earliest-registration claim. The full Extended block projects two balances,
both matched by the [packaged RPC audit](evidence/tops-list-activity-rpc.json).
Removing the address-list rule makes the captured block fail on the list root.

`lpInfos`, the mapping at root 31 containing dynamic arrays of three-word
records, remains **unconfigured**. Its creation/expiration paths were not
exercised in either capture. The profile must fail if those arrays change;
they are not ignored merely because `balanceOf` does not read them. A Rust
negative case injects a valid, preimage-bound array-length write and verifies
rejection. This is bounded qualification, not complete support for every TOPS
administrative or LP lifecycle path.

## StandardToken clone

The [clone qualification](evidence/standard-clone-qualification.json) verifies
the exact canonical 45-byte ERC-1167 runtime and pins its implementation,
`0x336b3d23809d754f2b40af53703626bdf87166f0`. The implementation's verified
`StandardToken` source is bound to historical runtime at both boundaries.
Its sole balance getter directly reads mapping 1. Allowances use mapping 2;
initialization, supply, maximum supply, metadata roots and decimals are
separately reviewed. Long-string data remains an unconfigured write.

The delegated getter trace and four independent word controls agree. The
complete native replay has no unresolved writes. A captured regression also
verifies that an implementation code change stops processing even in a block
without holder writes.

## RPC and holder evidence

The [TOPS](evidence/tops-rpc.json) and
[clone](evidence/standard-clone-rpc.json) audits run the actual packaged WASM
over all 1,024 baseline blocks. Initial holder values come from independent
canonical balance/storage reads. The
[TOPS](evidence/tops-wasm-holders.json) and
[clone](evidence/standard-clone-wasm-holders.json) retained-state replays bind
the actual event rows and complete clocks to those audits, never repair state
from reference rows, and check every retained holder at the final block.

| Original 1,024-block interval | TOPS | StandardToken clone |
| --- | ---: | ---: |
| Actual WASM balances checked against RPC | 34 | 48 |
| Emitted zero checks | 6 | 0 |
| Initialized observed holders | 7 | 26 |
| Checkpoint balance/storage RPC reads | 14 | 52 |
| Initialized reference observations | 66 | 72 |
| Matches without a new balance event | 32 | 24 |
| Final retained holders checked against RPC | 7 | 26 |
| Final zero balances | 4 | 0 |
| Cold unknown observations | 29 | 24 |
| Cold unknown nonzero observations | 20 | 24 |

All emitted checks, initialized observations and final holder checks match.
There are no processing balance RPC calls. Three captured Rust fixtures retain
complete token-writing transactions and reproduce the full blocks' output;
their six expected balances come from the completed independent RPC audits.
All **230 workspace Rust library/binary tests** pass, along with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff
checks. Repacking preserves both audited artifact hashes:

- SPKG: `d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`
- WASM: `861a879353a7fc7163a26c80f671fc3011e9ca3ee32a65dfc0badd99ac9794b6`

Cold unknown observations stay unknown. These counts cover observed holders,
not every address or a global holder enumeration. The two older TOPS list
balance checks belong to a separate single-block audit and are excluded from
baseline totals.

The [combined capture](evidence/tops-clone-combined.json) streams all 241
profiles and compares every protobuf field to the immutable prior 239-profile
capture plus the two independently audited cohorts. It checks fresh canonical
headers and layout dependencies. Reusing the prior balance observations is
conditional on identical package, layout, interval and output evidence; this
is not a fresh full-241 checkpoint or final-holder snapshot.

The aggregate preserves **105,419** previously RPC-verified emitted balances,
including **14,745 zeros**, and **172,819 initialized holder observations**,
including **67,401 matches without a new balance event**. Of the 241 configured
profiles, 240 emit in the baseline interval; hLBP retains its separate older
mint evidence. Every protobuf field in the new combined output matches.

## BLESS dependency investigation

BLESS (204) remains unqualified. The
[dependency inventory](evidence/bless-clone-dependencies.json) now identifies
an additional forwarding layer: its beacon at
`0x8244d6ffe0695b30b2bad424683ee3bc534ea464` delegates through the standard
ERC-1967 implementation slot to
`0x4c7ca8fcffe77281a8b81d4580cff8257d785491` (`DeBridgeTokenDeployer`). The
getter trace reads token implementation root **151** in beacon storage,
returning `0xcacebe8c354b70fa6e3107f3f6f699e4fbb3a98b`.

Both boundary traces agree. The delegate and token implementation have
historically bound verified source; the outer beacon's source is unavailable
in this inventory. Its trace also reads the transparent-proxy admin slot.
The complete forwarding behavior and relevant dependencies still need review;
these observations alone do not qualify the token or add it to coverage.

Ethereum, Base, HyperEVM and Arc remain the next network qualifications after
the BSC campaign. Their preliminary RPC probes do not establish token or
holder parity; see the [network sequence](network-expansion.md).
