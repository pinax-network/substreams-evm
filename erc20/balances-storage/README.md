# erc20/balances-storage

Experimental RPC-free ERC-20 balance events from Firehose Extended storage
changes. The reference implementation is [`erc20/balances`](../balances/).
Both packages use the **same shared protobuf definitions and generated Rust
types** from [`proto/v1/balances.proto`](../../proto/v1/balances.proto).

| Module | Output protobuf | Meaning |
| --- | --- | --- |
| `map_events` | `evm.balances.v1.Events` | `balances[]`: optional bytes `contract` (field 1), bytes `address` (2), decimal string `amount` (3) |
| `map_balance_changes` | `evm.balances.v1.BalanceChanges` | `balance_changes[]`: optional bytes `contract` (1), bytes `address` (2) |
| `map_storage_changes` | `evm.balances.storage.v1.BlockBalances` | Separate diagnostics: block identity, first old/final new values, unresolved slots |
| `map_erc20_candidates` | `evm.balances.storage.v1.StorageCandidates` | Separate discovery hypotheses, never promoted into balance events |

Public `map_events` and `map_balance_changes` are wire-compatible with
`erc20/balances`: no custom balance wrapper, native balances, block markers or
DatabaseChanges. Storage diagnostics are kept in a separate module. The existing
aggregator/sink packages can consume the shared event type; no production wiring
is changed by this experiment.

## Coverage

**Identical protobuf schema does not yet mean identical coverage.** The current
qualified adapter covers changed holders of BSC WBNB
`0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c`, mapping slot 3. The RPC reference
also queries unchanged event participants, approvals, transaction senders, token
contracts and special token events across many tokens. Those additional rows
remain coverage gaps until their state and token semantics are supported.

The mapper preserves explicit zero and full uint256 values. It reads persisted
writes, verifies Keccak preimages, separates the nested allowance mapping at
slot 4, orders changes by execution ordinal and rejects discontinuity. Failed
or reverted execution cannot become a balance update. Nonces help determine
persistence boundaries; they do not encode token balances.

The qualified WBNB runtime Keccak-256 is
`b7d84205eaaf83ce7b3940c6beaad6d22790255e34a9a2b486aa8cdfff118fe6`.
The Rust validation tools verify exact runtime code before and after each range.
Any persisted WBNB code change or unresolved WBNB write prevents public event
output. A standalone consumer must qualify the starting runtime separately;
a stateless block does not reveal preexisting code identity. Use BSC, Extended
producer versions 3–5 (live fixtures cover version 5), and start after deployment.

There is no complete holder bootstrap yet. Missing holders or unknown slots are
never treated as zero. Proxy upgrades, computed/rebasing/reflection balances and
other token layouts require additional qualification. See
[ERC-20 expansion](docs/erc20-expansion.md).

## Build and test

All new implementation, tests and validation commands are Rust. The native tools
are a separate workspace crate so HTTP/SQLite dependencies stay out of WASM.
From the repository root:

```sh
make -C erc20/balances-storage test
make -C erc20/balances-storage pack
cargo run --locked -p erc20-balances-storage-tools -- --help
```

The package is `spkg/erc20-balances-storage-v0.1.0.spkg`. The shared public
protobuf is imported directly, never copied. To regenerate diagnostics after
editing `storage.proto`, use `make -C erc20/balances-storage protogen` (Substreams
CLI and Buf required).

Offline tests cover persisted/reverted execution, EIP-7702, allowances, corrupt
preimages, zero/max values, protobuf wire compatibility, real BSC fixtures,
comparison gaps, state continuity, RPC batch IDs, canonical hash binding and
partial failure evidence. The full-block fixture includes recorded RPC oracles;
the ERC-20 test checks all 18 WBNB updates and excludes native balances.

## Validate against RPC and the correct reference

Choose a new output directory for each run. Substreams CLI uses its normal
authentication. RPC uses `RPC_URL` (default `https://bsc.rpc.pinax.network`) and
optionally `RPC_API_KEY` or `SUBSTREAMS_API_KEY`. Credentials stay in environment
variables; transport errors do not print provider URLs or keys.

```sh
cargo run --locked -p erc20-balances-storage-tools -- audit-rpc \
  --start 122260950 --blocks 64 \
  --output erc20/balances-storage/out/rpc-64

cargo run --locked -p erc20-balances-storage-tools -- compare \
  --start 122260950 --blocks 64 \
  --output erc20/balances-storage/out/reference-64

cargo run --locked -p erc20-balances-storage-tools -- probe-erc20 \
  --start 122260950 --blocks 16 \
  --output erc20/balances-storage/out/discovery-16
```

Use `--endpoint` to select the Substreams endpoint. `audit-rpc` checks every
emitted old/new value with historical `balanceOf` at the canonical parent/current
block hash (EIP-1898 `requireCanonical: true`), with no height/latest fallback.
It also checks that public `map_events` equals the diagnostic projection on every
block. Empty, malformed, missing, duplicate-ID or error RPC results cannot pass.
Completed checks and counts survive a later transport failure.

`compare` captures **both packages' `map_events`**, using the repository's
`erc20-balances-v0.3.4.spkg` as its default reference. It records every observed
row in SQLite, compares shared values, counts reference-only/candidate-only rows
on every block, and audits disputed values at their original hashes. WBNB
reference-only initial values may seed a comparison ledger; seed-only checks are
counted separately and never claimed as verified storage updates. Unsupported
contracts remain explicit coverage gaps. Ordering of repeated balances is not
significant; keys and exact amounts are compared.

Raw outputs, logs, the SQLite comparison database and a JSON report are retained.
Exit code 0 for `compare` requires bounded value **and row-coverage parity**.
Coverage gaps, mismatches and incomplete captures return nonzero. All tools check
BSC chain ID and provider finality; the schema has no block hash, so reference
stream identity relies on exact capture heights and stable RPC boundary headers.
These are bounded historical checks, not a production cutover qualification.

See [qualification evidence](docs/qualification.md) for current results. The
[earlier aggregator prototype's evidence](docs/legacy-qualification.md) is kept
for provenance; its native balance results do not establish parity with the
ERC-20 reference.
