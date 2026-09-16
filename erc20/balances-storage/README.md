# erc20/balances-storage

A single RPC-free `map_events` reads Firehose Extended blocks and emits
**`evm.balances.v1.Events`**, using the exact shared protobuf and Rust types from
[`erc20/balances`](../balances/) and [`proto/v1/balances.proto`](../../proto/v1/balances.proto).
There are no intermediate map modules, imported map dependencies, custom protobufs
or generated bindings in this package. Only `map_events` creates an output cache.

## Configurable verified layouts

No token address, balance slot or runtime is built into the mapper. Supply a JSON
array in the `map_events` parameter. The default is `[]`, which emits no balances.
Each entry describes a **previously qualified balance mapping**, optionally
with a reviewed zero-word fallback or proxy with its implementation pinned:

| Field | Format | Meaning |
| --- | --- | --- |
| `contract` | 20-byte `0x` hex | Token contract |
| `balance_slot` | 32-byte `0x` hex | Mapping base for `mapping(address => uint256)` balances |
| `code_hash` | 32-byte `0x` hex | Qualified runtime Keccak-256, checked by the Rust audit tools |
| `deployment` | Optional object | `block` and 32-byte `block_hash` pin first CREATE for a direct mapping or minimal proxy, without a zero-word fallback |
| `other_slots` | Optional array of 32-byte `0x` hex | Explicitly qualified non-balance scalar slots |
| `other_mapping_slots` | Optional array of 32-byte `0x` hex | Explicitly qualified non-balance mapping bases, including nested mappings |
| `other_mapping_words` | Optional object mapping 32-byte `0x` bases to counts 1–32 | Reviewed non-balance mappings with multiword values, such as governance checkpoint structs |
| `zero_balance` | Optional object | `value` (32-byte `0x` hex uint256) replaces a zero mapping word; `storage_slot` (32 bytes) identifies its scalar dependency, omitted only for a runtime constant |
| `proxy` | Optional object | `implementation_slot` (32 bytes), `implementation` (20 bytes), and implementation `code_hash` (32 bytes), all `0x` hex |
| `beacon_proxy` | Optional object, mutually exclusive with `proxy` | `beacon_slot`, `beacon`, `beacon_code_hash`, `implementation_slot`, `implementation`, `implementation_code_hash`; addresses are 20 bytes, slots/hashes 32 bytes |
| `minimal_proxy` | Optional object, mutually exclusive with other proxy kinds | `implementation` (20 bytes) and its `code_hash` (32 bytes); the token runtime hash must bind the exact standard 45-byte ERC-1167 forwarder |

The caller must establish that the configured projection equals `balanceOf` for the
pinned runtime. Matching a few samples or finding a mapping-shaped write alone
is insufficient. The mapper cannot infer preexisting code identity from a block
without code changes: independently qualify the starting runtime before using
it outside the audit tools. Persisted code changes to configured contracts
fail, including changes back to the expected runtime, except a qualified first
CREATE described below. For a configured proxy,
the tools also check the implementation storage word and implementation runtime
at both boundaries. The map rejects **every persisted implementation-slot write**
(including an upgrade and upgrade back) and all implementation code changes.
The implementation slot cannot appear in an ignore list. A proxy runtime hash
alone is insufficient; the implementation's balance semantics must also be reviewed.
For a reviewed beacon proxy, `beacon_slot` belongs to the token and
`implementation_slot` belongs to the separate beacon contract. The tools bind
both pointers, both dependency runtimes, and the beacon's `implementation()`
return value. The mapper rejects changes to either pointer or dependency code,
including a beacon upgrade with no token writes and an upgrade followed by a
restore. The caller must review that the beacon getter reads this scalar directly;
arbitrary computed or nested beacon resolvers are unsupported. Diamond proxies,
rebasing balances and other computed balances also remain unsupported.

