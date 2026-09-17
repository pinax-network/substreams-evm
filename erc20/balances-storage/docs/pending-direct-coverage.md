# Nine additional direct BSC layouts

Ranks **151, 167, 168, 172, 174, 194, 196, 198 and 199** pass historical
qualification across **122288006–122289029**, inclusive. The
[combined fixture](../tests/fixtures/bsc-pending-direct-layouts.json) now has
**197 profiles**. Three candidates from the reviewed top 200 remain outside it:
LBP (143), VSD (179) and hLBP (189).

Production remains one RPC-free `map_events`, shared `evm.balances.v1.Events`,
caller-qualified layouts and default parameters `[]`. This batch adds explicit
test profiles and captured Rust regressions. It makes no production logic change.

## Getter and bookkeeping qualification

[Qualification evidence](evidence/pending-direct-qualification.json) preserves
each historical runtime, reviewed getter instructions, independent controls and
the original unresolved-storage errors. Verified source is unavailable for
these nine contracts: their qualification is based on reviewed runtime bytecode,
canonical historical calls and captured execution traces.

| Rank | Token symbol | Balance mapping | Emitted RPC checks |
| --- | --- | --- | --- |
| 151 | LUMY | 0 | 80 |
| 167 | 喵喵币 | 0 | 82 |
| 168 | AARON | 0 | 92 |
| 172 | TAIYI | 0 | 70 |
| 174 | 天轨 | 1 | 54 |
| 194 | USDB | 0 | 58 |
| 196 | 215-NO | 0 | 29 |
| 198 | BLove | 0 | 61 |
| 199 | 215-YES | 0 | 27 |

Each successful `balanceOf` path validates the address argument, reads its
mapping word and returns it unchanged. No successful path calls another contract
or selects a different result based on caller or other state. Runtime identities
are checked at both historical boundaries. All 36 balance-word controls (zero,
one, 123 and uint256 maximum), 36 address/caller controls and 72 controls varying
explicit non-balance fields pass. These controls supplement the instruction
review; value equality alone is not a layout proof.

Three candidates initially failed on additional bookkeeping writes:

- Rank 167 uses mapping 30, with a separate public raw getter at selector
  `b54ee8dd` and an arithmetic write path at PC 8992. Its balance getter reads
  only mapping 0. The explicit `other_mapping_slots` entry accounts for this
  reviewed field without assigning an unproven economic meaning to it.
- Ranks 196 and 199 use four-word records under three nested mapping hashes
  rooted at 16. The reviewed creation path writes offsets 0–3; offset 2 is a
  timestamp and offset 3 contains a low-byte flag. Another path updates offsets
  0 and 2. The explicit `other_mapping_words` width is four. A changed offset
  initially looked like an unrelated scalar key; it is not configured as one.
  Their allowance calls revert, so no allowance root is inferred.

[Canonical transaction traces](evidence/pending-direct-bookkeeping.json) record
the write instructions, words and Keccak ancestry. Captures at blocks
**122288021** and **122288022** preserve all three original failure cases.
Removing mapping 30 or truncating the record width reproduces rejection in Rust.
The NO fixture witnesses offset 3; the YES fixture witnesses offset 2. A width
of three is therefore not claimed to fail on the captured YES transaction.
Nine captured fixtures cover all nine tokens and 33 independent RPC balances.

## Actual WASM and retained holders

The [complete native scan](evidence/pending-direct-native-scan.json) and
[packaged-WASM audit](evidence/pending-direct-rpc.json) both produce **553 balances**.
All 553 canonical historical RPC checks match, including **24 zeros**. Complete
canonical clock evidence covers 180 blocks with output and **844 empty blocks**.
Every configured token emits during the 1,024-block window.

The [native holder audit](evidence/pending-direct-holder-coverage.json) initializes
**313 observed holders**, using **626 explicit checkpoint balance/storage reads**.
All **870 reference observations** match. Cold state instead has **288 unknown
observations**, including **42 nonzero values**, so initialization or earlier
history remains necessary.

The [independent actual-WASM replay](evidence/pending-direct-wasm-holders.json)
matches all 870 observations, including **317 initialized carry-forward matches**
without a new event. A final canonical RPC snapshot matches all **313 retained
holders**, including **81 zeros**. Reference values never repair retained state;
processing performs no balance RPC calls. The per-token `carry_forward_matches`
field inherited from the native report counts only 29 cold-state observations
after an earlier event; it is distinct from the 317 initialized matches.

## Combined scope and remaining work

The [combined 197-profile actual-WASM regression](evidence/pending-direct-combined.json)
compares every protobuf event field with the complete union of six disjoint,
historically RPC-audited cohorts on the same canonical interval. All **103,575
previously checked balances** match. This is a fresh capture with fresh canonical
headers and reused immutable balance-check evidence, not 103,575 new RPC calls.

The first five audits used the earlier package; the new nine-token audit and
combined run use the shareholder-removal fix. Both package identities and the
source evidence digests are retained. Their identical event histories preserve
the earlier **169,670 initialized holder observations**, including **66,096
initialized carry-forward matches**. No fresh combined 197-token checkpoint or
final holder snapshot is claimed. The disjoint audits contain **14,548 emitted
zero checks** in total.

Current SPKG SHA-256:
`d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`.
Current WASM SHA-256:
`861a879353a7fc7163a26c80f671fc3011e9ca3ee32a65dfc0badd99ac9794b6`.

All **213 workspace Rust library/binary tests pass**, together with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
Repackaging preserves the exact audited package and WASM digests above.

Remaining work is explicit:

- [LBP's external pending rewards](lbp-reward-mismatch.md) change balances for
  33 verified holders without a raw balance write. A correct initial checkpoint
  becomes stale. This requires a separate design decision about persistent
  holder/reward state and is not fixed by these direct layouts.
- VSD still needs its proxy dependencies and non-balance fields qualified.
- [hLBP's quiet-holder audit](hlbp-quiet-holder-coverage.md) matches 88 initialized
  observations and four final holders, but has no captured changed-balance case.
  Those observations are separate from the aggregate above.

The [shareholder-removal audit](shareholder-removal-coverage.md) also remains a
separate interval. It captures a real tail pop; swap-and-pop has synthetic
coverage but no captured real swap. These results establish bounded token and
observed-holder parity, not global holder enumeration or universal ERC-20
support. The requested sequence after BSC is
[Ethereum, Base, HyperEVM and Arc](network-expansion.md), with independent
qualification for each network.
