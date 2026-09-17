# BSC candidates 101–150 and stored-share balances

The next 50 contracts from the immutable RPC reference ranking add **33 qualified
profiles**, bringing the combined fixture to **133 tokens**. Seventeen candidates
remain under review. This is evidence for the tested historical runtimes and
interval, not a claim that every BSC token or every possible holder is covered.

The [combined fixture](../tests/fixtures/bsc-ranks101-150-layouts.json) keeps the
previous 100 profiles intact. Production still exposes one RPC-free `map_events`,
shared `evm.balances.v1.Events`, caller-qualified layouts and default parameters
`[]`. All executable qualification, comparison and regression tooling is Rust.

## Qualification of the new profiles

The [qualification evidence](evidence/ranks101-150-qualification.json) records
each getter, storage layout, historical runtime identity and independent RPC
controls. The original interval is **122288006–122289029**, inclusive: 1,024
consecutive Extended blocks.

- **25 direct getters:** ranks 102, 103, 109, 110, 111, 112, 113, 115, 116,
  119, 120, 125, 127, 129, 131, 133, 134, 135, 137, 138, 139, 141, 147,
  148 and 150. Verified source is bound to the historical runtime. The complete
  getter returns the balance mapping; zero, one, 123 and uint256-max overrides
  independently confirm that behavior.
- **Seven reviewed proxy-family profiles:** ranks 101, 105, 106, 114, 123,
  126 and 132. Exact forwarding runtimes, implementation or beacon dependencies,
  pointers and getter controls are verified at the tested interval boundaries.
  A shared symbol or matching outer proxy bytecode alone is insufficient.
- **sPro, rank 118:** a stored-share getter requires unsigned division by a
  pinned scalar. Its qualification uses complete bytecode and trace review;
  verified source was unavailable.

Rank 127, WCOL, had only one observed nonzero holder and did not meet the
automatic candidate heuristic. Its public `balanceOf` mapping at slot 3 was
qualified independently from runtime-bound Solmate source and getter controls.
The heuristic was not weakened.

Non-balance fields are explicit reviewed scalar or mapping roots. Ranks 131 and
148 include EnumerableSet metadata, but arbitrary array-element updates are
not qualified by those entries and still stop processing. No broad storage
range is silently ignored.

## Why sPro differed from the raw mapping

For `0xdfe1308fb3ef1dc2f87b4ffaef5ebdf80be1e4ea`, the complete `balanceOf`
selector and ABI path reaches mapping 12 at PC 4318, then scalar 11 at PC 4321.
The checked division helper executes unsigned `DIV` at PC 4375. The successful
path has no further storage read or external call. Its zero-divisor branches
revert. The separate allowance getter reads nested mapping 13 at PC 428.

At canonical block **122288026**, holder
`0xc0021e0849fadefb98761f40829009905dbd8ee8` has raw shares
`95590469363238633351927479563642395151229371442472908023066917122628824706604`.
The divisor is
`2795910913004693103940856641295427684519894749984128958137870`.
RPC returns their floor quotient, **34189383116112962**. Emitting raw shares
therefore cannot match `balanceOf`.

The optional `balance_divisor` rule applies that reviewed calculation only
after raw-word continuity checks. The checkpoint tool uses the same projection.
Qualification checks the divisor at both boundaries. Every persisted divisor
change stops processing, including change-and-restore, system-call changes and
blocks without holder writes. The error requires requalification and rebuilding
retained balances before resuming. Reverted changes and unchanged writes do not
invalidate the interval.

The rule requires a positive 32-byte divisor and a distinct protected scalar
slot. It cannot combine with another balance formula or a deployment baseline.
This supports a verified **stable-divisor interval**; it does not implement
rebasing across rate changes or silently repair balances of untouched holders.

Across five address contexts, **140 independent RPC state-override controls**
cover zero, one, 123, both sides of the real divisor, the divisor itself and
uint256 maximum, using four divisors. All match unsigned floor division. Five
additional zero-divisor traces revert. The complete interval has zero persisted
divisor changes and **78 emitted sPro balances in 39 blocks**, all covered by
the packaged WASM audit.

## Packaged output and holder validation

