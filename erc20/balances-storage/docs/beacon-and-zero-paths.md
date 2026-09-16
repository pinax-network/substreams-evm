# Beacon proxies and the zero-balance path

The explicit [20-token fixture](../tests/fixtures/bsc-beacon-layouts.json) adds
KII, BNC4, WCOL, AXIS and two additional constant-fallback proxies. It remains a
caller-supplied test configuration. Production defaults are empty, and the only
module is the RPC-free `map_events` with the shared balances protobuf.

## A gap in nonzero transfer sampling

The new Rust `inspect-ranked` command replays a post-block holder for each
candidate and probes mapping words 0, 1, 123 and uint256 max with read-only state
overrides. It verifies the original survey/check digests and canonical block
hash, retains old failed RPC responses as probe provenance, and never promotes a
layout automatically. Probe selection prefers nonzero words, so setting them to
zero actually exercises a different branch.

The [top-50 sweep](evidence/top50-zero-paths.json) completed controls for **48
contracts**. Forty returned the supplied word, while **eight returned a nonzero
fallback for zero**. Five were already covered; the other three had passed the
earlier ordinary transfer samples because their observed words were nonzero:

| Rank | Contract | Mapping base | Zero-word result, raw units | Dependency |
| ---: | --- | ---: | --- | --- |
| 9 | `0xe485050ef170385eb3e1efe4de4e47aaf7017c79` | 2 | `29514590000000000000000` | Constant in pinned implementation `0x6ea87b475ab0870ec305a2381d0469c2b2b1ce3b` |
| 17 | AXIS `0xfebbd3f5fd764ad71cf7895d23caebd1eb4b8701` | 0 | `6000000000000000000` | Scalar slot 3 |
| 49 | `0x98d1341b8ba3dd907d14cf2915014bc3221e80c6` | 2 | `17865090000000000000000` | Constant in pinned implementation `0x9327a8c767f8e29e0408aa7961fdce884d273f54` |

Separate historical inspections confirm **real zero storage words** with those
nonzero RPC results; they are not only synthetic overrides. The
[captured cases](../tests/fixtures/bsc-extra-zero-cases.json) are Rust regressions.
AXIS dependency controls show that a zero holder word follows scalar values 0,
17 and uint256 max, while holder word 1 stays 1. The two implementation traces
execute literal pushes of their fallback constants. Nested allowance getters
were checked against bases 1 or 3 before adding those ignore rules.
[Getter and constant evidence](evidence/beacon-delegate-paths.json).

These three profiles and BNC4 have bytecode/trace/control evidence, without
verified Solidity source. KII and WCOL have independently bound verified source.
Passing a finite control set alone does not prove arbitrary token semantics.

The first sweep used only rows with previously decoded RPC results and therefore
skipped vUSDT. Its 47-token result remains in `out/top50-zero-paths`. The corrected
tool also inspects post-block rows with an earlier unresolved response; its
48-token result is separately saved in `out/top50-zero-paths-complete`.

## BNC4 beacon support

BNC4 (`0x7c8d5502b544ddaf8852fc46d1174e34876d545c`) does not store its implementation
directly. Its EIP-1967 beacon slot points to
`0x453f0bfbb8c46bfe5ae029099d587bc46883ed27`. The beacon's `implementation()` getter
reads address slot 1, which points to
`0x2781ba79f733ceda7467d2dd21688823d89ad53d`. The implementation reads balance
mapping base 51; allowances use 52 and total supply uses scalar 53.

The new optional `beacon_proxy` configuration pins both pointers and both
dependency runtimes. Qualification checks the beacon getter as called by the
token, and direct state-override controls confirm its storage dependency.
Ingestion rejects a changed token beacon pointer, changed beacon implementation
pointer, or changed beacon/implementation code. This includes an upgrade with no
token writes and an upgrade reversed within the block. Rust regressions cover
these cases and reverted writes; the sampled live windows contain stable
implementations. Arbitrary computed or nested beacon resolvers remain unsupported.

The existing `proxy` format remains unchanged and cannot be combined with
`beacon_proxy`. Protected pointer slots cannot be ignored or used as balance or
fallback-dependency slots.

## Source-qualified KII and WCOL

