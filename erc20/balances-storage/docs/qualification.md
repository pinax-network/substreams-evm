# Single-map ERC-20 qualification

The current package has exactly one module, `map_events`, taking parameters and
Extended blocks directly and returning `evm.balances.v1.Events`. It has no custom
protobuf, generated `pb.rs`, Buf config or intermediate cache. The only protobuf
source is the same shared `balances.proto` used by `erc20/balances`.

There is no built-in token address, runtime or balance slot. JSON parameters
supply caller-qualified direct-mapping layouts, runtime hashes and explicitly
ignored non-balance storage. The default layout list is empty. WBNB appears only
as an explicit regression fixture/test configuration, not in production logic.

Runtime identity is checked by the Rust validation tools at both range
boundaries. The mapper rejects all code changes for configured contracts and
unresolved writes. Qualifying the configured storage semantics and starting
runtime remains a caller prerequisite; the stateless map cannot independently
recover preexisting code identity. Proxy and computed-balance layouts remain
outside this adapter.

## Tests

Rust regression tests cover arbitrary token addresses, multiple mapping bases
including a full-width 256-bit slot, exact shared event encoding, scalar and
nested non-balance mappings, corrupt configuration/preimages, explicit zero,
uint256 max, code changes, failed/reverted execution and captured BSC blocks.
The captured complete block still matches all 18 recorded ERC-20 RPC values when
its layout is passed explicitly.

Native discovery runs directly over captured Extended Block protobuf files and
is excluded from WASM. It is no longer a Substreams map or output cache.

## Evidence boundaries

The single-map live audit checks each emitted **end-of-block** balance against
canonical block-hash-pinned `balanceOf`. It no longer captures custom old/new
storage diagnostics. Events has no source hash, so the runner binds finalized
capture heights to RPC headers, validates continuity and rechecks stability.
This trusts provider finality, not independent consensus proofs.

The comparator checks the new and reference packages' `map_events` outputs and
reports every row-coverage gap. Only candidate-observed values enter its ledger;
reference-only rows never seed candidate state. Matching schema and sampled
values does not establish complete token/holder coverage.

Earlier [multi-map qualification](multi-map-qualification.md) and
[aggregator prototype qualification](legacy-qualification.md) are retained as
historical evidence. Their package hashes and before/after audit counts do not
qualify this new artifact. Current single-map results are recorded below.

## Final artifact and results — 2026-09-16

Package SHA-256:
`255730e1269615545b00f7c84e475fa04716f3325e7fa275ecde25184e920f0e`.
WASM SHA-256:
`9be227abf6f47afacee33ce156d3dc586c4de1b92dd944eb0585ce7bffe2705c`.
The packaged graph contains exactly `map_events`; its only non-runtime function
export is `map_events`. Host imports remain `env.output`, `env.register_panic`
and `env.skip_empty_output`, with no RPC imports.

The explicit regression configuration has SHA-256
`c24596b31fe5027fafcca5ddffc08456de7859649299d8cd11a748568357f1ea`.
These live runs use that WBNB test configuration, not a built-in address list.

| Run | Blocks | Emitted balances checked | Zero values | Mismatches |
| --- | ---: | ---: | ---: | ---: |
| [RPC audit 122260950–122261013](evidence/single-map-rpc-64.json) | 64 | 1,414 | 298 | 0 |
| [RPC audit 122261100–122261227](evidence/single-map-rpc-128.json) | 128 | 2,227 | 519 | 0 |

Together: **3,641 end-of-block RPC checks across 192 blocks, zero mismatches**.
These counts are intentionally different from the earlier two-sided audit: the
single public Events output contains final amounts only.

The [reference comparison](evidence/single-map-reference-64.json) matches all
1,414 shared updates and 20/20 final-block RPC samples. It records 17,701
reference-only rows and 621 reference token contracts without candidate updates.
There are zero value disagreements or candidate-only rows. The correct result
remains `coverage_gap`, exit code 1, not universal ERC-20 parity.

Two earlier runs are preserved as incomplete: [25-call batches](evidence/single-map-rpc-64-incomplete.json)
stopped after 341 checks/15 blocks, and [10-call batches](evidence/single-map-rpc-64-small-incomplete.json)
stopped after 117 checks/6 blocks. Both failed on a non-array response at a
single-call remainder. The Rust client now sends ordinary single requests for
such remainders and keeps strict response-ID/error/value validation. The complete
runs above used this correction, one worker and batches of up to 25.

The [native discovery probe](evidence/single-map-native-probe.json) reads the
captured block 122260950 directly: 26 token-like contracts, 42 layout hypotheses,
364 candidate-value checks and zero promoted adapters. No extra Substreams map
or cache is used.

All **68 workspace library/binary tests** pass on Rust 1.88, including 25 mapper,
persistence/configuration/schema tests and 26 native-tool tests. Workspace WASM
checking, targeted Clippy with warnings denied, and packing pass. The SDK's
macro-generated string-pointer ABI wrapper has a narrowly scoped Clippy
exception; extraction and native tools retain the normal checks.
