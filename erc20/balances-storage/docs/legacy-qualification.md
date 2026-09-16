# Historical aggregator prototype qualification — 2026-09-16

This document records the superseded `evm-balances-storage` prototype before its
move to `erc20/balances-storage`. References to native balances, `db_out`, the old
comparator and Python describe that historical run only. Current behavior and
Rust validation are documented in [qualification](qualification.md).


The first prototype reads native BNB and WBNB balances without RPC. It decoded
all WBNB writes in two disjoint finalized BSC windows. Comparison against the
live package version exposed one repeatable native-balance discrepancy in the
reference. **Neither run is recorded as full parity.**

| Measure | 122260950–122261013 | 122261100–122261227 |
| --- | ---: | ---: |
| Blocks | 64 | 128 |
| Native updates | 6,269 | 12,274 |
| WBNB updates | 1,414 | 2,227 |
| WBNB storage changes, including allowances | 2,632 | 4,189 |
| Unresolved WBNB slots | 0 | 0 |
| Distinct keys updated by the new mapper | 3,840 | 5,949 |
| Candidate keys never observed by reference | 0 | 0 |
| Block/key comparisons after a storage-derived observation | 137,256 | 445,173 |
| Reference-only bootstrap observations | 4,064 | 5,853 |
| Balance disagreements | 64 | 128 |
| Disagreements independently checked by historical RPC | 64 / 64 | 128 / 128 |
| Disputed values agreeing with new mapper | 64 / 64 | 128 / 128 |
| Separate native/WBNB RPC samples passing | 20 / 20 | 20 / 20 |

The reports are [64 blocks, audited](evidence/64-audited.json) and
[128 blocks, audited](evidence/128-audited.json). The [initial failing report](evidence/64-initial.json)
is retained too. Its earlier `snapshot_comparisons` count included reference-only
seeded accounts; subsequent reports separate those as `seed_only_comparisons`.
Seeds are not independent evidence for this mapper. Disjoint-window distinct-key
counts must not be added as unique holders.

## Why the reference differs

Every disagreement is native BNB at
`0xfffffffffffffffffffffffffffffffffffffffe`. For captured block **122260950**:

- The last transaction fee credit, ordinal **3237**, leaves
  **1675456549641341 wei** at that address.
- The block-level change, ordinal **3244**, resets that balance to **0**.
- This package and historical `eth_getBalance` both return **0**.
- `evm-balances-v0.3.3.spkg` returns **1675456549641341**.

The [v0.3.3 native source](https://github.com/pinax-network/substreams-evm/blob/d8a731d/native-balances/src/lib.rs)
processes block changes before transaction changes, overwriting the final reset
with the earlier fee credit. Its RPC account discovery uses transaction/call
addresses, rather than every address in balance changes. The system fee address
therefore retains the overwritten state-derived value. The current main-branch
native module has a different implementation; this finding specifically concerns
the v0.3.3 artifact used as the comparison reference.

BSC's [fee-distribution implementation](https://github.com/bnb-chain/bsc/blob/c5533ab5b7244dc474add10740834417a2c605d7/consensus/parlia/parlia.go#L2006)
also resets the system account before distributing its accumulated balance.
The captured Extended block, its final reset and the block-pinned RPC values
provide the direct evidence for this case. No RPC service failure is inferred.

The full captured block is committed as a regression fixture. **All 100 output
balances (82 native, 18 WBNB) were independently checked at that exact block**;
the offline test compares against these recorded RPC values. See
[`bsc-122260950.json`](../tests/fixtures/bsc-122260950.json) for provenance,
hashes and the oracle, and `bsc-122260950.pb` for the complete block.

The comparator reports `reference_disagreement` and returns nonzero even when
RPC supports all candidate values. It does not erase, exclude or zero-fill the
disputed address.

## Performance observations

Historical replay, including CLI startup, over the same endpoint and blocks:

| Run | Storage mapper | Existing package |
| --- | ---: | ---: |
| First 64-block capture | 35.76 blocks/s | 2.12 blocks/s |
| 64-block repeat with RPC discrepancy auditing | 36.68 blocks/s | 4.63 blocks/s |
| Disjoint 128-block capture | 63.98 blocks/s | 2.08 blocks/s |

The 128-block Substreams log reports **0 already cached blocks** for the mapper;
the replay transferred about 1,002 KiB. Cached RPC responses and startup overhead
can affect the reference timing, as the repeat demonstrates. The mapper emits
only native BNB and WBNB, while the reference executes its broader token pipeline.
These are encouraging bounded measurements, not an equal-workload benchmark,
sustained catch-up guarantee or production capacity estimate.

RPC auditing time is excluded from both stream timings. Package SHA-256 values,
first/last block identity and raw execution summaries are retained in `evidence/`.
The original qualification artifacts predate adding the README and tightening
the code-identity guard to reject every persisted WBNB code transition. The
[final package replay](evidence/final-replay.json) repeated both windows and
produced **identical decoded output for all 192 blocks**. That record contains
the final package/WASM hashes and imports; the earlier reports remain intact.

## Checks and limits

The stronger [direct RPC audit over 64 blocks](evidence/rpc-exhaustive-64.json)
checks **all 7,683 emitted updates twice**, at the parent and current block:
15,366 checks (12,538 native / 2,828 WBNB), including 1,634 zero values, with
**zero mismatches**. Every request is pinned with EIP-1898 blockHash and
requireCanonical=true. This is independent of the old package and its bootstrap
seeds. Full per-observation evidence remains in the local audit output; its
SHA-256 is in the committed summary.

An initial [128-block exhaustive audit](evidence/rpc-exhaustive-128-incomplete.json)
stopped on an RPC transport error after 2,704 checks across 12 completed blocks.
It found zero mismatches in those checks, but is explicitly retained as
**incomplete**, not a pass.

The [conservative rerun over all 128 blocks](evidence/rpc-exhaustive-128.json)
completed using one worker and batches of 25: **29,002 checks, zero mismatches**
(24,548 native / 4,454 WBNB), including 2,960 zero values. Across both disjoint
windows this is **44,368 before/after checks over 192 blocks**, with no RPC balance
disagreements. Defaults now use this conservative batching configuration.
The [final PR package replay](evidence/pr-final-rpc-replay.json), including the
new ERC-20 discovery module, produces exactly the same balances and metadata
over all 192 exhaustively audited blocks. It records the final package/WASM
hashes and verifies that the WASM still has no RPC imports.

- 23 Rust tests and 20 Python comparison/audit/discovery tests passed.
- Targeted Clippy with warnings denied passed.
- Repository-wide validation passed: all 40 Rust library tests and the entire
  workspace's `wasm32-unknown-unknown` check, including the final code guard.
- WASM and both Substreams packages built successfully.
- WASM inspection found only `env.output`, `env.register_panic` and
  `env.skip_empty_output` imports: no RPC host imports.
- `db_out` replayed two real blocks successfully, emitting 101 and 119 rows
  including block markers.
- The ClickHouse comparison schema created successfully in ClickHouse local
  25.8.1.3064; an explicit zero balance inserted and read back successfully.

Not yet qualified: complete bootstrap, other tokens/networks, additional producer
versions, reversible/live block handling, a sustained isolated SQL sink, restart
continuity under sink failure, or end-to-end Token API queries. The full-block
fixture and failed EIP-7702 transactions extend regression coverage, but are not
an exhaustive EVM lifecycle matrix. The comparator trusts the RPC provider's
finalized header; it does not independently verify consensus or state proofs.
