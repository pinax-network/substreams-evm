# All 50 ranked BSC tokens

The [50-token fixture](../tests/fixtures/bsc-top50-layouts.json) covers every
contract selected from the original 1,024-block RPC stream. The eight additions
are qualified through historical bytecode review, canonical getter traces and
read-only storage controls. Verified Solidity source was unavailable for these
eight contracts; the evidence does not claim source verification.

| Rank | Contract | Balance base | Reviewed non-balance fields |
| ---: | --- | ---: | --- |
| 16 | `0x66661c7229901f568f16bd1551b3ba826f83ce49` | 0 | Allowances 1; supply/metadata 2–4 |
| 19 | `0xc69b16cf18cea1e5d0bb6a1a9db802097790ddd2` | 0 | Allowances 1; supply/metadata 2–4; owner 5; packed pending owner/pause 6 |
| 23 | `0x10d4183389e99233db3cc981c43443ebd28ebd5e` | 1 | Owner 0; allowances 2; supply/metadata 3–5 |
| 29 | `0x8d4967f460282145ff1ceca9bf78e1c2a358d176` | 0 | Allowances 1; supply/metadata/owner 2–5 |
| 30 | `0xcc43a8d700162b344b787d757a1b7fed32b78df3` | 0 | Allowances 1; supply/metadata/owner 2–5 |
| 33 | `0x2bf97abb0acab18157ad16fa3cb49396ff888888` | 0 | Allowances 1; supply/metadata/owner/pending owner 2–6 |
| 41 | `0xe448283a8b6331ef5061b35a45ba99fef9db0bd7` | 0 | Allowances 1; supply/metadata/owner 2–5 |
| 46 | `0xe27023ef180541ce8dce4f507c7914b1f59fd96c` | 0 | Allowances 1; supply/metadata 2–4; packed pause/owner 5 |

Ranks count balance rows in this RPC stream, not market capitalization. All
profiles are explicit test parameters. Production defaults remain `[]`.

## Getter review and controls

The [qualification record](evidence/top50-qualification.json) retains runtime
bytes/hashes, executed getter instructions, mapping roots, canonical block
identities, trace digests and override results. Review follows selector dispatch,
ABI decoding and return encoding, not just the observed SLOAD. The balance
bodies directly hash the address with the configured base, load one word and
return it; no special-holder branch, caller dependency, multiplier or external
call appears in those bodies. The corresponding SLOAD PCs are 756, 1295, 956,
335, 335, 448, 335 and 2734 in rank order.

Every getter passes independent raw-word controls for **0, 1, 123 and uint256
max**. Additional zero-address and all-ones-address controls, with different
callers, return the explicitly overridden raw words. Standard non-balance
getters have separate traces and controls. Allowance hash preimages identify
their nested mapping roots; supply, metadata and administrative reads identify
the explicit scalar fields above. Unknown writes still stop processing.
Role mappings, long dynamic metadata and unreviewed administrative storage are
not silently added to the permitted fields.

Ranks 29, 30 and 41 have byte-for-byte runtime equality with the previously
qualified rank-15 deployment token. They were already deployed at the beginning
of this window, so their profiles deliberately do not inherit rank 15's later
deployment block or zero-initialization rule. Runtime boundaries are checked at
122288005 and 122289029 for all eight additions. Ingestion continues to reject
runtime changes within the range.

Packed administration required a separate control: rank 46's owner address is
shifted eight bits above its pause flag in slot 5, while rank 19 packs its pause
flag above a pending-owner address in slot 6. Treating the entire rank-46 word
as the owner caused an initial **non-balance** control assertion to fail; it was
not a `balanceOf` mismatch. [Corrected packed-field controls](evidence/top50-packed-controls.json)
check both paused and unpaused values: the address getter returns 123 while
`balanceOf` remains the independently overridden raw value 456. No balance
projection change was needed.

## Actual WASM and holders

The [actual WASM audit](evidence/top50-rpc.json) checks **1,024 consecutive blocks,
122288006–122289029**: all **87,501 emitted balances** match independent
hash-pinned RPC, including **12,373 zeros**. All **50 configured tokens** emit
rows. The eight additions contribute **3,648 checks**.
[Per-token counts](evidence/top50-rpc-token-counts.json) retain their individual
coverage. This run completed without an RPC transport failure.

