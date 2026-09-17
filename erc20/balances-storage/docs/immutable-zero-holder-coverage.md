# Immutable-zero mapping and BSC top-100 follow-up

The [100-profile fixture](../tests/fixtures/bsc-top100-layouts.json) adds the
remaining log-only candidate, rank 70 at
`0xe0150e5020e6326448502d0e3f03afd971a85fd8`. It uses an explicit
`immutable_zero_mapping` rule tied to the qualified direct deployment.
No address or rule is built into production parameters, which remain `[]`.
There is still one RPC-free `map_events` and the same shared Events protobuf.

## Why zero is justified here

The [previous creation review](evidence/log-only-creation-review.json) preserves
the complete runtime, captured CREATE and activity. The [qualification record](evidence/immutable-zero-qualification.json)
binds the new layout to that evidence and independently rechecks all 298
observed RPC participants.

1. At the canonical parent of block **121118665**, code is empty and nonce is
   zero. The successful captured CREATE installs the exact pinned runtime.
   Its 28-byte constructor only copies and returns the runtime; no balance or
   other storage is initialized.
2. The reviewed executable runtime has one `SSTORE`, at PC 856, writing a nested
   allowance mapping rooted at slot 1. It has no external call, delegate call,
   creation or self-destruct instruction. The balance getter reads mapping 0
   directly at PC 1050. The mapping is therefore empty for this reviewed
   deployment/runtime, rather than merely absent from the sampled writes.
3. Raw-word controls return zero, one, 123 and uint256 maximum when those values
   are independently supplied at the holder's mapping key. The getter is **not
   a constant-zero function**. These controls deliberately break the empty-state
   invariant and confirm the getter's actual storage dependency. Raw projection
   diagnostics continue to expose a nonzero word rather than masking it.
4. The unknown-selector fallback parses packed 51-byte records and emits
   Transfer logs without balance updates. The apparent `a9059cbb` transfer has
   no selector-table entry. Its block **122288749** emits 159 nonzero Transfer
   logs, while every one of the **298 reference participants** has RPC balance
   zero. Log amounts are not used to calculate balances.

This is bytecode/trace qualification; verified source was unavailable. The rule
must never be selected solely because a capture has no writes or sampled RPC
responses are zero.

## Production behavior and boundaries

The rule requires a pinned direct `deployment` and cannot combine with proxies,
zero-word fallbacks or address-derived balances. At the deployment block, the
existing CREATE checks validate runtime, hash, call provenance and ordinals.
Events before deployment or before CREATE are rejected.

Observed Transfer, Approval and the reference's OwnershipTransferred events
supply holders, transaction sender and token contract, using the shared ABI
decoders. Null addresses are excluded and duplicate participants are collapsed.
Unknown or malformed events fail. The result is a known zero from the qualified
invariant, not a default for missing state.

Every persisted balance-mapping write is rejected, including zero-to-zero,
nonzero no-ops and write/restore sequences. A separate optional no-op hook uses
the same persistence rules as ordinary changes; other layouts retain their
existing no-op filtering. Reverted calls and failed execution cannot emit rows
or trigger these persisted-storage checks. Explicitly reviewed allowance writes
remain permitted; unknown writes and code changes still fail.

The holder replay records `qualified_immutable_zero_mapping` baselines separately
from measured RPC checkpoints and address formulas. A token deployed inside the
replay initializes at the validated CREATE, never at the predeployment checkpoint.
Only observed holders are considered; this does not enumerate a global holder set.

## Validation

**171 workspace Rust library/binary tests pass**, with Clippy warnings denied,
workspace WASM compilation and targeted formatting. Eight new tests cover the
captured 298-holder comparison, explicit opt-in requirements, CREATE and event
timing, writes/no-ops/restores/system calls, code changes, allowances, unknown
storage, malformed/reverted/failed logs, participant selection and checkpoint
initialization timing.

The [WASM import scan](evidence/top100-wasm-imports.json) records only output,
logging, panic and empty-output imports, with **zero RPC imports**. The actual
new package also executes the historical creation block successfully with no
balance rows. [Creation delivery evidence](evidence/immutable-zero-creation-wasm.json)
requires the CLI's complete block receipt and a separate matching canonical
block-clock capture. An empty stream alone is not treated as proof.

Across **1,024 consecutive blocks, 122288006–122289029**, the actual new package
passes **96,269 emitted-balance comparisons**, including **13,714 zeros**, across
all 100 tokens, with **zero historical RPC mismatches**. The new candidate
contributes 298 checks. Results are retained in the [RPC audit](evidence/top100-rpc.json),
[per-token counts](evidence/top100-rpc-token-counts.json), and
[artifact identities](evidence/top100-artifacts.json).

The actual WASM output for this candidate has **exact row and value parity**
with the reference throughout all 1,024 blocks: 298 rows in its active block and
none in the other 1,023 blocks. The [comparison](evidence/immutable-zero-row-parity.json)
checks both complete row sets, not just the values of rows that happened to emit.

The [initialized-holder replay](evidence/top100-holder-coverage.json) passes
**157,421 observations**, including **4,364 carried-forward matches**, with zero
unknown or incorrect initialized values. The new candidate contributes 298
matches and has no cold unknowns. Its 298 invariant-derived baselines are
recorded separately from **51,186 stored-holder checkpoints**, which use
**102,372 balance/storage RPC reads**. Processing makes **zero balance RPC
calls** and 1,024 header verification calls. Reference observations never repair
retained state.

The other profiles retain **56,788 cold unknown observations**, including
**29,933 nonzero values**, with no incorrect known values. Adding the explicit
empty-mapping rule does not turn those unknowns into zero or claim raw event-row
parity for every token.

## Remaining scope

All original top-100 candidates have reviewed layout semantics for this
bounded campaign. This does not prove every code path or all BSC tokens. This
batch still rejected 4Stock membership-list writes pending their own
qualification. Its [first registration was captured](evidence/4stock-membership-follow-up.json)
at block **120607788**, call 193: slot 35 grows from zero to one, its first
address element is written, and mapping-36 field 4 is set. The initialization
fields and deployment also required review before that earlier block could pass.
The subsequent [holder-registration extension](holder-registration-coverage.md)
qualifies those paths in a separate fixture and audits the launch interval.
Cold stored holders still need a verified checkpoint or full
history, and the other layouts retain their documented row and dependency limits.
Overlapping prior BSC reports must not be summed as independent history.

The requested [network expansion](network-expansion.md) remains Ethereum, Base,
HyperEVM and Arc, each with separate token/runtime, Extended-block and holder
qualification. Initial RPC prerequisite probes are not token parity evidence.
