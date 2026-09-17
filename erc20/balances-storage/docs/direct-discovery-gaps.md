# MUSD and GAIX: source-led qualification of discovery gaps

MUSD (rank 386) and GAIX (rank 394) have direct `balanceOf` getters. Their
initial survey found matching slot-1 and slot-0 hypotheses, respectively, but
each had only **one nonzero holder** in the discovery sample. The unchanged
discovery threshold requires at least two. These were insufficient discovery
samples, not historical value mismatches.

The [original survey evidence](evidence/direct-gaps-discovery.json) is retained.
The separate [qualification](evidence/direct-gaps-qualified.json) binds verified
source to the historical runtime, checks each storage-field shape, traces the
getter and independently overrides the balance word with zero, one, 123 and
uint256 maximum. Both getters return the exact word for all four controls.
Fresh canonical runtime checks bind the parent and final interval boundaries.

MUSD's balance root is 1, allowances are rooted at 2 and its role records at 0
contain a nested membership mapping and an admin-role word. The profile permits
only that reviewed two-word role shape and scalar roots 3–5. GAIX uses balance
root 0, allowance root 1 and nonce root 7, with reviewed scalar/string roots
2–6 and 8–9. Unknown storage writes still fail.

Both profiles pass all **1,024 consecutive Extended blocks** from 122288006
through 122289029 with no unresolved writes. The production mapper needs no
change for either profile.

## Actual packaged output and holders

The [packaged-WASM RPC audit](evidence/direct-gaps-rpc.json) matches **35 emitted
balances**, including **20 zeros**. Complete canonical clocks cover blocks with
empty output. The [checkpoint](evidence/direct-gaps-holder-coverage.json)
initializes **18 observed holders** using **36 balance/storage reads**.

The [actual-WASM holder replay](evidence/direct-gaps-wasm-holders.json) matches
**64 initialized observations**, including **29 without a new balance event**,
and all **18 final balances**, including **12 zeros**. Processing performs no
balance RPC calls and reference rows never repair state. Cold replay retains
**29 unknown observations**, including **eight nonzero** observations.

Two captured transaction fixtures preserve **five independently checked
balances** as Rust regressions. These holders come from the bounded reference
window; the checkpoint is not global enumeration.

The [combined capture](evidence/direct-gaps-combined.json) brings the explicit
fixture to **348 profiles** in
[`bsc-direct-gaps-layouts.json`](../tests/fixtures/bsc-direct-gaps-layouts.json).
Every protobuf event field matches for **108,561 previously RPC-verified
emitted balances**, including **15,079 zeros**. There are 347 emitting profiles;
hLBP retains its separate older mint evidence. Identical event histories
preserve evidence for **177,844 initialized observations**, including **69,284
without a new event**. This comparison reuses immutable RPC evidence with
fresh canonical headers, not a new full-348 checkpoint or global snapshot.

All **263 workspace library/binary tests** pass, along with Clippy with warnings
denied, workspace WASM compilation, scoped formatting and diff checks. The
production package remains the XVS-audited artifact:

- SPKG SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`
- WASM SHA-256: `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`

Forty-seven of ranks 351–400 remain unqualified, alongside LBP, TITAN, ORD,
YBC and 钻石 from earlier ranks.

## Reflection findings

The separate [BabyDoge/10SET host model](reflection-holder-model.md) explains
both excluded and ordinary holder getters and verifies passive 10SET balance
changes. Those tokens remain unqualified for the production map. Their model
and raw-storage replay counts are separate from these packaged-output results.
