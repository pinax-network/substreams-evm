# XVS unsigned balance width

XVS (`0xcf6bb5389c92bdda8a3747ddb454cb7a64626c63`, rank 362) stores balances
as `mapping(address => uint96)` at slot 2. Its historical source and getter
trace bind to runtime hash
`0x27d40afa4bd607ec4235e05bff971472c1d9effd4d4757a9da3b3daa735a3eb3`.
The getter returns only the lower 96 bits, while a full-word projection returns
all 256 bits.

At block 122288254, independent read-only RPC controls return 0, 1 and 123 for
those words. Overriding the balance word with uint256 maximum returns
`79228162514264337593543950335`, the uint96 maximum, rather than the full
overridden word. This is a controlled counterexample, not a naturally observed
historical mismatch. Normal samples alone did not reveal the width difference.

## Mapper correction

The optional `balance_bits` layout field explicitly supplies the reviewed
unsigned width, in whole bytes from 8 through 256, at byte offset zero.
Omission retains the existing full-word behavior. Values are projected only
after full raw-word continuity checks; equal public balances cannot conceal
inconsistent high bits in the underlying storage history. Widths are never
inferred from sampled magnitudes, and signed values or arbitrary struct offsets
are not covered. Other balance formulas cannot be combined with this rule.

The XVS configuration uses `balance_bits: 96`, reviewed owner/lock metadata in
slot 0, and allowance, delegate, nested checkpoint, checkpoint-count and nonce
mapping roots 1 and 3 through 6. Each checkpoint is a single word containing
`uint32 fromBlock` and `uint96 votes`; it is not an OpenZeppelin dynamic array.
The [qualification evidence](evidence/xvs-qualified.json) preserves source
layout, historical binding and the original full-word control mismatch.

Production remains one RPC-free `map_events`, shared `evm.balances.v1.Events`,
explicit caller-qualified layouts and default parameters `[]`.

## Validation

The [canonical runtime checks](evidence/xvs-runtime-checks.json) bind XVS at
the parent boundary, getter-control block and final boundary. The actual
[packaged-WASM RPC audit](evidence/xvs-rpc.json) matches **19 emitted balances**,
including **two zeros**, over all **1,024 blocks**, 122288006–122289029.
Canonical clocks account for the 1,016 empty-output blocks.

The [holder setup](evidence/xvs-holder-coverage.json) initializes **21 observed
holders** using **42 explicit balance/storage reads** at the parent checkpoint.
The [actual-WASM holder replay](evidence/xvs-wasm-holders.json) matches **36
initialized observations**, including **17 without a new balance event**, and
all **21 final balances**, including **six zeros**. Processing performs no
balance RPC calls and reference amounts never repair state. Cold replay retains
**17 unknown observations**, including **12 nonzero** observations.

The [combined replay](evidence/xvs-combined.json) uses **346 configured
profiles** in [`bsc-xvs-layouts.json`](../tests/fixtures/bsc-xvs-layouts.json),
with 345 emitting in this interval and hLBP retaining its separate older mint
evidence. Every protobuf event field matches for **108,526 previously
RPC-verified balances**, including **15,059 zeros**. Identical event histories
preserve evidence for **177,780 initialized observations** and **69,255 matches
without a new event**.

The captured Rust regression retains one actual transfer fixture with two
independently checked balances. Further regressions retain all four RPC word
controls, test explicit width validation and incompatible rules, and ensure
high-bit changes cannot conceal raw-word discontinuity.

All **256 workspace library/binary tests** pass, along with Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.

Audited artifact SHA-256 values:

- SPKG: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`
- WASM: `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`

The fresh combined replay compares every protobuf event field from the new
package with immutable prior RPC-verified output. The old package is retained
and hash-bound separately: a changed package does not inherit its qualification
merely by sharing a filename. This does not create a new all-profile checkpoint
or global holder enumeration.

## Other BSC findings

The [ranks 351–400 investigation](ranks351-400-investigation.md) preserves 1,654
successful exploratory RPC comparisons with no historical value mismatches.
Forty-four candidates have a unique matching mapping hypothesis; only XVS is
promoted by this follow-up after separate source, full-window, packaged and
holder checks. The remaining candidates still require those gates.

The [钻石 coupled controls](evidence/diamond-record-controls.json) explain the
earlier zero results: the low byte at offset 9 of its root-19 account record
bypasses the contribution when nonzero. Eleven storage-only cases produce 22
getter outcomes without replacing either deployed contract's code. Enabled
pending values, capacity limits, accumulated values and proportional rounding
can change `balanceOf`; these simulated states are not historical mismatches.
No production calculated-balance support is added by that diagnostic.
