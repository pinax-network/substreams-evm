# Fallback balances, vUSDT and holder coverage

This page preserves the 14-token qualification. The subsequent
[beacon and zero-path follow-up](beacon-and-zero-paths.md) expands the fixture to
20 tokens and finds three additional fallback paths missed by transfer sampling.

The explicit [14-token test fixture](../tests/fixtures/bsc-fallback-layouts.json)
adds vUSDT and five zero-word fallback profiles to the previous eight layouts.
The package still has one RPC-free `map_events`, an empty default configuration,
and the shared `evm.balances.v1.Events` output. These are bounded qualifications,
not a claim that all ERC-20s or all holders are covered.

## Why seven historical values differed

For five contracts, `balanceOf(holder)` returns a nonzero fallback when the
holder's mapping word is zero. The raw storage and Firehose data agreed; treating
that raw zero as the public balance was wrong. The new optional `zero_balance`
rule projects zero to an explicitly reviewed value and leaves nonzero words
unchanged. No token address or fallback value is built into the mapper.

| Contract | Mapping base | Zero-word fallback, raw units | Dependency |
| --- | ---: | --- | --- |
| `0x45056c2627c9e60753aeef604ee9575709f8e88f` | 8 | `7000000000000000000` | Scalar slot 4 |
| `0x2b74ac567edaa7a65237b4f92042667e784439d3` | 8 | `8000000000000000000` | Scalar slot 4 |
| `0x5c8fb0f805f62cd082c00d9d250a388b543cc453` | 8 | `8000000000000000000` | Scalar slot 4 |
| `0x1922cf6ef99e41591a33687816baee1d5efc6db7` | 0 | `9000000000` | Scalar slot 7 |
| `0x70a163085db5617b700fafeac1c7a2794e41a724` | 2 | `320173463400000000000` | Constant in pinned implementation |

The [inspection evidence](evidence/fallback-source-and-behavior.json) preserves
runtime identities, historical hashes, storage reads and read-only controls.
For all four scalar-dependent profiles, overriding the holder word to zero and
the dependency to 0, 17 or uint256 max returns the dependency. With holder word
1 and dependency 17, the result stays 1. Mapping-only controls also cover 123 and
uint256 max. The fifth profile delegates to
`0x75cc906469c92fb3f742a79369a30102d129fae9`; its zero branch returns the constant
loaded by `PUSH9` at PC 3342. Both proxy and implementation runtimes are pinned.

These five profiles were reviewed through bytecode, execution traces and state
overrides; verified Solidity source was unavailable. They do not have the same
source provenance as the nine direct-mapping profiles. Controls and sampled
parity alone do not establish universal contract semantics.

The [retest](evidence/fallback-resolved-mismatches.json) joins every one of the
**seven original mismatches** to its new result by contract, holder, block,
boundary and canonical hash. All seven now match the same historical RPC value.
The original raw zero and failed result remain in the evidence. Rust regression
tests cover those seven examples through the public projection.

Raw words still drive within-block continuity: raw 0 and raw fallback can have
the same public value but must not be confused. Every persisted write to a
configured fallback dependency fails, including change-and-restore and a change
with no individual holder writes. A dependency change could affect arbitrarily
many holders; propagating that change and rebuilding their state remains future
work. Reverted writes do not trigger this guard. Null holder addresses are
excluded from public output, matching `erc20/balances`.

## vUSDT's remaining writes

The earlier ABI decoding fix resolved vUSDT's 96-byte return data. Its 64 native
mapper failures had a separate cause: other persisted storage was not yet
reviewed. Venus's [published deployment artifact at a pinned commit](https://github.com/VenusProtocol/venus-protocol/blob/42db10c5ad8ba6feb9e48959ad6427cfe8b06e08/deployments/bscmainnet/VBep20Delegate.json)
exactly matches the historical runtime of implementation
`0xcdfea50f7ceccb24fe804657db8e6c93b689941e`. The inspection verifies every literal
source hash in the artifact metadata. `VToken.balanceOf` directly returns
`accountTokens[owner]` at mapping base 14.

The reviewed configuration covers the allowance mapping at 15, the two-word
borrow record at 16, and explicitly listed scalar accounting/administrative
fields. The wrapper's custom implementation slot is **18**, not EIP-1967.
Existing proxy guards reject any persisted implementation-slot write or code
change. No unknown writes were automatically ignored. All six newly configured
tokens now have **zero strict mapper errors across 339 captured Extended blocks**.
Their [targeted survey](evidence/fallback-six-survey.json) contains 252 before/after
checks, zero RPC errors and zero value mismatches. It still exits nonzero with
`coverage_gap` because unchanged/reference-only rows are not emitted by storage.

