# Nine more source-bound BSC profiles

Nine candidates from the sampled RPC stream pass separate packaged RPC and
initialized-holder checks across **122288006–122289029**, inclusive. The
explicit test configuration now contains **421 profiles among the first 450**.
Ranking counts observations in this window, not market capitalization.

| Cohort | Profiles | Emitted RPC checks | Emitted zeros | Initialized observations | Carry-forward matches | Final holders | Final zeros |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| MGOL and NFTC | 2 | 37 | 0 | 56 | 19 | 20 | 1 |
| WIN and THE | 2 | 37 | 2 | 58 | 21 | 32 | 5 |
| LTC, COS, TWT, X and FRT | 5 | 72 | 9 | 135 | 63 | 61 | 25 |
| **New profiles total** | **9** | **146** | **11** | **249** | **103** | **113** | **31** |

All completed gates have zero value mismatches. Independent parent checkpoints
initialize 113 observed token/holder pairs with 226 balance/storage reads.
Actual production-WASM events drive retained state, with no processing balance
RPC calls and no repairs from reference rows. Every initialized holder is
checked against fresh RPC at the final block.

## Getter coverage

[MGOL and NFTC](simple450-coverage.md) have constructor-only child contracts
with an inherited direct balance getter. [WIN and THE](public450-coverage.md)
use compiler-generated public mapping getters. Their complete source/compiler
runtimes match the historical bytes exactly.

The [five-token cohort](standard450-coverage.md) also uses direct mappings.
FRT's compiler runtime matches exactly; LTC, COS, TWT and X require only checked
CBOR metadata suffix replacements before their complete runtime bytes match.
Reviewed packed metadata includes WIN's issuer/decimals, THE's initial-mint
flag/minter, and TWT's owner/decimals. FRT's treasury, pair and allowlist affect
state-changing operations without changing how `balanceOf` reads balances.
Dynamic string payloads and unknown roots remain unqualified.

Sixteen new Rust regressions retain nine captured transaction fixtures with
24 independent RPC expectations. They replay 180 raw-word controls, 128
ordinary metadata controls and two additional packed-word balance controls.
Separate field-getter and rejected-call controls bind packed interpretation
and ABI behavior. Every source comparison refers to returned compiler
artifacts bound to historical code, not a claimed local compiler rerun.

All **326 workspace Rust library/binary tests** pass, along with Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.
The OG extension adds two host regressions, for five model tests in total.

## Combined replay and limits

The [combined report](evidence/source450-combined.json) uses
[`bsc-source450-layouts.json`](../tests/fixtures/bsc-source450-layouts.json).
Every protobuf event field matches for **109,976 previously RPC-verified
emitted balances**, including **15,247 zeros**. There are 420 emitting profiles;
hLBP retains its separate older mint evidence. Identical event histories
preserve **180,135 initialized observations**, including **70,160 without a
new event**.

The fresh capture binds the previous 412-profile output and these nine newly
audited profiles to canonical headers. It reuses immutable RPC evidence and
does not establish a new full-421 holder checkpoint or global enumeration.
Without their explicit checkpoints, the new cohorts retain **96 unknown
observations, including 42 nonzero balances**. Raw event counts therefore do
not establish identical cold stream coverage.

The [summary](evidence/source450-summary.json) lists the 29 unqualified
candidates among the first 450: seven from the first 400 and 22 from ranks
401–450. The [OG host-model follow-up](og450-host-model.md) resolves its three
previously unsupported sampled branches with 49 additional RPC controls;
it adds no production profile. Recursive pool rewards, complete retained
dependency state, affected-holder output, restarts and rewinds remain separate
work for calculated balances.

The production SPKG remains
`f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`; its WASM
remains `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`.
The interface remains one RPC-free `map_events`, shared
`evm.balances.v1.Events`, explicit layouts and default parameters `[]`.
Ethereum, Base, HyperEVM and Arc still require independent qualification after
BSC.