KII (`0xeec6574eabba52bac3f0277f2cd5ac7e67197886`) pins implementation
`0x0d1b644365f97ddf06da5d46ee0ed5721a73d145`. Verified `HypERC20` inherits
OpenZeppelin's mapping-based `balanceOf` at base 51. Cross-chain transfer scaling
applies to mint/burn amounts, not a conversion in the balance getter. Reviewed
non-balance fields include allowances, supply, ownership and routing settings.

WCOL (`0xcfb9bef5f7b748ac72311f057f3a888bc73334d9`) is source-verified
`YieldBearingWrappedCollateral`, with Solmate's public `balanceOf` mapping at
base 3. Its Venus integration changes underlying accounting; it does not redefine
the balance getter. The earlier discovery gate had only one nonzero holder, so
it correctly withheld automatic inference. The reviewed source and controls
establish the explicit layout; the inference threshold was not weakened.
[Source/runtime and inspection evidence](evidence/beacon-source-and-behavior.json).

## Validation

The [five-candidate native survey](evidence/beacon-five-survey.json) covers **430
captured Extended blocks**, with **3,362 before/after checks**, zero value
mismatches, zero RPC errors and zero strict mapper errors. A separate
[WCOL replay](evidence/beacon-wcol-survey.json) covers 128 consecutive blocks and
88 matching before/after checks, also with zero mapper errors. Both retain
`coverage_gap` status because unchanged reference rows are not emitted.

| Inclusive BSC range | Actual WASM balances checked | Zero balances | Mismatches |
| --- | ---: | ---: | ---: |
| 122288160–122288415 | 19,552 | 2,421 | 0 |
| 122288560–122288815 | 17,285 | 2,687 | 0 |
| Total | **36,837** | **5,108** | **0** |

All **20 tokens emitted balances in both windows**; the six additions account
for 2,291 checks. Reports: [A](evidence/beacon-rpc-a.json),
[B](evidence/beacon-rpc-b.json), [per-token counts](evidence/beacon-rpc-token-counts.json).
Package SHA-256: `3dc09d82a321b28bcb2945d36156fddcd3f1f38a8ef97dd363ba05c46bbc8b83`.
WASM SHA-256: `bc057cefdc70d79c28e1b5146a067c746368602130857d572aa40f385e5fbde8`.

The [128-block holder replay](evidence/beacon-holder-coverage.json) matches all
**14,464 reference observations** with a 6,160-holder test checkpoint. Twelve
checkpoint holders have zero storage words and nonzero public balances; four
come from the three newly found fallback profiles. There are zero seeded
unknowns or mismatches. Setup uses 12,320 RPC reads; ongoing balance processing
uses none. Without the checkpoint, 4,388 observations remain unknown, including
1,193 nonzero balances. These are bounded observed-holder results, not global
holder enumeration or a production bootstrap.

The CI-equivalent workspace run passed **104 Rust library/binary tests**.
Targeted Clippy with warnings denied, workspace WASM compilation and targeted
formatting also passed. Regressions include external beacon upgrades and
restore, changed dependency runtimes/getters, malformed controls, and the three
newly captured zero-holder cases. Package metadata confirms one `map_events`
with module hash `7d13906c42c07fc8099e0ea91a00a6b73181c992`.

## Remaining cases and reproduction

Rank 7, `0xec22e64c0a16821dc1b457045936c0219b47155e`, emitted 3,002 reference
balances in one block with **no storage writes**. Its inspected getter reads
scalar slots 3 and 4 and returns a nonzero value unaffected by overriding a
hypothetical mapping at base 0. It needs a separate review of computed balance
semantics; inventing a direct mapping would be incorrect. WCOL was the other
candidate without an automatic probe and is now covered by the source review.
The two contracts deployed inside the original window still need deployment-
boundary qualification; calls before their code exists remain unavailable RPC
comparisons, not zero balances. Full bootstrap, shared-default changes and the
other unreviewed top-50 layouts remain outstanding.

Use the commands in [the previous qualification](fallback-balances.md#reproduction)
with `tests/fixtures/bsc-beacon-layouts.json`. Reproduce the zero-path sweep with:

```sh
cargo run --locked -p erc20-balances-storage-tools -- inspect-ranked \
  --survey erc20/balances-storage/out/top50-parity/report.json \
  --output erc20/balances-storage/out/my-top50-zero-paths
```

All executable tooling and tests are Rust. Source, runtime, trace and raw check
captures remain in the local output directories; committed evidence retains
canonical hashes, trace/check digests and per-token observations.