## Actual WASM validation

| Inclusive BSC range | Emitted balances checked | Zero balances | Mismatches |
| --- | ---: | ---: | ---: |
| 122288160–122288415 | 18,261 | 2,373 | 0 |
| 122288560–122288815 | 16,285 | 2,652 | 0 |
| Total | **34,546** | **5,025** | **0** |

Both runs exercised **all 14 tokens**. The six additions account for 137 emitted
checks: 89 vUSDT and 48 across the five fallback profiles. Reports:
[window A](evidence/fallback-rpc-a.json), [window B](evidence/fallback-rpc-b.json),
[per-token counts](evidence/fallback-rpc-token-counts.json). These check every
emitted balance against hash-pinned RPC; they do not claim identical raw event
rows or complete holder enumeration.

Audited package SHA-256:
`21ea25a33a6819a9705b09487c69b2500a2e8d48474d34813f66a6b8c6edff7f`.
The final package includes the updated README and has SHA-256
`7c478b020a2f3f735518a77544191a257c3779ae7af0dc038c6b208093dff49d`.
Both packages have module hash `d2dd6a9e7b1fdd407e72728765a36791d4fed253`
and identical executable WASM.
WASM SHA-256:
`910b306ca0c3f3fbb202f09c90a9c42ea49f4a59190097e7dd965ef034a321d6`.

## Holder state and validation

The [128-block holder replay](evidence/fallback-holder-coverage.json), covering
122288160–122288287, initializes 5,595 observed holders at the preceding canonical
block. It matches all **13,605 reference observations** across all 14 tokens,
with zero seeded unknowns or value mismatches. The checkpoint includes eight
holders whose storage word is zero but public balance is nonzero, spanning all
five fallback profiles. It reads raw storage and independently compares the
projected value with `balanceOf` before initializing state.

Without that checkpoint, **4,248 observations remain unknown**, including
**1,138 nonzero balances**. Neither consumer has an incorrect known value in this
window. Setup uses 11,190 RPC reads; balance processing uses zero RPC calls.
Reference values are compared but never used to repair ongoing consumer state.
This is a bounded test checkpoint over observed addresses, not a complete global
holder snapshot or production bootstrap. Global fallback changes, complete holder
enumeration, deployment-boundary qualification and other token semantics remain
unfinished.

Validation passed 97 workspace library/binary tests, targeted Clippy with
warnings denied, the workspace WASM check and targeted formatting. An additional
unrestricted workspace test run hit an existing generated-protobuf doctest error
in `proto/src/pb/uniswap.v3.rs:204`; the CI-equivalent library/binary tests passed.

## Reproduction

All executable tests and tools remain Rust. From the repository root:

```sh
cargo run --locked -p erc20-balances-storage-tools -- audit-rpc \
  --start 122288160 --blocks 256 --workers 2 \
  --layouts erc20/balances-storage/tests/fixtures/bsc-fallback-layouts.json \
  --output erc20/balances-storage/out/my-fallback-audit

cargo run --locked -p erc20-balances-storage-tools -- capture-blocks \
  --start 122288160 --blocks 128 \
  --output erc20/balances-storage/out/my-fallback-holder-blocks

cargo run --locked -p erc20-balances-storage-tools -- holder-coverage \
  --ranking erc20/balances-storage/out/top50-1024/report.json \
  --block-dir erc20/balances-storage/out/my-fallback-holder-blocks \
  --layouts erc20/balances-storage/tests/fixtures/bsc-fallback-layouts.json \
  --output erc20/balances-storage/out/my-fallback-holder-coverage
```

The ranking is the preserved 1,024-block RPC stream starting at 122288006; use
`rank-tokens --start 122288006 --blocks 1024 --top 50` to reproduce it. Configure
the appropriate Substreams and Firehose endpoints separately. Local evidence
preserves an initial fetch failure against an endpoint without `sf.firehose.v2.Fetch`
at `out/fallback-holder-blocks/report.json`; the successful Firehose capture is
in `out/fallback-holder-blocks-public`. An earlier survey used an empty staging
layout file and failed before validation at `out/fallback-six-survey-initial`;
the valid run is separately retained at `out/fallback-six-survey-ready`.