For `minimal_proxy`, the implementation address is embedded in the exact
[ERC-1167 runtime](https://eips.ethereum.org/EIPS/eip-1167), rather than a storage
pointer. Configuration binds the runtime hash to that forwarder and target. The
audit checks the implementation code, and ingestion rejects implementation code
changes even without token writes. Other clone bytecode variants require separate
qualification; an immutable forwarding address does not make its target code or
balance semantics automatically safe.

With `deployment`, the tools verify empty code and nonce zero immediately before
the pinned block, and the expected runtime at deployment and audit boundaries.
The mapper requires exactly one persisted code creation in a successful CREATE,
checks its runtime bytes and execution ordinals, and rejects earlier storage/code
activity, nonzero initial storage, and later code changes. Constructor writes
before the code-change ordinal are included. Deployment support currently applies
only to direct mappings and standard minimal proxies; storage-pointer proxy and
fallback initialization remain unsupported.
The native holder replay initializes the observed holder set to zero **at the
validated CREATE**, then applies that block's writes. It does not call `balanceOf`
before deployment or treat an unavailable response as zero. This is a bounded
test baseline, not a complete global holder enumeration.

With `zero_balance`, nonzero mapping words pass through unchanged. Zero words
emit the explicitly pinned fallback. Raw words still drive continuity checks;
projection happens only at output. For a storage-dependent fallback, the tools
verify its value at both boundaries, and the mapper rejects **every persisted
dependency-slot write**, including change-and-restore and holder-silent changes.
The dependency cannot be ignored. Such changes need requalification and a rebuild
of affected holder state; they are not implemented as silent global updates.

Verified Keccak preimages identify holder keys. Transaction/call/log addresses
are fallback candidates, accepted only when their mapping hash matches exactly.
Persisted writes are ordered by execution ordinal; reverted execution cannot
emit balances. Direct layouts preserve zero and full uint256 values; fallback
layouts apply their explicit zero rule. Null holder addresses are excluded, as in
the RPC reference. Unknown
writes for a configured token cause an error unless they belong to an explicitly
configured other slot/mapping. Unconfigured contracts emit no rows.

Identical protobuf format does not mean complete ERC-20 coverage: unchanged
participants and holders with no observed write remain unknown. The reference
also queries approval participants, senders, token contracts and special events.
Missing data never becomes zero. A full holder dataset still needs a verified
bootstrap and additional qualified token semantics.

## Build and Rust tests

```sh
make -C erc20/balances-storage test
make -C erc20/balances-storage pack
```

Output: `spkg/erc20-balances-storage-v0.1.0.spkg`. No Buf generation is needed here;
the public schema is already maintained by the shared `proto` crate.

Tests call extraction functions directly in Rust. They cover two arbitrary token
addresses with different mapping bases (including a full-width 256-bit base),
zero/max values, allowances, malformed configuration, code changes, reverted and
failed transactions, real captured blocks, RPC failures and schema compatibility.
The native tools' HTTP/SQLite dependencies do not enter the mapper's WASM.

## Compare and audit

The regression layout file below is an **explicit test input**, containing the
previously qualified BSC WBNB layout. It is not a default or built-in token list.
Replace it with your qualified layouts. Current live qualification uses BSC.

```sh
cargo run --locked -p erc20-balances-storage-tools -- audit-rpc \
  --layouts erc20/balances-storage/tests/fixtures/verified-layouts.json \
  --start 122260950 --blocks 64 \
  --output erc20/balances-storage/out/single-map-audit

cargo run --locked -p erc20-balances-storage-tools -- compare \
  --layouts erc20/balances-storage/tests/fixtures/verified-layouts.json \
  --start 122260950 --blocks 64 \
  --output erc20/balances-storage/out/single-map-comparison
```

Both commands capture only `map_events`. The comparison reference is
`erc20-balances-v0.3.4.spkg`. Runtime identity is verified at both range boundaries
for every configured token. `audit-rpc` checks **every emitted end-of-block
balance** with EIP-1898 `blockHash` / `requireCanonical: true`. The shared Events
schema has no old values or source hash; old/new extraction ordering is tested
natively, while live audit binds finalized capture heights to RPC headers and
rechecks their continuity/stability. It trusts provider finality, not independent
consensus proofs.

All observed rows, value differences and coverage gaps are retained in SQLite;
reference-only holders never seed candidate state. Raw captures, RPC checks and
JSON reports remain in each new output directory, including partial failures.
Comparison exits zero only for bounded value and row-coverage parity. RPC audit
exits zero only when all emitted values pass and at least one was checked.

Use `--endpoint` for the Substreams endpoint. RPC uses `RPC_URL` (default
`https://bsc.rpc.pinax.network`) with optional `RPC_API_KEY` or
`SUBSTREAMS_API_KEY`. The Substreams CLI uses its normal authentication.
Credentials remain in environment variables, not report fields.

## Native layout discovery

Discovery operates directly on captured `sf.ethereum.type.v2.Block` protobuf
files in the Rust tool. It is excluded from WASM, has no map handler and creates
no Substreams cache:

```sh
cargo run --locked -p erc20-balances-storage-tools -- probe-erc20 \
  --block-file erc20/balances-storage/tests/fixtures/bsc-122260950.pb \
  --output erc20/balances-storage/out/native-discovery
```

Repeat `--block-file` for consecutive blocks. These must be full Extended block
fixtures, not JSON-RPC blocks. Hypotheses are checked with historical `balanceOf`
and remain diagnostics; no layout is automatically promoted into parameters.
See [qualification](docs/qualification.md) and [ERC-20 expansion](docs/erc20-expansion.md).

## Test the busiest tokens from the RPC stream

The Rust tools can select tokens from actual `erc20/balances` output and test
their storage on earlier/later active block samples. They create no additional
Substreams modules or caches. See the [first top-ten results](docs/top-token-parity.md).

```sh
cargo run --locked -p erc20-balances-storage-tools -- rank-tokens \
  --blocks 512 --top 10 --output erc20/balances-storage/out/ranking

cargo run --locked -p erc20-balances-storage-tools -- capture-blocks \
  --ranking erc20/balances-storage/out/ranking/report.json \
  --samples-per-token 8 --output erc20/balances-storage/out/active-blocks

cargo run --locked -p erc20-balances-storage-tools -- test-ranked \
  --ranking erc20/balances-storage/out/ranking/report.json \
  --block-dir erc20/balances-storage/out/active-blocks \
  --layouts erc20/balances-storage/tests/fixtures/verified-layouts.json \
  --output erc20/balances-storage/out/ranked-parity
```

`rank-tokens` defaults to a window just before the finalized BSC head; `--start`
makes it reproducible. Ranking counts emitted balance rows, not market cap or
transfer count. `capture-blocks` uses the `firecore` CLI with gzip and canonical
block IDs; both it and `substreams` must be on PATH. Configure their endpoints
with `--endpoint`; RPC credentials follow the environment variables above.

The test selects a mapping hypothesis from the earlier half of each token's
sampled active blocks, then freezes it for the later half. It checks every
observed candidate value before and after the block with hash-pinned `balanceOf`.
Mismatches also trigger a hash-pinned `eth_getStorageAt` check. The actual native
mapper is replayed with caller-supplied reviewed layouts when available; other
tokens use **unqualified diagnostic inputs with empty ignore lists**. Unknown
writes remain mapper errors, never automatically become ignored storage.

Each report separates hypothesis values from strict mapper output and missing
reference rows. No state is carried across missing sampled blocks, no RPC row
seeds the mapper, and no hypothesis is promoted into configuration. A completed
investigation with gaps or mismatches exits nonzero; `bounded_parity` alone exits
zero and still does not establish universal token semantics. Preserve the entire
output directory: JSONL observations and checks are referenced by their SHA-256
digests in the report. Repeat `--block-dir` to add more captured samples.

## Reviewed candidates and holder state

`tests/fixtures/bsc-reviewed-layouts.json` explicitly configures BSC USDT, BTCB,
USDC (pinned implementation), and the existing WBNB control. The file is not a
default. See [expanded qualification and holder coverage](docs/holder-coverage.md)
for source/runtime evidence, the zero-word mismatch explanation and live checks.

The [next BNB qualification](docs/next-bnb-candidates.md) adds ETH, BUSD, CAKE and
USD1 in `tests/fixtures/bsc-expanded-layouts.json`. It includes multiword governance
storage, an ABI decoding compatibility fix, and explicit diagnosis of calls before
contract deployment. The new fixture is also caller-supplied, never a default.

The [fallback and vUSDT follow-up](docs/fallback-balances.md) expands the explicit
test configuration to 14 tokens in `tests/fixtures/bsc-fallback-layouts.json`.
It fixes all seven recorded fallback-value mismatches and qualifies vUSDT's
custom proxy implementation using Venus's published deployment artifact.

The [beacon and zero-path follow-up](docs/beacon-and-zero-paths.md) adds KII,
BNC4, WCOL and three further fallback profiles to the explicit
`tests/fixtures/bsc-beacon-layouts.json` test input. `inspect-ranked` probes zero,
1, 123 and uint256 max on candidate mappings. This catches fallback paths absent
from ordinary transfer samples; passing these controls never automatically
qualifies a layout.

The [deployment follow-up](docs/deployment-holder-coverage.md) qualifies a direct
token's first mint. The [clone follow-up](docs/clone-holder-coverage.md) adds a
source-verified minimal-proxy implementation and replays both deployments with
`tests/fixtures/bsc-clone-layouts.json` (22 explicit test layouts). Three more
individually checked tokens with that same implementation are included in
`tests/fixtures/bsc-clone-family-layouts.json` (25 layouts).

```sh
cargo run --locked -p erc20-balances-storage-tools -- inspect-ranked \
  --survey erc20/balances-storage/out/ranked-parity/report.json \
  --output erc20/balances-storage/out/zero-path-review
```

The original survey and checks must remain together, with their recorded digest.
The probe selects post-block holders and can revisit earlier unresolved RPC
responses using the current decoder. Contracts without a mapping candidate stay
explicitly untested. Use repeated `--contract` filters to narrow a sweep.

`recheck-rpc --checks <rpc-checks.jsonl> --output <new-directory>` diagnoses prior
unresolved checks without overwriting them. `test-ranked` accepts repeated
`--contract` filters for focused retests of selected ranked tokens and records the
selection in its report. Token RPC output follows the reference ABI decoder: a
complete leading uint256 word is required and trailing return bytes are accepted.

The `holder-coverage` Rust command compares a cold consumer with one initialized
from a **test-only historical RPC checkpoint**. It requires consecutive captured
blocks and a ranking that includes them:

```sh
cargo run --locked -p erc20-balances-storage-tools -- holder-coverage \
  --ranking erc20/balances-storage/out/ranking/report.json \
  --block-dir erc20/balances-storage/out/consecutive-blocks \
  --layouts erc20/balances-storage/tests/fixtures/bsc-reviewed-layouts.json \
  --output erc20/balances-storage/out/holder-state-test
```

It records the setup RPC reads and checkpoint hash, then applies actual map
outputs without consulting RPC for balances during processing. Reference amounts
are only compared, never inserted into consumer state. The test checkpoint covers
only addresses queried in that bounded reference window; it is not a complete
global holder snapshot or a deployed sink. Full cold-start coverage still needs
a trusted checkpoint or complete history, plus a consumer that retains updates.

`inspect-balance --contract ... --address ... --balance-slot ... --block ...
--output ...` diagnoses a mapping at a canonical block using `debug_traceCall`
and read-only state overrides for zero, 1, 123 and uint256 max. Optional
`--source <Sourcify-v2-response.json>` verifies that the source record's runtime
matches the historical runtime before saving its layout/provenance. Overrides
simulate `eth_call`; no transaction is sent.

For a project-published deployment artifact, use `--deployment-artifact <json>`
with `--artifact-url <immutable-source-url>` instead of `--source`. The tool binds
the artifact address and runtime to historical RPC and checks each literal
source's hash. `--zero-dependency-slot <32-byte-hex>` also probes the scalar
dependency with zero, 17 and uint256 max while controlling the holder word.
These controls are evidence for review, not automatic layout qualification.
