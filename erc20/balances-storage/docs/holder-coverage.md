# BSC reviewed layouts and holder coverage

This page records the first four-token qualification. The subsequent
[eight-token qualification](next-bnb-candidates.md) adds ETH/BUSD/CAKE/USD1,
fixes the vUSDT RPC return decoder, and diagnoses all 280 earlier unresolved calls.
The hashes and measurements below remain evidence for the earlier build.

The expanded tests qualify explicit USDT, BTCB and USDC configurations alongside
the existing WBNB configuration. Across two independent windows, **8,772 emitted
WASM balances matched RPC**, including 1,358 zeros. A separate native consumer
test matched **8,984 holder observations** after initialization from a bounded
historical checkpoint.

This establishes correct values for the tested holders and windows. It does not
establish complete global holder coverage or identical event rows to the RPC
reference. A cold consumer still had 2,373 unknown observations, including 700
nonzero balances. Production checkpoint loading, complete holder enumeration and
sink integration remain unfinished.

## Reviewed configurations

The [reviewed fixture](../tests/fixtures/bsc-reviewed-layouts.json) is supplied
explicitly by the caller. It is not a built-in token list. The package still has
only `map_events`, uses the shared `evm.balances.v1.Events` protobuf, and performs
no RPC calls.

| Token | Contract | Balance slot | Allowance slot | Classification |
| --- | --- | ---: | ---: | --- |
| USDT | `0x55d398326f99059ff775485246999027b3197955` | 1 | 2 | Direct mapping |
| WBNB | `0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c` | 3 | 4 | Direct mapping; existing qualification |
| USDC | `0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d` | 1 | 2 | Direct mapping behind a pinned EIP-1967 proxy |
| BTCB | `0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c` | 1 | 2 | Direct mapping |

[Source/runtime evidence](evidence/core-source-qualification.json) binds the
verified Sourcify records to the historical runtime at block 122288005 for
USDT, BTCB, the USDC proxy and its implementation. Source review found that
`balanceOf(account)` returns `_balances[account]`; transfer, mint and burn update
that mapping. The fixture classifies the independent allowance mapping and
reviewed owner/supply slots. Unknown writes still fail instead of being ignored.

USDC delegates to `0xba5fe23f8a3a24bed3236f05f2fcf35fd0bf0b5c`. Its configuration
pins the proxy runtime, EIP-1967 implementation slot, implementation address and
implementation runtime. The Rust qualification tools check all identities at
the range boundaries. During continuous processing, the mapper rejects every
persisted write to the implementation slot, including an upgrade followed by an
upgrade back. It also rejects code changes at either address. Reverted changes
are ignored. The initial identity check remains the caller's responsibility;
configuration alone cannot verify state before the first block. Proxy admin
changes are unclassified writes and fail as well.

