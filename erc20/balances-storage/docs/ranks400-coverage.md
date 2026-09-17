# Additional direct and proxy candidates from BSC ranks 351–400

This cohort qualifies sixteen direct getters and eleven instances of previously
reviewed proxy families from the original RPC stream, bringing the explicit
fixture to **375 of the first 400 profiles**. Twenty-five remain unqualified.
The interval is
**122288006–122289029**, inclusive. Ranking measures observations in that stream,
not market capitalization. Layouts remain explicit test inputs; the production
default remains `[]` and there is still one RPC-free `map_events` using
`evm.balances.v1.Events`.

## Direct getters and metadata

The [source qualification](evidence/direct400-qualified.json) binds each getter
to verified source and its historical runtime. Each getter reads the configured
address mapping directly. Fresh read-only overrides at zero, one, 123 and
uint256 maximum return the exact stored word. Runtime checks cover both interval
boundaries. MOSS exposes its mapping as the public `balanceOf` getter.

| Rank | Source contract name | Balance root |
| ---: | --- | ---: |
| 352 | PepeToken | 9 |
| 357 | PeerToken | 0 |
| 358 | KavaOFT | 5 |
| 363 | FHE | 0 |
| 365 | Aria | 0 |
| 367 | SomniaOFT | 5 |
| 369 | MOSS | 4 |
| 370 | GETAI | 2 |
| 371 | GOT | 0 |
| 374 | KOMA | 2 |
| 375 | HashToken | 4 |
| 384 | Luck | 1 |
| 388 | BoBoToken | 0 |
| 389 | HanaToken | 0 |
| 393 | OpenUSD | 5 |
| 398 | Token | 0 |

PeerToken overrides its inherited voting clock with timestamps; Luck uses block
numbers. Source packing, fresh `clock()` and `CLOCK_MODE()` calls establish the
distinct checkpoint configurations. Their checkpoint records contain a uint48
clock and uint208 value in one word.

GOT uses three fixed words for enumerable role records: the member-array root,
the member-index mapping root and the admin role. FHE uses the simpler two-word
role layout. The [initial source review](evidence/direct400-initial-qualification.json)
rejected GOT because the review helper expected that simpler shape. Explicit
review of the nested enumerable structure corrected the configuration; it did
not discard a balance mismatch or relax discovery thresholds. Dynamic role-list
payloads remain unqualified and still stop processing.

MOSS's fixed ten-word price array and OpenUSD's two packed uint128 totals are
also reviewed explicitly. Fee, nonce, allowance, OFT and scalar metadata use
only their declared roots. Unknown writes, including unreviewed long dynamic
metadata, remain errors. These profiles require no production algorithm change.

## Exact proxy families

The [proxy qualification](evidence/family400-qualified.json) and
[fresh dependency checks](evidence/family400-runtime-checks.json) cover seven
minimal proxies and four beacon proxies. Each layout starts from the current
qualified profile, then independently verifies the candidate's forwarding code,
implementation code, pointers, getter storage reads and four raw-word controls.
Matching a proxy shell alone is insufficient. A template's deployment witness
is never copied: all eleven candidates already have code at the parent block.

| Candidate ranks | Reviewed family and original evidence |
| --- | --- |
| 368, 380, 381, 385, 396, 399 | Minimal proxy to `0x024f18294970b5c76c0691b87f138a0317156422`; [next proxy qualification](next-proxy-holder-coverage.md) |
| 400 | Minimal proxy to `0x8b4329947e34b6d56d71a3385cac122bade7d78d`; [clone qualification](clone-holder-coverage.md) |
| 360 | BLESS family, including the beacon's forwarding implementation and admin; [admin dependency](beacon-admin-holder-coverage.md) |
| 378, 397 | SecuritiesToken with its namespaced raw balance mapping; [getter and beacon review](securities-proxy-holder-coverage.md) |
| 387 | BridgeToken beacon and forwarding layer; [ranked proxy qualification](ranked-proxy-coverage.md) |

The source and manual-bytecode provenance of those original reviews retains its
original scope. Fresh traces must use only the configured code and storage
dependencies. Processing still rejects changes to protected dependencies,
including changes restored within the block. No new production rule is needed.

## Packaged and holder validation

Both the [direct](evidence/direct400-rpc.json) and
[proxy](evidence/family400-rpc.json) packaged audits match every emitted balance
against RPC. Their separate [direct](evidence/direct400-wasm-holders.json) and
[proxy](evidence/family400-wasm-holders.json) retained-state replays also pass.

| Measurement | Sixteen direct profiles | Eleven proxy profiles | Total |
| --- | ---: | ---: | ---: |
| Emitted balances checked against RPC | 304 | 229 | 533 |
| Emitted zeros | 14 | 44 | 58 |
| Initialized observed holders | 218 | 184 | 402 |
| Checkpoint balance/storage reads | 436 | 368 | 804 |
| Initialized reference observations | 543 | 358 | 901 |
| Matches without a new balance event | 239 | 129 | 368 |
| Final holders checked against RPC | 218 | 184 | 402 |
| Final zero balances | 76 | 84 | 160 |
| Cold unknown observations | 232 | 123 | 355 |
| Cold unknown nonzero observations | 97 | 28 | 125 |

Final cohort and combined results are recorded in
[the summary](evidence/ranks400-summary.json). Both cohorts use the same audited
production package. RPC checks are hash-pinned; complete clocks cover blocks
without output. Holder initialization uses independent parent-block balance and
storage reads. Subsequent processing makes no balance RPC calls and reference
values never repair retained state. Final checks include every initialized
observed holder, not only the holders with a recent event.

The captured Rust regressions in `src/ranks400_tests.rs` retain token-writing
transactions from complete Extended blocks and verify their output against
**78 independent RPC expectations in 27 fixtures**. Filtering is accepted only
when it preserves the full block's output for that token. All **265 workspace
library/binary tests** pass, together with Clippy with warnings denied, workspace
WASM compilation, scoped formatting and diff checks.

The [combined capture](evidence/ranks400-combined.json) runs all 375 profiles in
[`bsc-ranks400-layouts.json`](../tests/fixtures/bsc-ranks400-layouts.json). Every
protobuf event field matches for **109,094 previously RPC-verified emitted
balances**, including **15,137 zeros**. There are **374 emitting profiles**;
hLBP retains separate older mint evidence. Identical event histories preserve
**178,745 initialized observations**, including **69,652 without a new event**.
This reuses immutable RPC evidence with fresh canonical headers, not a new
full-375 holder checkpoint or global snapshot. The production artifacts remain:

- SPKG SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`
- WASM SHA-256: `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`

The summary lists all **25 remaining candidates** by rank and address. Twenty
come from ranks 351–400; the five earlier candidates are LBP, TITAN, ORD, YBC
and 钻石.

Cold unknowns, initialized holder observations, emitted parity and global holder
enumeration remain separate. Matching this interval does not qualify every
future administrative path or every holder on BSC. Reflection and external or
time-dependent getters remain separate investigations; their host-only models
do not establish production support. Ethereum, Base, HyperEVM and Arc still need
independent qualification after BSC.

Reproduce emitted checks with the Rust `audit-rpc` command, a cohort's
`tests/fixtures/{direct400,family400}/layouts.json`, start 122288006 and 1024
blocks. Use `holder-coverage` with the original ranking/reference and complete
Extended block directory for the native holder comparison. Use fresh output
directories so unsuccessful evidence is preserved.
