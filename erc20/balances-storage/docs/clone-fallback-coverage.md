# BSC clone with a stored zero-word fallback

The [subsequent eight-token qualification](remaining-ranked-coverage.md) extends
coverage to 149 profiles. This report retains its original 141-profile scope.

Rank **144**, `0xc4c5f33e3860e1ad31d52fd377cc39f74784d19f`, is now qualified
for the same BSC interval, **122288006–122289029**. The
[combined fixture](../tests/fixtures/bsc-clone-fallback-layouts.json) contains
141 profiles, preserving all previous 140 parameters. Nine candidates from
ranks 101–150 remain unqualified. This extends tested coverage using existing
layout rules; production still has one RPC-free `map_events` and shared
`evm.balances.v1.Events`.

## Getter and dependency review

The token is a canonical EIP-1167 clone targeting
`0x75b2fd32294ec37c70e8f3b1bbce81b6cc71f0c6`. The implementation's complete
getter body at PCs 4165–4271 reads mapping root 0. A nonzero word is returned
unchanged. For a zero word, it returns the unsigned value in storage slot 8
when positive, otherwise zero. Both branches return through the same ABI
wrapper; there are no other getter storage reads, external calls or caller
conditions. Verified implementation source was unavailable, so this result
uses historical bytecode and both getter branches, supplemented by RPC probes.

The [qualification record](evidence/clone-fallback-qualification.json) includes
the disassembly, runtime identity and **80 read-only override checks**: five
holder contexts, four raw balance words and four fallback values, including
zero and uint256 maximum. Both clone and implementation runtime hashes are
pinned at both boundaries. The fallback, **1,000,000,000 raw token units**, is
pinned to **slot 8** with `zero_balance`; it is not an immutable constant.
Any persisted change requires requalification and rebuilding dependent holder
state, even without a balance write. Only allowance mapping root 1 is ignored.

## Output and holders

The [actual packaged WASM audit](evidence/clone-fallback-rpc.json) checks all
1,024 blocks and matches **22 emitted balances in 11 active blocks**, with no
RPC mismatches. A separate complete clock capture binds all **1,013 empty
outputs** to canonical block identities. No emitted row exercises the zero-word
fallback in this interval; that path is exercised by the override probes and
the real holder checkpoint below.

The [native holder audit](evidence/clone-fallback-holder-coverage.json) initializes
**84 observed holders** using **168 explicit balance/storage RPC reads**.
**81 have zero stored words but positive RPC fallback balances.** All 130
initialized reference observations match. Without a checkpoint, 108 nonzero
observations remain unknown; the tool does not misclassify those as zero.

The [independent actual-WASM replay](evidence/clone-fallback-wasm-holders.json)
also matches all **130 observations**, including **108 without a new event**.
A final canonical RPC snapshot matches every retained holder: **84 total,
81 with the fallback amount**. Reference observations never repair state, and
processing makes no balance RPC calls. A positive fallback for arbitrary
addresses does not provide a way to enumerate global holders.

## Regression and aggregate scope

Three captured Rust tests check emitted balances, all 84 checkpoint projections
(including the failure of raw-word projection for 81 holders), and fallback
dependency changes without balance activity. **198 workspace library/binary
tests pass**, along with Clippy with warnings denied, targeted formatting and
diff checks. Repackaging preserves the exact package and WASM digests from the
previous 140-token audit; no ingestion logic or package metadata changed.

The [aggregate evidence](evidence/clone-fallback-summary.json) covers
**99,811 emitted RPC checks** and **163,678 initialized holder observations**,
with zero mismatches. These totals combine the previous 140-token WASM run and
the additional one-token run on the identical package and canonical interval.
A combined native replay with all 141 layouts matches their complete union at
every block. This is not a new combined 141-token WASM stream.

Ranks **104, 107, 117, 121, 122, 124, 140, 143 and 146** remain separate work.
The [next seven getter probes](evidence/remaining-ranked-getter-probes.json)
and [native hypothesis scan](evidence/remaining-ranked-layout-scan.json) are
retained as unqualified diagnostics. Their successful getter paths read direct
balance words. Six tentative layouts have no unresolved writes in the interval;
rank 104 fails at blocks 122288448 and 122288504 on literal slot 2. Its
`totalSupply()` dispatcher selects PC 1304 and returns slot 2 at PC 1327;
qualifying that explicit field and auditing the seven actual outputs/holders
are still required. The two failed blocks are retained, not counted as parity.

Ethereum, Base, HyperEVM and Arc follow the BSC campaign, with independent
runtime, Extended-block, RPC and holder qualification. Existing RPC prerequisite
probes do not establish token parity on those networks.
