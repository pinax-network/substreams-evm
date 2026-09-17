# BSC ranks 51–100 and holder exceptions

The [expanded fixture](../tests/fixtures/bsc-next-candidates-layouts.json) contains
76 qualified layouts: the original 50 plus 26 candidates from ranks 51–100 of the
same immutable 1,024-block RPC capture. Rankings count reference balance rows,
not market capitalization. The next 50 account for 14,432 reference rows; no new
ranking window or sampling reset was used.

The [original survey](evidence/next-candidates-survey.json) remains unchanged,
including ambiguous, unresolved and insufficient-sample outcomes. It evaluates
317 sampled full blocks; the actual WASM and holder replay below use all 1,024
consecutive blocks, **122288006–122289029**. The 26 additions are:

| Basis | Ranks | Getter |
| --- | --- | --- |
| Verified source and compiler storage layout | 51, 53, 58, 60, 66, 68, 73, 81, 82, 83, 87, 88, 90, 91, 94, 95, 99, 100 | Direct address-to-uint256 mapping |
| Exact previously reviewed BEP proxy/implementation family | 54, 80, 98 | Mapping 1 behind pinned EIP-1967 implementation |
| Exact previously reviewed SecuritiesToken beacon family | 52, 75, 76, 78 | Namespaced ERC-20 mapping behind pinned beacon/implementation |
| Reviewed proxy source and implementation bytecode | 63 | Mapping 2 with constant zero fallback and holder exceptions |

[Qualification evidence](evidence/next-candidates-qualification.json) binds each
profile to its contract, historical runtime, mapping root and independent
getter controls. Sources are bound to the actual historical runtime, not only a
current explorer label. Proxy/beacon dependency pointers and code are checked at
both boundaries, with persisted upgrade/code changes rejected during ingestion.
Known-runtime siblings do not inherit unrelated deployment baselines.

The permitted non-balance fields come from explicit source/bytecode review.
Dynamic metadata payloads, enumerable role arrays and unreviewed administration
remain unsupported and cause an error when encountered. Source availability is
not blanket permission to ignore arbitrary storage. Every configured token
passes the complete captured-block preflight without unresolved writes.

## Empty burn-address mismatch

The previous rule replaced every zero storage word with a configured fallback.
Two getters instead return zero for address `0x000000000000000000000000000000000000dead`
(and the null address), while returning a synthetic amount for ordinary empty
wallets:

| Contract | Ordinary holder, raw zero | Burn holder, raw zero |
| --- | ---: | ---: |
| `0x98d1341b8ba3dd907d14cf2915014bc3221e80c6` (previously sampled) | 17865090000000000000000 | 0 |
| `0x30f0bd9d666ceeb419c882452f546e5694d5d79a` (rank 63) | 18752320000000000000000 | 0 |

These are integer token units. Both return nonzero raw words unchanged, even for
the burn holder. The first token's earlier sample did not exercise an empty
burn-address balance, so its passing measurements did not expose this branch.

The layout now accepts an explicit `zero_balance.excluded_addresses` list of
verified runtime-constant addresses. The mapper applies the exception only after
raw-word continuity validation. It neither removes burn holders from output nor
changes their nonzero balances. Null holders remain excluded, matching the RPC
reference's event policy. Storage-dependent exclusion lists are not supported.

[Getter instructions](evidence/holder-fallback-getters.json) preserve the exact
mapping read, zero/nonzero branch, zero/dead comparisons and fallback constant
for both implementations. [Historical RPC controls](evidence/holder-fallback-controls.json)
test all nine fallback profiles with raw values 0, 1, 123 and uint256 max for an
ordinary holder, the burn holder and the null holder. All **88 numeric results**
match the corrected layouts; **20 null-address reverts** from five getters are
retained. The other seven fallback profiles retain their prior semantics.

Current historical layout fixtures were corrected too. Earlier retained report
digests refer to their original configurations, available at commit `a6e006b`;
those reports are not rewritten as if they had exercised this new control.

## Actual WASM and retained holders