The [452-block holder replay](evidence/top50-final-holder-coverage.json), covering
122288006–122288457, matches all **68,670 reference observations across 50
tokens**. There are no unknown or incorrect initialized values. The additions
contribute **2,619 observations**, including **93 carried-forward balances**.
Setup uses **25,843 stored-holder checkpoints** (51,686 balance/storage RPC
reads), 3,001 computed initial balances and 19/184 holders initialized at the
previously qualified deployments. Reference values never repair candidate state;
processing makes no balance RPC calls.

Cold replay preserves **25,063 unknown observations**, including **13,200
nonzero values**, with zero incorrect known values. The additions account for
946 unknown observations, including 311 nonzero values. Checkpoint parity is
therefore distinct from complete cold-start holder coverage. A checkpoint or
complete history is still required for stored balances predating the stream.

The [full-window holder replay](evidence/top50-full-holder-coverage.json) then
extends the same test to **all 1,024 blocks, 122288006–122289029**. All
**142,989 reference observations across all 50 tokens** match initialized state,
with zero initialized unknowns or mismatches. There are **3,738 carried-forward
matches** without a new emission in that block. The eight additions contribute
**6,074 matching observations**, including **232 carried-forward values**.

The expanded checkpoint contains **47,515 existing stored holders**, verified
with **95,030 balance/storage RPC reads**. It also includes 3,001 independently
computed initial balances and **710/211 holders** initialized at the two pinned
deployments. The larger deployment counts reflect holders first observed later
in this window; earlier-deployed runtime siblings still require checkpoints.
Processing uses no balance RPC reads or reference-value repairs.

Full-window cold replay retains **51,750 unknown observations**, including
**27,701 nonzero balances**, with zero incorrect known values. The eight additions
account for 2,194 unknown observations, including 766 nonzero values. These are
preserved coverage gaps, not zeros. The full-window result includes the shorter
replay and must not be added to its counts.

The [capture record](evidence/top50-full-capture.json) retains hashes for the 572
new Extended blocks and the two capture-report digests. The 452-block prefix was
reused. Captures check the canonical block hash and parent before/after fetching;
replay independently checks continuity and canonical headers for the full range.
The retained reports' legacy `processing_rpc_calls: 0` field refers only to
balance reads during replay. It excludes RPC header verification. The tool now
reports `processing_balance_rpc_calls` and `processing_header_rpc_calls`
separately; this reporting correction does not alter the replay or its results.

## Regressions and limits

The eight [captured cases](../tests/fixtures/top50-final/cases.json) retain actual
transactions and original headers, with **19 independently queried historical
RPC balances**. Each transaction subset was verified to preserve its token's
full-block output before saving. The live stream audit and holder replay use
full blocks. A second Rust regression checks that the two packed administrative
words do not change the captured raw holder outputs.

**132 workspace Rust library/binary tests pass**, along with Clippy with warnings
denied, workspace WASM compilation and targeted formatting/diff checks. The
[artifact record](evidence/top50-artifacts.json) binds package, runtime, layout
and captured-case digests. Runtime WASM is unchanged and has no RPC imports.
There is still one `map_events`, shared `evm.balances.v1.Events`, no additional
cache/protobuf and no production token allowlist. All executable tooling and
tests are Rust.

This completes qualification of the **original ranked top 50**, not all BSC
contracts or every future administrative operation. The qualification windows
overlap earlier reports and their totals are not additive independent history.
Global holder enumeration, production bootstrap, identical raw event-row
coverage and qualification of further tokens remain separate work. The
checkpoint covers only addresses observed in its bounded reference window.

Reproduce the audit with the 50-token fixture and
`audit-rpc --start 122288006 --blocks 1024 --workers 2`, using an appropriate
Substreams endpoint and a fresh output directory. Reproduce the holder replay
with `holder-coverage`, `out/top50-1024/report.json`, its digest-matching reference
capture and consecutive Extended blocks for the stated range.
For the full-window replay, use blocks 122288006–122289029; the retained local
directory is `out/top50-full-holder-blocks`.
