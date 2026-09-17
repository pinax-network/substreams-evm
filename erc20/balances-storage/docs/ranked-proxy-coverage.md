# BSC proxy qualification and upgradeable beacons

The [subsequent clone qualification](clone-fallback-coverage.md) extends coverage
to 141 profiles. This report retains the original 140-token run and evidence.

Seven more candidates from ranks 101–150 are qualified in
[the 140-token fixture](../tests/fixtures/bsc-ranked-proxy-layouts.json).
The original 133 profiles retain identical parameters and output. Ten candidates
from this cohort remain unqualified. Production keeps one RPC-free `map_events`,
shared `evm.balances.v1.Events`, explicit caller-qualified layouts and defaults
`[]`. All executable diagnostics and tests remain Rust.

## The additional beacon dependency

Ranks **142 and 149**, contracts
`0x30117e4bc17d7b044194b76a38365c53b72f7d49` and
`0x595deaad1eb5476ff1e649fdb7efc36f1e4679cc`, use a beacon that is itself an
ERC-1967 proxy. Pinning its outer runtime and token-implementation storage word
alone would miss an upgrade of the code executing the beacon getter. Such an
upgrade could change balance semantics without modifying either token contract.

The [qualification evidence](evidence/ranked-proxy-qualification.json) binds all
four runtime roles and three pointer roles at both interval boundaries:

| Account or role | Verified behavior |
| --- | --- |
| Each token proxy | Its ERC-1967 beacon slot selects `0xb6f6d86a8f9879a9c87f643768d9efc38c1da6e7` |
| Beacon proxy | Its ERC-1967 implementation slot selects `0x621199f6beb2ba6fbd962e8a52a320ea4f6d4aa3` |
| Beacon delegate, `BridgeImplementation` | `implementation()` calls `tokenImplementation()`, which reads slot 1 in **beacon storage** |
| Token implementation | Slot 1 selects `0x7f8c5e730121657e17e452c5a1ba3fa1ef96f22a`; its `balanceOf` directly returns mapping 5 |

All implementation sources are bound to their historical runtimes. The beacon
trace independently reads the forwarding pointer at PC 133 and slot 1 at
PC 4702 in the delegated frame. The token getter reads its balance word at
PC 544. Its balance result is independent of the reviewed metadata, allowance
and nonce fields.

The optional `beacon_proxy.proxy` field pins this one extra forwarding layer,
using `implementation_slot`, `implementation` and `code_hash`. Its pointer is
scoped to the beacon account; a similarly numbered token slot is a separate
field. Ambiguous dependency addresses, overlapping pointer roles and recursive
proxy declarations are rejected.

Qualification checks the extra pointer and code at both boundaries. During
ingestion, every persisted change to either beacon pointer or dependency code
stops processing, including a change followed by a restore, a system-call
change, or a block with no token holder writes. Reverted or failed execution
does not invalidate the layout. The extra dependency cannot be bypassed by
placing a token storage field on an ignore list.

This adds support for a reviewed single forwarding layer. It does not infer
arbitrary beacon resolver graphs, computed targets or new implementation
semantics after an upgrade. Those need new qualification.

## Other newly qualified profiles

- **Ranks 108, 130 and 136:** verified BEP20 proxy implementations with the
  direct balance mapping at slot 1 and allowances at slot 2.
- **Rank 128:** verified `BurnMintERC20Upgrade`, with the ordinary balance
  mapping at slot 51. Minter/burner set metadata is explicit; arbitrary set
  element writes are not silently allowed.
- **Rank 145:** verified `COAI`, with the OpenZeppelin namespaced ERC-20
  balance mapping. Allowances, nonces, initialization, ownership, EIP-712 and
  mint/burn bookkeeping roots are reviewed separately from balances.

The complete getter review is supplemented by **28 raw balance-word controls**
and **148 non-balance-field controls**. The latter independently override each
configured root with zero and uint256 maximum and verify the holder's RPC
balance does not change. These probes support source review; matching samples
alone do not qualify a getter. Unknown writes, including unconfigured dynamic
string data and EnumerableSet elements, still stop processing.

## RPC and retained-holder results

The same **1,024 consecutive BSC blocks, 122288006–122289029**, are checked
with the updated package and all 140 profiles. The
[actual WASM audit](evidence/ranked-proxy-rpc.json) matches **99,789 emitted
balances**, including **14,146 zeros**, with **zero mismatches**. All 140
configured tokens emit. The new seven contribute **524 balances**, including
49 zeros. The native scan has no unresolved changes or protected beacon-pointer
writes across the complete interval.

The [new-token holder replay](evidence/ranked-proxy-holder-coverage.json) uses
315 independently checked initial holders and **630 explicit balance/storage
RPC reads**. All **989 reference observations** match, with zero initialized
unknowns or value mismatches. Processing makes no balance RPC calls; all 1,024
block headers are checked against canonical RPC.

An [independent replay of actual WASM output](evidence/ranked-proxy-wasm-holders.json)
also covers the full **140-token** set. It reuses the prior 133-token checkpoint
only after verifying its original digest, identical layout parameters and the
same canonical checkpoint and interval hashes. It adds the seven new-token
checkpoints and applies deployment baselines at the qualified CREATE blocks.
Reference observations never repair state.

All **163,548 initialized observations** match, including **63,759 observations
without a new balance event in that block**. All **99,265 prior-token output
rows** retain their complete row/value sets in every block. A final independent
RPC snapshot checks **all 315 retained holders of the seven new tokens**,
including **162 zeros**, with no mismatches.

These are bounded holder tests, not a global holder enumeration. Checkpoint
initialization remains explicit. Repeated unchanged RPC observations need
retained state; the protobuf schema matches the RPC solution, but raw event
row counts are not universally identical.

## Regression checks and remaining scope

All **195 workspace Rust library/binary tests** pass. New captured cases cover
each of the seven profiles. Injecting the additional beacon-pointer change
into both captured bridge-token cases makes processing fail; removing the new
pin reproduces the missed dependency. Other regressions cover changes followed
by restores, code changes, system calls, reverted/failed writes, account-scoped
slots, invalid configurations and both RPC qualification boundaries.

Clippy with warnings denied, workspace WASM compilation, targeted formatting and
diff checks pass. Generated-protobuf doctests are outside this test result.
The [import scan](evidence/ranked-proxy-wasm-imports.json) contains no RPC
functions. [Artifact digests](evidence/ranked-proxy-artifacts.json) bind the
package, WASM, layouts, fixtures and actual output.

The [remaining cohort](evidence/ranked-proxy-summary.json) contains ranks
**104, 107, 117, 121, 122, 124, 140, 143, 144 and 146**. Their missing source,
unreviewed storage/getter behavior or external reward dependencies still require
work. Earlier reports keep their original scope and package identities.
Ethereum, Base, HyperEVM and Arc remain the next requested network
qualifications after BSC; these BSC results do not establish their token parity.
