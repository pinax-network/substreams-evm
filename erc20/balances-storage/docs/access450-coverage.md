# Five more BSC profiles: permits, roles and short proxies

SLX, PIN, NXT and the sampled USDT and TCF contracts pass independent
packaged RPC and initialized-holder checks across **122288006–122289029**,
inclusive. The explicit test configuration contains **426 profiles among the
first 450 RPC-stream candidates**. Ranks and token labels identify observations
in this window, not market capitalization or issuer identity.

| Cohort | Profiles | Emitted RPC checks | Initialized observations | Carry-forward matches | Final holders | Final zeros |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| SLX and PIN | 2 | 30 | 53 | 23 | 12 | 1 |
| NXT | 1 | 14 | 28 | 14 | 4 | 1 |
| USDT and TCF forwarding contracts | 2 | 34 | 58 | 24 | 10 | 2 |
| **New profiles total** | **5** | **78** | **139** | **61** | **26** | **4** |

Every completed gate has zero value mismatches. None of these 78 emitted
balances is zero. Independent parent checkpoints initialize 26 observed
token/holder pairs using 52 balance/storage reads. Actual production-WASM
events drive retained state without processing balance RPC calls or repairs
from reference rows; fresh final RPC balances check every initialized holder.

## What the new cases cover

[SLX and PIN](permit450-coverage.md) use source-bound direct getters. SLX's
permit nonces do not change balance calculation. PIN's original strict
rejections came from ordinary allowance writes. Its sell-fee burn credits
null-address storage; a captured regression checks the raw null balance
against RPC and verifies that both module outputs filter that holder.

[NXT](roles450-coverage.md) adds packed owner/pause metadata, blacklist and
role membership. Its final layout uses the narrow reachable membership rule.
It rejects the unused outer role-admin word and a word adjacent to a nested
membership value. The broader initial candidate and its test evidence remain
preserved; they are not the layout used in the combined fixture.

[The two forwarding contracts](short450-coverage.md) have separately pinned
outer code, EIP-1967 implementation pointers and effective code. Verified
source is unavailable, so their complete getter and forwarding paths were
reviewed in historical bytecode. TCF's initial seven strict rejections came
from a packed privileged-address/temporary-transfer-flag word, which its
balance getter does not read. All unreviewed scalar writes remain errors in
these new layouts.

The original SLX/PIN RPC transport failure remains preserved. A separate full
audit rechecked the exact complete capture, every canonical header and every
emitted balance before accepting the cohort. Other setup and stricter-layout
investigations retain their original evidence too.

All **352 workspace Rust library/binary tests** pass, together with Clippy
with warnings denied, workspace WASM compilation, scoped formatting and diff
checks. The five profiles add 23 regressions and five captured transaction
fixtures with ten independent nonnull RPC expectations plus PIN's null balance.
Three additional OG regressions exercise the pool gates and historical scan.

## Combined replay and limits

The [combined report](evidence/access450-combined.json) uses
[`bsc-access450-layouts.json`](../tests/fixtures/bsc-access450-layouts.json).
Every protobuf event field matches for **110,054 previously RPC-verified
emitted balances**, including **15,247 zeros**. There are 425 emitting profiles;
hLBP retains its separate older mint evidence. Identical event histories
preserve **180,274 initialized observations**, including **70,221 without a
new event**.

The fresh capture compares the previous 421-profile output and these five
newly audited profiles at canonical headers. It reuses immutable RPC evidence;
it is not a new full-426 holder checkpoint or global enumeration. Without
their explicit checkpoints, the new cohorts retain **61 unknown observations,
including 33 nonzero balances**. Raw event counts therefore do not establish
identical cold-stream coverage.

Final export also binds qualifier reports to their layout files, cohort report
hashes to the combined capture's sources, and native checkpoint/reference
hashes to the retained-holder evidence. These checks prevent superseded NXT
candidate artifacts from being mixed into the final counts.

The [summary](evidence/access450-summary.json) lists 24 unqualified candidates
among the first 450: seven from the first 400 and 17 from ranks 401–450.
The [OG host model](og450-host-model.md) separately adds 19 pool-gate cases:
ten finite numeric matches, six guarded recursive reverts and three clock
underflow cases. All 1,024 historical pool snapshots had zero reward rates
and matched raw balances. OG remains outside production qualification.

The [role-width review](evidence/roles450-width-review.json) also reproduces
an unknown-write guard gap in older MUSD and OLY layouts: the generic recursive
record-width rule accepts `membership_hash + 1` even though a membership is
one word. These are synthetic boundary cases, not historical value mismatches
or demonstrated reachable writes. Existing profiles still need a source-specific
follow-up before narrowing or changing that generic rule. The bounded replay
results do not establish that every unknown nested write is rejected.

The production SPKG remains
`f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`; its WASM
remains `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`.
The interface remains one RPC-free `map_events`, shared
`evm.balances.v1.Events`, explicit layouts and default parameters `[]`.
Ethereum, Base, HyperEVM and Arc require independent qualification after BSC.