The [actual WASM RPC audit](evidence/ranks101-150-rpc.json) checks **99,265
emitted balances**, including **14,097 zeros**, across all 133 tokens with
**zero mismatches**. The 33 new profiles contribute **2,996** of those checks.
Every block is delivered. The [previous 100-token output comparison](evidence/ranks101-150-top100-regression.json)
confirms their complete emitted row/value sets remain identical in every block:
**96,269 unchanged regression rows**.

The [holder replay](evidence/ranks101-150-holder-coverage.json) separately checks
retained values against the original RPC reference. Its initialization uses
**52,468 bounded holder checkpoints**, with **104,936 balance/storage RPC
reads**. These are explicit test initialization costs, not processing calls or
proof of a complete global holder enumeration. Immutable-zero and deployment
baselines remain separately reported. Cold-start unknowns remain explicit.
All **162,559 initialized reference observations** match, with zero initialized
unknowns or value mismatches and zero processing balance RPC calls. The native
cold replay records **4,422** matches carried from earlier observed updates;
its carry count deliberately excludes values available only from the checkpoint.

The [independent reconstruction from actual WASM](evidence/ranks101-150-wasm-holders.json)
applies only emitted storage balances after initialization, checks every
reference observation for the 33 new tokens, then checks the entire retained
holder set for those tokens against RPC at the final canonical block. Reference
observations never repair retained state. It also verifies that the actual
output equals the audited row set and that both replays use the same block
hashes. These checks distinguish balance correctness from identical raw event
row counts: repeated unchanged RPC observations need retained state.
This reconstruction matches **5,138 observations**, including **2,142** with
no update in that block (using either the checkpoint or earlier emitted state).
The final snapshot checks **all 1,282 retained holders**, including **674 zeros**,
with zero RPC mismatches. For sPro alone, all **156 observations** match: 78
newly emitted balances and 78 values retained from initialization.

All **188 workspace Rust library/binary tests** pass, including captured
historical RPC regressions for each new profile. Removing sPro's divisor
reproduces the original error. Additional tests cover uint256 rounding, raw
continuity, holder-silent and restored dependency changes, reverted and failed
writes, invalid configurations and both RPC qualification boundaries.
Clippy with warnings denied, workspace WASM compilation, targeted formatting
and diff checks pass. Generated-protobuf doctests are outside this test result.
The [WASM import scan](evidence/ranks101-150-wasm-imports.json) contains no RPC
functions; [artifact digests](evidence/ranks101-150-artifacts.json) bind the
package, WASM, layouts, fixtures and actual output.

## Remaining candidates and network scope

The [campaign summary](evidence/ranks101-150-summary.json) retains the 17
unqualified candidates: ranks **104, 107, 108, 117, 121, 122, 124, 128, 130,
136, 140, 142, 143, 144, 145, 146 and 149**. They need complete runtime,
dependency or getter review before promotion.

The [next proxy inventory](evidence/ranks101-150-pending-proxies.json) resolves
the dependencies of ranks 108, 122, 128, 130, 136, 142, 145 and 149 at both
boundaries. Their pointers and runtimes are unchanged there. Seven have
runtime-bound implementation source; rank 122 still requires bytecode review.
This inventory alone does not qualify their getters or storage rules.

Rank 144 is a new minimal-proxy implementation whose zero-word getter returns
1,000,000,000 in all five tested address contexts. The
[probe results](evidence/rank144-zero-probes.json) are retained, but its full
getter and bookkeeping qualification is not complete.

Rank 143, LBP, can add rewards from an external hashrate contract to its raw
balance. Runtime-bound source shows dependencies on holder shares, reward
indices, global accumulators and time. The [80 historical probes](evidence/lbp-reward-probes.json)
over eight observed holders and ten canonical blocks found no nonzero pending
reward or raw-value mismatch. That limited result does not justify ignoring
the additional getter branch. LBP remains unqualified; full support must also
handle holders whose rewards change without a token balance-slot write.

After the BSC review, qualification continues on **Ethereum, Base, HyperEVM
and Arc**, in that order, using the [same per-network process](network-expansion.md).
Their preliminary RPC probes are not token, Extended-block or holder-parity
evidence. BSC results are not automatically transferable to another chain.
