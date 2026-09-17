# BSC ranks 301–350: 36 more qualified profiles

The explicit test configuration now contains **332 qualified profiles** from the
original top 350 RPC-stream candidates. This follow-up adds 36 profiles; 14
from the latest 50 still need review, alongside the four earlier computed-balance
candidates. The [candidate summary](evidence/ranks301-350-summary.json) records
each addition and each pending token.

Production remains one RPC-free `map_events`, the shared `evm.balances.v1.Events`
protobuf and default parameters `[]`. These additions require no production
mapper change. Tests and executable diagnostics remain Rust.

## Qualification

The [source and getter evidence](evidence/ranks301-350-qualification.json)
qualifies 26 direct/public mappings against verified historical source and
four independent holder-word controls. Explicit storage rules cover only
reviewed fields, including two-word voting checkpoints and bridge-supply
records. Unconfigured role, bridge-membership and other administrative writes
still reject processing where their handling has not been reviewed.

Ten other profiles reuse an exact reviewed runtime family. Fresh
[canonical dependency checks](evidence/ranks301-350-runtime-checks.json) bind
each token's implementation and beacon dependencies; matching a proxy shell
alone is insufficient. Deployment metadata from template tokens is removed
only for candidates whose own code already exists at the parent boundary.

All 36 profiles pass a complete native scan of **1,024 consecutive Extended
blocks**, from **122288006 through 122289029**, with no unresolved writes.
These are bounded runtime and interval qualifications, not universal token
semantics or complete global holder enumeration.

WMAI is included. Its original automatic discovery sample had only one nonzero
holder, below the required two. Its verified source, getter trace and word
controls independently establish balance slot 1; discovery thresholds are
unchanged. Its final qualified layout also handles the reviewed allowance,
supply, ownership and fee-configuration fields.

## Packaged RPC output and retained holders

The [actual packaged-WASM audit](evidence/ranks301-350-rpc.json) matches all
**935 emitted balances** with historical RPC, including **121 zeros**. Complete
canonical clocks account for empty-output blocks.

The [native holder experiment](evidence/ranks301-350-holder-coverage.json)
initializes 637 observed holders with **1,274 explicit balance/storage reads**.
The [actual-WASM holder replay](evidence/ranks301-350-wasm-holders.json) then
confirms:

- **1,458 initialized reference observations**, with no mismatches.
- **523 matches without a new balance event**.
- All **637 final balances** match a fresh RPC snapshot, including **203 zeros**.
- Processing performs no balance RPC calls; reference observations never repair
  retained state.
- Cold state leaves **513 observations unknown**, including **206 nonzero**.
  Missing history is not converted into zero balances.

The [fresh combined capture](evidence/ranks301-350-combined.json) uses
[`bsc-ranks301-350-layouts.json`](../tests/fixtures/bsc-ranks301-350-layouts.json).
It preserves every protobuf event field for **108,212 previously RPC-verified
emitted balances**, including **15,036 zeros**. There are 331 emitting profiles;
hLBP is quiet in the original interval and has separate older mint evidence.
Identical combined event histories preserve evidence for **177,227 initialized
observations** and **69,016 matches without a new event**.

That combined comparison reuses immutable RPC evidence with fresh canonical
headers and runtime checks. It does not create a fresh full-332 checkpoint or
global holder snapshot. The separately initialized 36-token cohort supplies the
new holder evidence.

Thirty-six captured transaction fixtures retain **136 independently checked
balances** in [Rust regression tests](../src/ranks301_350_tests.rs). Validation
passes **251 workspace library/binary tests**, Clippy with warnings denied,
workspace WASM compilation, scoped formatting and diff checks. The production
SPKG remains `f13ce00a08e3610ed607a168c1f2afac6fb92ac63561f945e97a55d7b4801503`.

## Pending work

Ranks **303, 312, 319, 321, 325, 326, 327, 332, 333, 338, 339, 341, 342 and 345**
remain outside the combined fixture. The original [survey](ranks301-350-investigation.md)
and predeployment failure are preserved. Source/runtime review, explicit storage
rules and packaged holder audits remain necessary before promotion.

Tracker's [additional diagnostic](evidence/tracker-storage-followup.json)
records the complete observed balance getter path and probes dividend-related
getters. Its selector dispatch leads to an address decode, a single slot-0
mapping read and an untransformed return. Dividend getters read other fields;
they do not establish that dividends are part of `balanceOf`. The previously
observed slot-12 values remain separate bookkeeping. Tracker's remaining
storage writes still need review, so it is not promoted.

LBP, TITAN, ORD and YBC remain unqualified for production calculated balances.
After BSC, repeat the same independent qualification for
[Ethereum, Base, HyperEVM and Arc](network-expansion.md).
