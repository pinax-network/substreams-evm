# Eight more BSC balance layouts

The [next cohort](ranks151-200-coverage.md) extends this snapshot to 178 profiles.

Ranks **104, 107, 117, 121, 122, 124, 140 and 146** now pass qualification,
bringing the [combined fixture](../tests/fixtures/bsc-remaining-ranked-layouts.json)
to **149 profiles**. The interval remains **122288006–122289029**, inclusive.
Rank 143, the reward-bearing LBP contract, remains unqualified. Production keeps
one RPC-free `map_events`, shared `evm.balances.v1.Events`, caller-qualified
layouts and default parameters `[]`.

## Getter and field review

The [qualification evidence](evidence/remaining-ranked-qualification.json)
binds the reviewed bytecode paths to historical runtime identities at both
boundaries. Verified source was unavailable for these direct contracts and the
proxy implementation. Each valid `balanceOf` path reads one balance mapping
word and returns it unchanged. The other branches validate call value and
canonical calldata; no caller/time-dependent balance branch, secondary balance
read or external getter call appears in these paths.

| Rank | Balance root | Reviewed additional field |
| --- | --- | --- |
| 104 | 0 | Allowances at 1; total supply at 2 |
| 107 | 0 | Allowances at 1 |
| 117 | 5 | Allowances at 6 |
| 121 | 0 | Allowances at 1 |
| 122 | 51 | Allowances at 52; pinned ERC-1967 implementation |
| 124 | 0 | Allowances at 1 |
| 140 | 0 | Allowances at 1 |
| 146 | 4 | Allowances at 5 |

**160 read-only raw-word controls** cover zero, one, 123 and uint256 maximum
across five holder contexts per token. **26 non-balance-field controls** check
allowance or supply getter results and verify `balanceOf` remains unchanged.
Allowance traces independently identify the nested mapping key; adjacent slot
numbers alone are not accepted as proof.

Rank 104 previously stopped at blocks **122288448 and 122288504** because slot 2
was unqualified. The `totalSupply()` dispatcher and storage read identify that
field; independent zero/maximum overrides confirm its effect on supply and
independence from the balance getter. Adding only this explicit field resolves
both failures. The [original failed scan](evidence/remaining-ranked-layout-scan.json)
is preserved, and captured regressions reproduce both failures when the field
is removed. All other unknown writes still stop processing.

Rank 122 uses a runtime-bound, source-verified `TransparentUpgradeableProxy`.
Its implementation at `0x9151434b16b9763660705744891fa906f660ecc5` reads balance
root 51. Both runtimes and the implementation pointer are qualified at both
boundaries. The proxy admin slot is not ignored; changes still stop processing.

## RPC output and holder coverage

The [actual packaged-WASM audit](evidence/remaining-ranked-rpc.json) matches
all **787 emitted balances**, including **126 zeros**, with no RPC mismatches.
All eight tokens emit. Complete canonical clock evidence covers **773 empty
output blocks**, alongside 251 blocks with output.

The [native holder audit](evidence/remaining-ranked-holder-coverage.json)
initializes **451 observed holders** with **902 explicit balance/storage RPC
reads**. All **1,233 reference observations** match, with no initialized unknowns
or mismatches. Cold-start unknowns remain explicit: 439 observations, including
106 nonzero values, cannot be reconstructed from new writes alone.

The [independent actual-WASM replay](evidence/remaining-ranked-wasm-holders.json)
also matches all 1,233 observations, including **446 without a new event**.
A final canonical RPC snapshot matches **all 451 retained holders**, including
**167 zeros**. Reference observations never repair retained state. Processing
makes no balance RPC calls; initialization and final verification are separate.

## Regression and combined results

Ten captured blocks cover the eight profiles and both previously failing supply
updates. A separate captured regression detects 33 reward-driven holder
mismatches. **201 workspace Rust library/binary tests pass**, together with Clippy
with warnings denied, targeted formatting and diff checks. Repackaging preserves
the exact previous package and WASM digests; this extension changes qualification
fixtures and regression tests, not ingestion logic or package metadata.

The [combined evidence](evidence/remaining-ranked-summary.json) totals
**100,598 emitted RPC checks**, including **14,272 zeros**, and **164,911
initialized holder observations**, all matching. These totals combine disjoint
140-, 1- and 8-token WASM runs on the identical package and canonical interval.
A combined native replay with all 149 layouts matches their complete row/value
union in every block. No new combined 149-token WASM capture is claimed.

These remain bounded tests, not global holder enumeration or universal ERC-20
support. [LBP counterexamples](lbp-reward-mismatch.md) now show 33 holders whose
RPC balances change without any holder balance-word write. It still needs
qualification of its external, time-dependent pending rewards; zero samples and
pool exemptions cannot establish raw-mapping parity.
Ethereum, Base, HyperEVM and Arc retain their separate requested qualification
sequence after the BSC work.
