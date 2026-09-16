# Top ERC-20 tokens from the BSC RPC balances stream

On 2026-09-16, streamed `erc20/balances` v0.3.4 `map_events` for finalized BSC
blocks **122284478–122284989** (512 blocks). The stream contained **108,845 rows
from 1,353 contracts**. Ranked contracts by emitted balance rows, with contract
address as the tie breaker. This measures activity in the balances stream,
including approvals and large bursts; it is not a market-cap ranking.

**No top-ten token passed exact row parity.** Nine have candidate mapping values
that matched the checked RPC values; the fourth-ranked token has a real mismatch
between its mapping word and `balanceOf`. No new layout was promoted. The mapper
and its single `map_events` module are unchanged.

## Method

- Captured the first 32 blocks plus up to eight evenly spaced active blocks per
  top token, including first/last activity. This yielded **89 unique Extended
  blocks** and included all activity of tokens with fewer than eight active blocks.
- Matched Firehose block and parent hashes to finalized RPC headers. Bound
  `balanceOf` and raw storage reads to EIP-1898 canonical block hashes; checked
  header stability again after the calls.
- For each token, used the earlier half of its sampled active blocks to select
  a unique candidate mapping with at least two nonzero holders and no observed
  end-value disagreement. Froze that candidate for the later half (holdout).
- Compared exact `(block, contract, holder, amount)` rows. Missing reference rows
  stay missing, including zeros. No RPC bootstrap, cross-gap carry-forward or
  implicit zero was used. Samples are bounded evidence, not full-range coverage.
- Audited **14,020 before/after candidate values** against direct RPC:
  **14,018 matches, two mismatches, zero RPC errors**. Every shared end-of-block
  value matched the streamed reference; the two failures are pre-block values.
- Replayed the actual native `project` function used by `map_events`. WBNB used
  the existing reviewed fixture. Every other token used an explicitly
  unqualified balance-slot hypothesis with **no ignored slots/mappings**. Those
  strict mapper failures require layout review; they are not wrong emitted values.
  Replay includes all 89 sampled blocks per token, even when the RPC reference
  emits no rows for that token, so extra mapper output would also fail parity.
- Checked runtime hashes at sampled boundaries. Equal runtime hashes alone do
  not establish direct, non-proxy semantics or rule out intervening upgrades.

## Results

Symbols were read from each contract and are display labels, not identity checks.
Candidate slots are decimal. Shared/missing counts below concern **holdout mapping
hypotheses**, not successful strict mapper blocks or the entire 512-block stream.

| Rank | Symbol | RPC stream rows | Candidate slot | Shared holdout rows | Missing holdout rows | Direct RPC failures / checks | Strict mapper errors across 89 sampled blocks |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | USDT | 37,716 | 1 | 2,474 | 975 | 0 / 9,538 | 88 |
| 2 | WBNB | 11,281 | 3 | 665 | 419 | 0 / 2,448 | **0**, reviewed layout |
| 3 | USDC | 4,016 | 1 | 277 | 157 | 0 / 930 | 40 |
| 4 | 无头苍蝇 | 3,017 | 8 | 4 | 5 | **2 / 16** | 3 |
| 5 | REAL | 1,686 | 0 | 20 | 12 | 0 / 78 | 2 |
| 6 | KII | 1,379 | 51 | 106 | 30 | 0 / 376 | 6 |
| 7 | Cake | 1,312 | 1 | 93 | 82 | 0 / 356 | 3 |
| 8 | 万福金安 | 1,233 | 2 | 4 | 1,211 | 0 / 24 | 1 |
| 9 | BTCB | 1,221 | 1 | 52 | 41 | 0 / 238 | 9 |
| 10 | $ARC | 1,016 | 0 | 6 | 4 | 0 / 16 | 4 |

All contract addresses, code hashes, active block lists, sample hashes, layout
hypotheses and strict mapper failures are in the JSON evidence linked below.
Even the reviewed WBNB control has missing output rows: its emitted storage
values are correct, but the RPC module also emits balances for addresses without
a persisted balance write in that block.

Large bursts matter. Rank 4 has 3,004 missing rows in its earlier samples; rank 10
has 1,004. Rank 8 has 1,211 in holdout. A balance mapping found from a few real
writes cannot account for all these reference rows.

## Confirmed counterexample to direct mapping semantics

Contract `0x2b74ac567edaa7a65237b4f92042667e784439d3`, candidate mapping base 8:

| Before block | Holder | Firehose old word | RPC storage word | RPC `balanceOf` raw units |
| --- | --- | ---: | ---: | ---: |
| 122284599 | `0xe22f75f89eb7542aa56dac0dcfc3ef1d6fe9880c` | 0 | 0 | 8000000000000000000 |
| 122284986 | `0x12bc56a931f888e0f669b1f36eea7c5bcc681b21` | 0 | 0 | 8000000000000000000 |

Each row uses the same canonical parent hash for `balanceOf` and
`eth_getStorageAt`. RPC raw storage agrees with Firehose, so the proposed mapping
word does not equal the public balance for these holders. The exact contract
logic was not qualified; this is evidence to reject the direct-mapping hypothesis,
not to guess a default or adjustment. One failure was in discovery, one in holdout.

## Evidence and reproduction

- [Ranked RPC sample](evidence/top-tokens-ranking.json): selected top ten with
  active heights, total counts, reference package/capture digests and boundaries.
  The complete 1,353-contract ranking stays in the local capture directory.
- [Final parity report](evidence/top-tokens-parity.json): all ten results; the
  investigation completed but the parity gate returns `mismatch` and exit code 1.
- [Two mismatch checks](evidence/top-tokens-mismatches.json): canonical hashes,
  storage keys and both RPC observations.
- [Initial minimal-layout diagnostic](evidence/top-tokens-initial-survey.json):
  preserved before applying the existing reviewed WBNB control. WBNB had 22
  unknown-write errors with empty ignore lists; its reviewed allowance mapping
  removed those errors without changing balances or filling missing rows.

Use the Rust commands in the package README with `rank-tokens --start 122284478
--blocks 512 --top 10`. For the complete sample above, also run `capture-blocks
--start 122284478 --blocks 32` into a second output directory and pass both
directories to `test-ranked --block-dir`. Use the existing
`tests/fixtures/verified-layouts.json` for `--layouts`. A fresh ranking omitting
`--start` selects a new finalized window and can produce a different top ten.

Locally retained raw data is under `out/top-tokens-512`,
`out/top-tokens-discovery-blocks`, `out/top-tokens-active-blocks`, and
`out/top-tokens-parity-complete`. Checks and observations have digests in the final
report. Large raw captures are intentionally not committed.

Validation: 75 workspace Rust library/binary tests pass, native-tool Clippy passes
with warnings denied, and the complete workspace passes the WASM target check.
The expected live parity failure is separate from the tooling's regression tests.

## Next qualification work

USDT, USDC and BTCB are strong next candidates for semantic and allowance-layout
review, followed by independent continuous-window mapper/RPC comparisons. The
tests do not establish support for proxies, reflection, rebasing or default
balances. Unknown writes remain failures until their roles are verified.

Exact row parity also needs a design for reference-requested holders with no
write. The current stateless map cannot recover an untouched holder's value from
storage changes alone. A verified state bootstrap plus complete updates, or
other complete execution data, is needed before promising identical coverage.