Primary source records:
[USDT](https://sourcify.dev/server/v2/contract/56/0x55d398326f99059ff775485246999027b3197955?fields=all),
[BTCB](https://sourcify.dev/server/v2/contract/56/0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c?fields=all),
[USDC proxy](https://sourcify.dev/server/v2/contract/56/0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d?fields=all),
[USDC implementation](https://sourcify.dev/server/v2/contract/56/0xba5fe23f8a3a24bed3236f05f2fcf35fd0bf0b5c?fields=all).

## Emitted WASM balances against RPC

Every emitted balance was checked using canonical block-hash-bound `balanceOf`
calls. Both captures used the rebuilt package, with no native replacement for
the mapper in this test.

| BSC range, inclusive | Blocks | Emitted balances checked | Zero balances | Mismatches |
| --- | ---: | ---: | ---: | ---: |
| 122288006–122288069 | 64 | 4,144 | 718 | 0 |
| 122284478–122284541 | 64 | 4,628 | 640 | 0 |

Reports: [current window](evidence/core-rpc-current.json),
[independent window](evidence/core-rpc-independent.json).

Audited package SHA-256:
`08b099887e1e3ae1c39d8d0f31ce80f8a6aa7bf5e350e66ad4fbd1c2ea7d14a3`.
WASM SHA-256:
`bee8080def54f32419df26f74be011bc23046cb671044a5c31afce9a11f93d32`.

## Holder state after initialization

The Rust `holder-coverage` command replays consecutive Extended blocks through
the same native mapper into two off-chain states: one starts empty, and one
starts from an explicit test checkpoint. The checkpoint includes only addresses
observed in that window's RPC reference. At the parent block, it independently
reads the mapping word and `balanceOf` and requires equality. It records every
setup read and its digest. Reference values are never used to fill gaps during
replay. Balance updates come only from the mapper; canonical header checks still
use RPC. Block gaps and forks fail the test.

| Token | Reference observations, both windows | Direct emissions | Matching retained values | Unknown without checkpoint | Matches with checkpoint |
| --- | ---: | ---: | ---: | ---: | ---: |
| USDT | 6,233 | 4,785 | 131 | 1,317 | 6,233 |
| WBNB | 1,829 | 1,132 | 0 | 697 | 1,829 |
| USDC | 729 | 447 | 10 | 272 | 729 |
| BTCB | 193 | 106 | 0 | 87 | 193 |
| Total | 8,984 | 6,470 | 141 | 2,373 | 8,984 |

The current 64-block window used 3,364 checkpoint addresses and 6,728 setup
balance/storage reads. The independent 32-block window, 122284478–122284509, used
2,026 addresses and 4,052 setup reads. These are per-window counts, not globally
distinct holders. Both runs had zero seeded unknowns or value mismatches.

Reports: [current holder replay](evidence/core-holder-coverage.json),
[independent holder replay](evidence/core-holder-coverage-independent.json).

The remaining coverage requirement is starting state. Retaining prior writes
helps, but cannot recover a holder whose balance was established before capture
and has not changed since. The same protobuf schema does not imply the same row
set: the RPC reference can emit a touched holder whose storage did not change.
A production consumer needs a trusted complete checkpoint or sufficient complete
history and must preserve unknown separately from zero. This test does not add a
store, extra map/cache, production checkpoint loader or deployed consumer.

## Expanded top-50 survey

The RPC reference emitted **190,651 rows across 1,696 contracts** in BSC blocks
122288006–122289029. The top 50 were selected by emitted row count. The native
survey replayed 339 unique Extended blocks: up to eight active samples per
selected token plus the continuous 64-block holder window. It made **57,686
before/after candidate checks**. These totals include 280 unresolved RPC/error
results, which remain failures to establish evidence rather than zero balances.

| Result | Tokens |
| --- | ---: |
| Candidate values matched; survey does not qualify layouts | 40 |
| Value mismatch | 5 |
| Unresolved RPC results | 3 |
| Insufficient active samples | 1 |
| No sufficient direct-mapping evidence | 1 |

The reviewed USDT/WBNB/USDC/BTCB configurations had respectively
31,566 / 8,280 / 3,260 / 734 checks, zero value mismatches, and zero strict mapper
errors. The other 36 matching candidates still need semantic/source review and
independent qualification. None of the 50 had exact raw event-row parity.

Seven value mismatches occurred across five tokens. Each had a zero mapping word
confirmed independently by `eth_getStorageAt`, but a nonzero `balanceOf` at the
parent boundary. Only the original counterexample below has been traced deeply;
the other four cannot yet be assigned the same explanation. The three unresolved
contracts account for 1, 143 and 136 RPC/error results respectively. Those original
failures are preserved in the run evidence.

The completed survey retains the aggregate status `insufficient_samples`, due to
its status precedence, and exits nonzero. `investigation_complete: true` records
that the investigation finished; it is not an all-pass result.

Evidence: [ranking summary](evidence/top50-ranking-summary.json),
[all 50 results](evidence/top50-parity-summary.json),
[seven mismatch records](evidence/top50-mismatches.json). Summary files retain
the full local report hashes and check-file hashes while omitting repeated errors,
rejected mapping hypotheses and long height lists. Full raw files remain under
the ignored `out/top50-1024` and `out/top50-parity` directories.

## Why the original token mismatched

For `0x2b74ac567edaa7a65237b4f92042667e784439d3` (symbol `无头苍蝇`), historical
execution at blocks 122284598 and 122284985 reads the holder mapping at base slot
8. If that word is zero, it reads global slot 4 instead. Slot 4 contained
`8000000000000000000` raw units in both cases. The observed path is:

```text
word = storage[keccak256(pad(holder) || pad(8))]
balanceOf(holder) = word != 0 ? word : storage[4]
```

The traces show the mapping `SLOAD` at PC 2082 and fallback `SLOAD` at PC 2094.
Read-only state overrides returned exactly 1, 123 and uint256 max when those
values replaced the holder word; replacing it with zero returned the global
default again. No transaction was sent. See the
[two historical inspections](evidence/default-balance-explanation.json).

The Firehose word was correct. The direct-mapping hypothesis was incomplete.
This token must remain unsupported by that adapter: even a holder with no write
can have a nonzero public balance, and a change to the shared default may affect
many holders. There is no hardcoded token exception in the mapper. A captured
Rust regression test preserves this counterexample and requires rejection of
the direct-mapping classification.

## Reproduce with Rust tooling

Run from the repository root with the existing provider environment configured.
Use new output directories; the tools preserve earlier runs. The endpoint options
can be set to an accessible BSC Substreams/Firehose service.

```sh
cargo run --locked -p erc20-balances-storage-tools -- rank-tokens \
  --start 122288006 --blocks 1024 --top 50 \
  --output erc20/balances-storage/out/recheck-ranking

cargo run --locked -p erc20-balances-storage-tools -- capture-blocks \
  --ranking erc20/balances-storage/out/recheck-ranking/report.json \
  --samples-per-token 8 --output erc20/balances-storage/out/recheck-samples

cargo run --locked -p erc20-balances-storage-tools -- capture-blocks \
  --start 122288006 --blocks 64 \
  --output erc20/balances-storage/out/recheck-continuous

cargo run --locked -p erc20-balances-storage-tools -- test-ranked \
  --ranking erc20/balances-storage/out/recheck-ranking/report.json \
  --block-dir erc20/balances-storage/out/recheck-samples \
  --block-dir erc20/balances-storage/out/recheck-continuous \
  --layouts erc20/balances-storage/tests/fixtures/bsc-reviewed-layouts.json \
  --output erc20/balances-storage/out/recheck-survey

cargo run --locked -p erc20-balances-storage-tools -- audit-rpc \
  --start 122288006 --blocks 64 \
  --layouts erc20/balances-storage/tests/fixtures/bsc-reviewed-layouts.json \
  --output erc20/balances-storage/out/recheck-wasm

cargo run --locked -p erc20-balances-storage-tools -- holder-coverage \
  --ranking erc20/balances-storage/out/recheck-ranking/report.json \
  --block-dir erc20/balances-storage/out/recheck-continuous \
  --layouts erc20/balances-storage/tests/fixtures/bsc-reviewed-layouts.json \
  --output erc20/balances-storage/out/recheck-holders
```

For the independent window, rank/capture from 122284478, capture 32 consecutive
blocks for holder coverage, and run the WASM audit over 64 blocks. Use
`inspect-balance` for canonical historical execution, storage reads and read-only
zero/nonzero controls. All executable test tooling and regression tests are Rust.