The [WASM audit](evidence/next-candidates-rpc.json) checks **91,741 emitted
balances**, including **12,792 zeros**, with **zero mismatches** across all 76
tokens and all 1,024 blocks. The 26 additions contribute **4,240 checks**.
[Per-token counts](evidence/next-candidates-rpc-token-counts.json) preserve each
token's coverage. This run completed without an RPC transport failure.

The [initialized-holder replay](evidence/next-candidates-holder-coverage.json)
matches **150,363 reference observations across all 76 tokens**, with zero
unknown or incorrect initialized values. **3,900** matches carry forward without
a new mapper emission. The 26 additions contribute **7,374 observations** and
**162 carried-forward matches**.

Setup verifies **49,684 stored-holder checkpoints** with **99,368 historical
balance/storage RPC reads**. A further 3,001 initial balances use the previously
qualified address formula, and 710/211 holders initialize at the two previously
pinned deployments. Reference observations never repair candidate state.
Processing makes **zero balance RPC calls** and 1,024 separately counted header
verification calls.

Cold replay retains **54,722 unknown observations**, including **29,224 nonzero
values**, with zero incorrect known values. The additions contribute 2,972
unknown observations, including 1,523 nonzero values. These are preserved gaps,
not assumed zeros. Checkpoints cover only holders observed in the bounded
reference window, not a complete global holder inventory.

## Remaining candidates and regressions

The [diagnostics](evidence/next-candidates-diagnostics.json) keep all 24 remaining
candidates explicitly unqualified:

- Rank 56 has matching sampled words in mappings 1 and 36. Getter execution reads
  mapping 1. Changing mapping 36 independently leaves `balanceOf` unchanged;
  its mirrored state is not an alternate balance layout. Full implementation
  and non-balance field review remain pending.
- Ranks 59 and 65 have no runtime at the start of the window. The original 7/23
  unresolved RPC responses remain recorded. Their first CREATE must be pinned
  and validated before initializing holders at deployment.
- Rank 70 has only one active block and 298 zero reference balances. Independent
  mapping-0 overrides return 0, 1, 123 and uint256 max as expected, but this does
  not provide real nonzero activity or emitted-row coverage.
- Ranks 57 and 64 need beacon/proxy and field qualification. Ranks 61, 74, 77, 84,
  86, 96 and 97 use the source-verified FlapTaxTokenV3 implementation; their field
  and dynamic metadata review remains pending. Ranks 71 and 93 require review of
  voting checkpoint arrays. Ranks 55, 62, 67, 69, 72, 79, 85, 89 and 92 require
  further historical bytecode review because source was unavailable.

The [26 captured Rust cases](../tests/fixtures/next-candidates/cases.json) contain
**73 independent historical RPC expectations**. Each retained transaction subset
was checked against its token's full-block output before saving. Additional Rust
regressions cover the two burn-holder exceptions, all nine fallback profiles,
nonzero pass-through, arbitrary configured addresses and malformed/duplicate
exclusions.

**136 workspace library/binary tests pass**, along with Clippy with warnings
denied, workspace WASM compilation and targeted formatting/diff checks. The
rebuilt package is byte-identical to the audited package and has no RPC imports.
[Artifact digests](evidence/next-candidates-artifacts.json) bind its package,
WASM, layouts and captured cases. There is still exactly one `map_events`, shared
`evm.balances.v1.Events`, no extra cache/protobuf and an empty production default.
All executable tooling and tests are Rust.

These windows overlap prior reports and cannot be added as independent history.
Complete cold-start holder enumeration, production bootstrap, identical event-row
coverage and qualification of the remaining candidates are not claimed.

Reproduce with the 76-token fixture, `audit-rpc --start 122288006 --blocks 1024
--workers 2`, and a fresh output directory. For `holder-coverage`, use the original
digest-matching RPC capture (`out/top50-1024/report.json` and `reference.jsonl`)
and all consecutive Extended blocks in `out/top50-full-holder-blocks`. The
capture identities are retained in the [earlier full-window record](evidence/top50-full-capture.json).
