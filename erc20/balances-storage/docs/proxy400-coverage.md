# Dood and DeepTokenOFT proxy qualification

Two additional candidates from the original BSC RPC stream bring the explicit
configuration to **377 of the first 400 profiles**. Twenty-three remain
unqualified. Ranking counts observations in that stream, not market value.
The tested interval remains **122288006–122289029**, inclusive.

The module still has one RPC-free `map_events`, the shared
`evm.balances.v1.Events` protobuf, explicit caller-supplied layouts and default
parameters `[]`. No production algorithm change or additional state module is
required for these two profiles.

## Getter and proxy dependencies

| Rank | Verified source name | Token contract | Implementation |
| ---: | --- | --- | --- |
| 390 | Dood | `0x722294f6c97102fb0ddb5b907c8d16bdeab3f6d9` | `0xdc9a9bf20f5e9d4d5efe866dd320a48611b37f39` |
| 391 | DeepTokenOFT | `0x9b6a1d4fa5d90e5f2d34130053978d14cd301d58` | `0x76ec4be0109305fb4543f1d1a8a515ac3bcabd60` |

The [source inventory](evidence/proxy400-source-review.json) binds verified
proxy and implementation sources to their historical runtimes at both interval
boundaries. The [qualification](evidence/proxy400-qualified.json) verifies the
implementation pointers, execution code hashes and fresh getter traces. Each
ordinary getter delegates once, reads only the implementation pointer and the
holder's balance word, and returns that word directly. Both use the namespaced
ERC-20 root
`0x52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00`.

The source's conventional storage-layout list is empty because these fields
use namespaces. Reviewed struct declarations, namespace constants and accessors
establish each field explicitly. They cover ERC-20 supply/name/symbol/allowances,
Ownable and Initializable. Dood's minter uses `keccak256("dood.minter") - 1`,
which differs from the ERC-7201 derivation used for its ERC-20 state.
DeepTokenOFT additionally uses LayerZero metadata, two-word AccessControl role
records, nonces, pause state, EIP-712 fields and a ban-list mapping. Unknown
dynamic payloads and unreviewed writes still stop processing.

The [initial review failure](evidence/proxy400-initial-qualification.json) is
preserved. It expected a direct Initializable accessor; source review established
the indirect `_initializableStorageSlot()` accessor and its single constant
return without overrides. The revised check accepts that exact source shape.
This was a review-helper rejection, not a discarded balance mismatch.

## Read-only RPC controls

Eight raw-word controls (zero, one, 123 and uint256 maximum for each token)
return the exact overridden balance. Ten additional controls cover ordinary
callers, Dood owner/minter values, DeepTokenOFT pause/ban-list state, its immutable
proxy admin and the ERC-1967 admin storage mirror. Nine return 123; the immutable
admin caller reverts with `ProxyDeniedAdminAccess()` (`0xd2b576ec`) as the proxy
source requires. Ordinary callers can read paused or banned holders' balances.

DeepTokenOFT's forwarding decision uses the code-bound immutable admin, rather
than the admin storage mirror. Zeroing that mirror leaves an ordinary
`balanceOf` call unchanged. Its writes remain unqualified in this configuration
and are deliberately rejected; this control does not expand supported
administrative behavior. Protected implementation changes also remain errors.

The [control fixture](../tests/fixtures/proxy400/controls.json) retains the exact
calls, overrides and RPC responses. Rust regressions replay the supported
overrides as synthetic Extended storage writes and compare mapper output to
those independent responses. A separate regression verifies that an unreviewed
admin-mirror write still stops processing. The special admin-caller revert is
recorded as RPC dispatch evidence, not a supported mapper call context.

## Packaged output and holders

The [packaged RPC audit](evidence/proxy400-rpc.json),
[native holder replay](evidence/proxy400-holder-coverage.json) and
[packaged holder replay](evidence/proxy400-wasm-holders.json) all pass:

| Measurement | Two new profiles |
| --- | ---: |
| Emitted balances independently checked against RPC | 33 |
| Emitted zeros | 3 |
| Initialized observed holders | 23 |
| Parent-checkpoint balance/storage reads | 46 |
| Initialized reference observations | 64 |
| Matches without a new balance event | 31 |
| Final holders checked against RPC | 23 |
| Final zero balances | 12 |
| Cold unknown observations | 31 |
| Cold unknown nonzero observations | 15 |

All 1,024 blocks are covered, including empty-output blocks through canonical
clocks. RPC reads are hash-pinned. Processing uses no balance RPC calls and the
reference stream never repairs retained state. The final check includes every
initialized observed holder. Two captured transaction fixtures retain four
independent RPC balance expectations in `src/proxy400_tests.rs`.
All **268 workspace Rust library/binary tests** pass, together with Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.

The [combined capture](evidence/proxy400-combined.json) runs
[`bsc-proxy400-layouts.json`](../tests/fixtures/bsc-proxy400-layouts.json).
Every protobuf event field matches for **109,127 previously RPC-verified emitted
balances**, including **15,140 zeros**. There are **376 emitting profiles**;
hLBP keeps its separate older mint evidence. Identical event histories preserve
**178,809 initialized observations**, including **69,683 without a new event**.
This comparison reuses immutable RPC evidence with fresh canonical headers;
it does not establish a new full-377 holder checkpoint or global enumeration.

The production artifacts are unchanged:

- SPKG SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`
- WASM SHA-256: `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`

The [summary](evidence/proxy400-summary.json) lists all 23 remaining candidates.
This qualification does not prove every future administrative path, cold-start
holder completeness or production reflection support. Ethereum, Base, HyperEVM
and Arc still need their own qualification after BSC.

Reproduce emitted checks with the Rust `audit-rpc` command, the
`tests/fixtures/proxy400/layouts.json` configuration, start 122288006 and 1024
blocks. Use `holder-coverage` with the original ranking/reference and full
Extended block directory for native holder comparison. Preserve failed runs
and use a fresh output directory for every run.
