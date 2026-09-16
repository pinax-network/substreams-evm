# First deployment and holder coverage

The explicit [21-token fixture](../tests/fixtures/bsc-deployment-layouts.json)
adds rank 15, `0x29a864a830528667e571c7a8f5f722cf704753c2`. It was created at
BSC block **122288338**, inside the original reference window. Its constructor
minted `1000000000000000000000000000` raw units to
`0x3dc263d768385802c8d2d20e9f7f06f6e9128782`. The previous code-change guard
rejected that first deployment, even though the trace contained its initial
balance. This was a coverage failure, not a different post-deployment RPC value.

## Qualified deployment

The optional `deployment` configuration pins the block number and hash. It is
currently restricted to direct mappings without proxies or zero-word fallbacks.
The audit checks empty runtime and nonce zero before deployment, the pinned
runtime at deployment, and runtime identity at the test boundaries. Ingestion
requires exactly one successful, persisted CREATE code change with matching
runtime bytes, hash and execution provenance. It rejects activity before the
pinned deployment, nonzero first storage values and subsequent code changes.

Constructor writes occur **before** runtime installation: here the balance write
has ordinal 2219, code creation 2223, and the CREATE spans 2211–2224. The mapper
includes those writes and subsequent calls in the same block. No extra module,
cache, protobuf or RPC ingestion call was introduced.

The reviewed balance getter reads mapping base 0 directly; zero, 1, 123 and
uint256-max overrides return those same values. Allowances use base 1; constructor
scalars 2–5 contain supply, name, symbol and ownership. Verified Solidity source
was unavailable, so this profile has historical bytecode/trace/control evidence,
not source verification. [Balance inspection](evidence/deployment-getter-inspection.json)
and [other getter traces and controls](evidence/deployment-other-getters.json).

The committed 13 KB protobuf fixture retains the actual deployment transaction
and original header; unrelated transactions and system changes are omitted.
The WASM audit uses full streamed blocks. [Fixture and artifact digests](evidence/deployment-artifacts.json).

## Measured coverage

The [actual WASM audit](evidence/deployment-rpc.json) covers 256 blocks,
**122288330–122288585**: **18,878 balances**, including **2,582 zeros**, with
**zero mismatches**. All 21 configured tokens emitted balances; the new token
accounts for 50 checks, including the initial mint and one zero.
[Per-token counts](evidence/deployment-rpc-token-counts.json).

The [deployment-only holder replay](evidence/deployment-only-holder-coverage.json)
covers 128 consecutive blocks, **122288330–122288457**. All **33 reference
observations across 19 holders match**, including 13 observations carried forward
without a balance write. There are **zero checkpoint balance reads**, zero
processing balance RPC calls, and zero unknown or mismatching holder values.
RPC is still used for runtime qualification and canonical header checks.

The observed holder set is initialized to zero only after validating the first
CREATE, then that block's writes are applied. A nonexistent token has no public
`balanceOf` result: unavailable pre-deployment calls remain unavailable and are
never counted as successful zero comparisons. This establishes complete state
for these observed holders from this deployment, not all possible ERC-20 holders.

The [broader 21-layout replay](evidence/deployment-holder-coverage.json) matches
all **16,099 observations** from the **20 active tokens**. Existing tokens use
a 9,353-holder checkpoint (18,706 balance/storage RPC reads); the new token's
19 holders instead use the verified deployment baseline. Fruit had no reference
observations in this 128-block range. The original tool labeled this `mismatch`
because not every configured token appeared, despite zero value mismatches.
The report retains that original result and records the corrected offline
classification, `coverage_gap`, with the inactive token listed explicitly.
The tool now distinguishes untested tokens from incorrect or unknown balances;
neither condition returns a passing exit status. Without checkpoints for existing
tokens, 6,413 observations remain unknown, including 3,863 nonzero balances.

Rust regressions cover the captured mint, constructor writes before code
installation, later same-block changes, missing/reverted/duplicate creation,
wrong code/block identity, invalid ordinals, prior storage, redeployment,
pre-deployment runtime/nonce checks, holder initialization and inactive-token
classification. The shared output remains `evm.balances.v1.Events` from one
RPC-free `map_events`, with empty default layouts.

Validation passed **112 workspace Rust library/binary tests**, targeted Clippy
with warnings denied, the workspace WASM check, targeted formatting and diff
checks. The rebuilt package exactly matches the audited package SHA-256
`36c8a7720ae7f80b31b3988addc8ff8d51f8272ed95a06839c092acafa47eeea`;
the WASM digest is
`47e738dd699373dc897e80bf0c0963063e38f64e9718dc6dffb6a0b2bb39ed22`.
These are library/binary test results; the previously documented generated-proto
doctest failure is outside that check.

## Computed rank-7 token

For `0xec22e64c0a16821dc1b457045936c0219b47155e`, the 3,002 reference balances in
block 122288080 do not come from balance writes. Ordinary holders receive:

```text
(uint256(keccak256(packed 20-byte holder address)) % 9001 + 8434) * 10^18
```

That formula explains 3,001 rows. The remaining holder is the address stored in
scalar slot 3; its getter instead reads balance mapping base **5**, returning
`5000000000000000000000000000`. Scalar slot 4 defines another special address,
currently zero. Read-only overrides setting either scalar to an ordinary holder
switch its getter to the stored-balance path. Twenty-one independently queried
ordinary holders match the formula; the formula plus special holder's raw storage
matches all 3,002 reference rows. The raw formula's one exception is preserved
in [characterization evidence](evidence/computed-address-characterization.json);
the [special getter trace](evidence/computed-special-inspection.json) identifies
its storage read. This is a diagnosis, not a promoted production layout.

Supporting it requires explicitly reviewed computed semantics, guards for both
special-address dependencies, and a way to emit observed holders with no balance
write. A direct mapping configuration would miss almost all of these holders.
The other pending deployment candidate, rank 26, is an immutable clone and still
needs implementation binding and initialization review. Remaining top-50 layouts,
global holder bootstrap and shared-default changes also remain open.

## Reproduction

All executable tooling is Rust. Use new output directories to retain failures.

```sh
cargo run --locked -p erc20-balances-storage-tools -- audit-rpc \
  --layouts erc20/balances-storage/tests/fixtures/bsc-deployment-layouts.json \
  --start 122288330 --blocks 256 --workers 2 \
  --output erc20/balances-storage/out/my-deployment-audit

cargo run --locked -p erc20-balances-storage-tools -- capture-blocks \
  --start 122288330 --blocks 128 \
  --output erc20/balances-storage/out/my-deployment-blocks

cargo run --locked -p erc20-balances-storage-tools -- holder-coverage \
  --ranking erc20/balances-storage/out/top50-1024/report.json \
  --block-dir erc20/balances-storage/out/my-deployment-blocks \
  --layouts erc20/balances-storage/tests/fixtures/bsc-deployment-layouts.json \
  --output erc20/balances-storage/out/my-deployment-holders
```

Use `--endpoint` to select a working Substreams provider for the audit. For the
deployment-only replay, supply a JSON layout array containing only the fixture's
entry with a `deployment` field. The ranking and original reference capture must
remain together with matching digests.
