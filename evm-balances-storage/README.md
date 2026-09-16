# evm-balances-storage

Experimental BSC native BNB and WBNB balances derived from Firehose Extended
state changes. The map and database modules make **no RPC calls**. This package
lives alongside `evm-balances` so both can run over the same finalized blocks.

## Current scope

| Asset | Balance source | Coverage |
| --- | --- | --- |
| Native BNB | Last persisted `BalanceChange.new_value` by execution ordinal | Changed accounts, including gas, system and block-level changes |
| WBNB | Persisted `StorageChange.new_value` at `keccak256(pad(holder) ++ pad(3))` | Changed holders of the pinned BSC WBNB contract |

WBNB is `0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c`. Its qualified runtime
Keccak-256 is `b7d84205eaaf83ce7b3940c6beaad6d22790255e34a9a2b486aa8cdfff118fe6`.
The comparison runner verifies the exact runtime at both ends of every range.
The mapper rejects any persisted in-range WBNB code change, including a change
back to the expected runtime. Start after deployment. A standalone sink must
qualify the starting code itself; the stateless mapper cannot infer preexisting
code identity from a block that contains no code change.

This is a **changed-key projection**, not a complete holder database. An account
that never changes in the selected range remains unknown. Neither missing
holders nor unrecognized storage slots become zero balances. A complete Token
API replacement needs a verified checkpoint or a qualified replay from
deployment/genesis, plus continuity from that checkpoint. Other ERC-20 tokens,
proxy upgrades, rebasing tokens, arbitrary EVM execution and other networks are
outside this first adapter.

## How it works

`map_balances` accepts Extended producer versions 3–5, validates block identity
and transaction status, and collects persisted state changes. Live qualification
currently covers BSC producer version 5. Successful transactions retain changes
from non-reverted calls; failed transactions retain gas effects and persisted
sender/authorization nonce changes. Nonces help distinguish execution that
survived failure; they are not used to infer a balance. The persistence rules
are adapted from [`substreams-evm-state` at 29251362](https://github.com/pinax-network/substreams-evm-state/blob/29251362c8651fd6d5dbe7b3d69820c29ae172c5/src/persist.rs).

Verified Keccak preimages reveal WBNB holder addresses even without a Transfer
event. Addresses found in transactions, calls and WBNB logs are fallback
candidates, accepted only when their slot hash matches exactly. Allowances are
recognized separately using both preimages of the nested mapping at slot 4.
An unrecognized WBNB write is emitted as an explicit unresolved slot, and
`db_out` refuses to emit an incomplete database update.

Per holder, the output retains the value before the first persisted change and
the value after the last, including explicit zero and exact uint256 amounts.
Ambiguous ordinals and discontinuous values fail. Block-level changes participate
in this ordering, including BSC's fee-distribution reset at the end of a block.

`db_out` uses the existing `native_balances`, `erc20_balances` and `blocks`
interfaces. It emits a block marker even when no balance changes. The included
ClickHouse schema retains history for comparison in a **dedicated database**;
it does not implement reorg rollback. Use finalized blocks only. Range comparison
rejects gaps, duplicate blocks and parent/hash mismatches. A restart or overlapping
replay can be deduplicated with `FINAL`; a different hash at an already accepted
height requires investigation rather than silently selecting a fork.

## Build and test

From the repository root, with Rust's `wasm32-unknown-unknown` target installed:

```sh
make -C evm-balances-storage test
make -C evm-balances-storage pack
make -C evm-balances-storage clickhouse
```

This builds `spkg/evm-balances-storage-v0.1.0.spkg` and the separate
`spkg/evm-clickhouse-balances-storage-v0.1.0.spkg`. Generated protobuf bindings
are committed; regenerate them with `make -C evm-balances-storage protogen`
after editing `balances.proto` (requires Substreams CLI and Buf).

Offline tests cover failure persistence, EIP-7702, reverted calls, slot
discovery, allowances, zero/max balances, input ambiguity, block continuity,
comparison coverage and discrepancy auditing. Captured transaction fixtures
have provenance in `tests/fixtures/bsc-failed-setcode.json`.

## Compare against the existing package

For a direct RPC baseline covering **every emitted balance**, run:

```sh
python3 evm-balances-storage/scripts/audit_rpc.py \
  --start 122260950 --blocks 64 \
  --output evm-balances-storage/out/my-rpc-audit \
  --endpoint bsc.substreams.pinax.network:443
```

This checks every `old_amount` at the parent block and every `amount` at the
current block using `eth_getBalance` or `eth_call balanceOf`. All calls use
EIP-1898 `blockHash` with `requireCanonical: true`; there is no height/latest
fallback. RPC batches match by response ID, reject missing/error/malformed
results, and require a complete uint256 return word for `balanceOf`. Each check
is retained in `rpc-checks.jsonl`; incomplete runs and mismatches fail the audit.
This validates emitted updates, not holders absent from both data sources.

The existing package comparison is a separate check:

```sh
python3 evm-balances-storage/scripts/compare.py \
  --start 122260950 --blocks 64 \
  --output evm-balances-storage/out/my-comparison \
  --endpoint bsc.substreams.pinax.network:443
```

Choose an unused output directory. Substreams CLI uses its normal authentication;
the audit uses `RPC_URL` (default `https://bsc.rpc.pinax.network`) and optionally
`RPC_API_KEY` or `SUBSTREAMS_API_KEY`. Python uses only its standard library.
RPC is used by this validation tool and by the old package, never by the new
ingestion modules.

The runner checks chain ID 56, RPC finality, WBNB runtime identity and matching
first/last headers. It runs the new mapper and the repository's
`evm-balances-v0.3.3.spkg` over exactly the same range, then writes:

- `storage.jsonl`, `reference.jsonl` and their execution logs;
- `comparison.sqlite`, containing observed updates and every disagreement;
- `report.json`, including coverage counts, package hashes, timing, stratified
  independent RPC samples, and RPC audits of up to 256 disputed observations.

Each ledger carries observed values forward between blocks. First observations
available only in the old package seed otherwise unknown accounts and are
counted separately from comparisons backed by a storage-derived update. This
can detect a missed later update, but cannot prove the correctness of a holder
that is only seeded, never updated, or missing from both streams. Candidate
holders missing from the reference are reported as coverage gaps. The block
hash, rather than arrival time, defines the comparison boundary.

Exit code 0 means bounded parity. Unresolved slots, missing coverage, RPC audit
failures and disagreements return nonzero. `reference_disagreement` means all
audited disputed values matched the new mapper and the independent samples
passed; it **still returns nonzero** and retains every disagreement. Failed
captures remain in the output directory for investigation.

## Qualification and remaining work

An experimental `map_erc20_candidates` module now investigates storage layouts
across contracts emitting ERC-20-shaped Transfer logs. It emits mapping
**hypotheses**, and has no connection to `db_out`. The accompanying probe compares
their before/after values with block-hash-pinned `balanceOf` calls. See
[ERC-20 expansion](docs/erc20-expansion.md) for results and the next implementation
gates. This does not enable arbitrary tokens in the balance sink.

See [the recorded BSC comparison](docs/qualification.md) for current evidence.
Short historical replay timing is not sustained live throughput or a production
capacity claim. These checks do not validate a production ClickHouse sink.

Before considering cutover: resolve the reference discrepancy, define and
verify the bootstrap checkpoint, qualify additional token layouts explicitly,
run longer independent windows and a sustained isolated sink, and verify
restart/continuity and end-to-end query results. Keep the existing package as a
comparison source throughout that work.
