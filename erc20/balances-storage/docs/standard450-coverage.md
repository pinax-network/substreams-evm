# LTC, COS, TWT, X and FRT balance layouts

This cohort reviews five more candidates from the original BSC RPC balance
stream over **122288006–122289029**, inclusive. Ranking counts sampled balance
observations. Layouts remain explicit test inputs with an empty production
default, one RPC-free `map_events` and shared `evm.balances.v1.Events` output.

| Rank | Token | Balance root | Reviewed non-balance fields |
| ---: | --- | ---: | --- |
| 408 | FRT | 0 | Allowance root 1; supply, names, owner, treasury, pair and allowlist root 8 |
| 416 | X | 0 | Allowance root 1; supply, decimals and names |
| 441 | LTC | 1 | Allowance root 2; owner, supply, decimals and names |
| 443 | COS | 1 | Allowance root 2; owner, supply, decimals and names |
| 450 | TWT | 0 | Allowance root 1; supply, names and packed owner/decimals |

## Complete getter and field review

Every complete `balanceOf` body returns `_balances[account]` directly. There is
no overriding derived getter, fallback value, conversion, external call, caller
branch or time dependency. The [source review](evidence/standard450-source-review.json)
retains complete source, compiler storage declarations and historical runtime
bindings. FRT's compiled bytes match exactly. For the other four tokens, only
declared CBOR metadata suffix replacements are accepted; the reconstructed
complete runtime must match the historical bytes. Fresh code reads cover the
parent, sampled getter block and final block.

TWT packs its uint8 decimals at byte zero and its owner address at byte one of
slot 5. Independent owner/decimals calls under two mixed-word overrides verify
that packing while `balanceOf` remains 123. The implementation uses no proxy;
the owner word is administrative metadata rather than a forwarding dependency.

FRT's treasury controls mint/burn authorization. Its pair and `isSwapAllowed`
mapping restrict buys and sells in `_update`; none participates in the inherited
balance getter. Mint/burn and transfers still update the ordinary balance
mapping. Only the compiler-declared allowance and allowlist roots are accepted
as non-balance mappings. Long string payloads and unknown roots remain errors.

Fresh hash-pinned read-only controls cover **100 raw mapping values**, **60
fixed-field/allowance/allowlist cases** and **two packed-field cases**. Another
ten allowance getter calls and four owner/decimals getter calls verify field
interpretation. All **176 RPC request/response pairs** are checked against the
exact sent override payloads in the [request evidence](evidence/standard450-rpc-requests.json).
The source review establishes semantics; matching samples alone does not.

## Replay evidence

The strict native scan passes all 1,024 complete Extended blocks without an
unresolved write. Packaged RPC and holder results are recorded in the
[summary](evidence/standard450-summary.json), with separate
[packaged](evidence/standard450-rpc.json),
[native holder](evidence/standard450-holder-coverage.json) and
[actual-WASM retained-state](evidence/standard450-wasm-holders.json) reports.

All five profiles emit, and complete canonical clocks cover the 994 blocks with
no output. Every gate has zero value mismatches.

| Measurement | Count |
| --- | ---: |
| Emitted balances checked against RPC | 72 |
| Emitted zeros | 9 |
| Initialized observed holders | 61 |
| Parent balance/storage reads | 122 |
| Initialized reference observations | 135 |
| Matches retained without a new event | 63 |
| Final holders checked against RPC | 61 |
| Final zero balances | 25 |
| Cold unknown observations | 59 |
| Cold unknown nonzero observations | 26 |

Initialization uses independent parent-block balance/storage reads. Actual
production-WASM events drive retained state, and reference amounts never repair
that state. Final RPC checks cover every initialized observed holder, including
holders without a recent balance event. Processing uses no balance RPC calls.

Five Rust fixtures retain whole token-writing transactions and their original
headers, accepted only when their output matches the corresponding complete
block. Live gates use complete blocks. Six regressions cover 15 independent captured
RPC amounts, null-address filtering, field controls, packed TWT storage, and
unknown mapping/fixed-word rejection.
All six focused regressions pass. Their Rust helper sources are retained locally
under `out/next-candidate-rust-scripts/standard450-followup/`.

This is bounded observed-holder coverage. Cold unknown state requires an
independent checkpoint or sufficient earlier history; it is never assumed zero.
There is no global-holder enumeration, new aggregate checkpoint, deployment
claim or guarantee for every future administrative path. Reproduce with the
Rust `audit-rpc` and `holder-coverage` commands, this cohort's fixture layouts
and the same start/range, using fresh output directories.
