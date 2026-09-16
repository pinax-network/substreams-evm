# Historical multi-map ERC-20 qualification

This records the earlier diagnostic-map version. The current single-map package
and its external layout configuration are documented in [qualification](qualification.md).


The current implementation lives in `erc20/balances-storage`, with
`erc20/balances` v0.3.4 as its reference. Both public modules reuse the existing
`evm.balances.v1` protobuf definitions and Rust types. The diagnostics schema is
separate. Native balance projection and the prototype's database/sink outputs
have been removed from this package.

All comparison, RPC auditing and discovery tools and tests are native Rust.
Earlier aggregator/native evidence is retained separately in
[legacy qualification](legacy-qualification.md); it is not evidence of complete
ERC-20 reference parity.

## Current package, 2026-09-16

`spkg/erc20-balances-storage-v0.1.0.spkg` SHA-256:
`78e7ce9aa2815d38a481869863d2c432fabcba7ed56319ff9a0ac303b3de4b71`.
WASM SHA-256:
`e90ae2f27b6cf64ec711572be5413e24a822d40da218556ad4c25794c4adf039`.
Import inspection found only `env.output`, `env.register_panic` and
`env.skip_empty_output`; no RPC host imports. Package inspection confirms the
same `map_events` / `map_balance_changes` output type names as the reference.
The public protobuf file is imported directly from the same shared directory,
and a Rust wire-format test covers the shared generated types and explicit zero.

## Comparison against erc20/balances

Over finalized BSC blocks **122260950–122261013**, the new `map_events` output
matched **all 1,414 shared WBNB updates**, with zero balance disagreements.
All 20 independent final-block `balanceOf` samples passed. The comparison ledger
performed 17,633 comparisons after storage-derived updates; 19,863 seed-only
comparisons are counted separately and are not independent validation.

The reference emitted **17,701 additional rows**, including balances for
**621 other token contracts** and unchanged WBNB participants. No candidate row
was missing from the reference. The result is **`coverage_gap` with exit code 1**,
not full parity. The report retains this distinction:
[64-block ERC-20 comparison](evidence/rust-erc20-reference-64.json).

The reference artifact is `erc20-balances-v0.3.4.spkg`, SHA-256
`8aaa03b551b9d67ce1ea9aa0dae4310eda1807f2bc161bd53b17f82de9d92543`.
Its public `Events` schema has no block hash; the runner checks exact output
heights and stable RPC boundary headers around finalized captures. This trusts
provider finality rather than independently verifying consensus.

## Direct RPC audit

The [64-block Rust audit](evidence/rust-erc20-rpc-64.json) checked **2,828 WBNB
balances**, covering every old/new value for all 1,414 emitted updates:
**zero mismatches**, including 594 explicit zeros. Every RPC request uses an
EIP-1898 block hash and `requireCanonical: true`. Public `map_events` output was
also checked against the diagnostic projection on every block. Runtime identity
is verified before the first block and after the last.

The [disjoint 128-block Rust audit](evidence/rust-erc20-rpc-128.json), blocks
122261100–122261227, adds **4,454 checks with zero mismatches**, including 1,040
explicit zeros. Together the two windows cover **7,282 WBNB old/new checks over
192 blocks**, and public event/diagnostic equality on every block. Both ran the
package hash above, using one worker and batches of 25.

## Rust discovery replay

The [16-block Rust probe](evidence/rust-erc20-discovery-16.json) repeated the
earlier discovery window: 239 token-like contracts, 522 candidate layouts,
6,380 before/after candidate-value checks and 215 contracts with a matching
candidate. One contract still has multiple matching layouts. **Zero adapters
were promoted**. Every layout and token statistic matched the retained
[detailed discovery report](evidence/erc20-discovery-16.json); the new summary
records the Rust run's package/check hashes and timing. Raw checks remain in
the local run directory. Matching a short window does not qualify an adapter.

## Tests and limits

- 24 mapper/persistence/protobuf tests and 25 native tool tests pass on Rust 1.88.
- The complete workspace's 66 library/binary tests and WASM target check pass.
- Targeted Clippy passes with warnings denied; the package builds and packs.
- All six prototype Python scripts/tests were replaced by the Rust tools crate.
- An expanded workspace run also attempted unrelated generated documentation
  examples and found a preexisting non-Rust example in `proto/src/pb/uniswap.v3.rs`
  (Swap). CI retains library tests and adds binary tests; the new crates' own
  documentation tests pass. This unrelated generated example was left unchanged.

Full ERC-20 row coverage, complete holder bootstrap, proxy and computed-balance
semantics, longer live sink runs, restart/reorg handling and end-to-end Token API
queries remain unqualified. No production sink or aggregator was rewired.
