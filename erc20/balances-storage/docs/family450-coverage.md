# Thirteen more BSC proxy candidates

This cohort qualifies ranks **410, 414, 420, 421, 425, 432, 434, 435, 438,
439, 442, 445 and 446** from the original RPC balance stream over
**122288006–122289029**, inclusive. Ranking measures observations in that stream.
The [layouts](../tests/fixtures/family450/layouts.json) remain explicit test inputs;
the production default is still `[]`, with one RPC-free `map_events` and the
shared `evm.balances.v1.Events` output.

## Effective implementation review

The [qualification](evidence/family450-qualified.json) independently resolves each
target's implementation, beacon and forwarding pointers. Historical code and
pointer checks cover both interval boundaries. Fresh getter traces bind executed
code and storage reads to those resolved dependencies. Matching an outer proxy
runtime alone does not qualify a target.

| Ranks | Reviewed effective implementation | Raw balance root |
| --- | --- | --- |
| 410, 421, 432, 434, 435, 438, 445, 446 | FlapTaxTokenV3 through canonical ERC-1167 proxies | 51 |
| 414 | SecuritiesToken through its reviewed beacon | ERC-7201 ERC20 namespace |
| 420 | Reviewed registration-enabled minimal proxy implementation | 1 |
| 425 | BridgeToken through a beacon with its own ERC-1967 forwarding layer | 5 |
| 439 | BEP20 implementation through a transparent proxy | 1 |
| 442 | TokenV2 through a canonical ERC-1167 proxy | 51 |

The [review basis](evidence/family450-review-basis.json) links and hashes the
previously committed source, compiler-layout and manual-bytecode evidence for
each exact runtime. Existing source provenance keeps its original scope,
including the source-unavailable SecuritiesToken beacon's reviewed bytecode.
Candidate-specific deployment witnesses and holder values are never copied.
All thirteen targets already exist at the parent block.

The fresh RPC controls cover **244 raw-word cases** across sampled, null, burn,
maximum and token addresses, deduplicating repeated addresses. Values zero,
one, 123 and uint256 maximum all return the raw stored word. Another **26 controls**
set each token's configured fixed metadata words simultaneously to zero or
maximum while retaining a balance of 123. The exported exact request/response
[record](evidence/family450-runtime-requests.json) is checked against every
recorded control payload. These controls supplement the complete reviewed
getter semantics; they do not infer mapping meaning from sample equality.

Allowance, nonce, pool, role and registration metadata retain only their reviewed
roots and shapes. Registration list changes still require their append/removal
witnesses; bounded string payload words do not authorize arbitrary extra words.
Unknown writes stop processing.

## Transparent-proxy admin refusal and correction

The [initial qualification](evidence/family450-initial-qualification.json) refused
AAVE, rank 439, because its getter trace reads an additional admin slot. The
other twelve passed their native replay. A fresh final run qualifies AAVE only
after resolving its nonzero admin at the parent and final blocks, checking the
complete caller-dependent forwarding path, and observing RPC reverts for both
the actual admin caller and an override that makes the default zero caller the
admin.

The admin slot is **not** added to ignored metadata. Persisted admin changes,
including a change followed by restoration in the same block, remain errors
even without holder writes. The same qualification still pins the implementation
pointer and runtime. This establishes the default caller's behavior for the
qualified interval, not arbitrary behavior after future admin or code changes.

## Packaged output and retained holders

The [packaged RPC audit](evidence/family450-rpc.json),
[native holder replay](evidence/family450-holder-coverage.json) and
[actual-WASM retained-state replay](evidence/family450-wasm-holders.json) all pass
the same 1,024 blocks. Complete canonical clocks include the 938 blocks with no
balance output. All thirteen profiles emit.

| Measurement | Count |
| --- | ---: |
| Emitted balances checked against independent RPC | 270 |
| Emitted zeros | 31 |
| Initialized observed holders | 167 |
| Parent balance/storage RPC reads | 334 |
| Initialized reference observations | 356 |
| Matches retained without a new balance event | 86 |
| Final holders checked against RPC | 167 |
| Final zero balances | 64 |
| Cold unknown observations | 84 |
| Cold unknown nonzero observations | 33 |
| Value mismatches | 0 |

The [summary](evidence/family450-summary.json) binds the layouts, production
package and reports. Initialization uses independent parent balance and storage
reads. Processing makes no balance RPC calls and reference values never repair
retained state. The final snapshot checks every initialized observed holder.

The Rust regressions in `src/family450_tests.rs` contain **13 captured fixtures
and 39 independent RPC expectations**, exact raw/metadata override regressions,
unknown mapping refusal, and implementation/admin change-and-restoration
refusals without holder writes. Captured fixtures retain whole token-writing
transactions and are accepted only when their output equals the full block;
live gates process complete Extended blocks. All six focused regressions pass.

Null-address output remains filtered by the shared Events behavior. Cold unknown
state requires a verified checkpoint or sufficient earlier history. This is
bounded observed-holder coverage, not global enumeration, universal ERC-20
support, or qualification of every future administrative path. Other networks
need their own code, data and holder checks.

Reproduce with the Rust `audit-rpc` and `holder-coverage` commands, the cohort
fixture layouts, start 122288006 and 1,024 blocks. Use fresh output directories
to preserve unsuccessful evidence. Helper source is retained locally under
`out/next-candidate-rust-scripts/family450-followup/`.
