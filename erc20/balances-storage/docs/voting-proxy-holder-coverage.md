# BSC voting, burn-participant and proxy follow-up

Six more candidates from the immutable top-250 ranking now have qualified
layouts: **GOATED (208), TAC (212), TCOM (214), USDon (222), ARX (240), and
S315 (246)**. The [combined fixture](../tests/fixtures/bsc-voting-proxy-layouts.json)
contains **239 profiles**. Ten candidates from ranks 201–250 and LBP (143)
remain unqualified. The [summary](evidence/voting-proxy-summary.json) binds the
individual reports and records those remaining candidates.

This batch adds reviewed configurations, captured Rust regressions and audit
evidence. Production logic is unchanged: one RPC-free `map_events`, shared
`evm.balances.v1.Events`, caller-qualified layouts and default `[]`. No token
allowlist, production store or protobuf is added.

## Direct getters with separate bookkeeping

The [voting/burn qualification](evidence/voting-burn-qualification.json) binds
verified source to historical runtime at both ends of blocks
**122288006–122289029**, inclusive. Each getter returns `_balances[account]`
from mapping root zero; independent zero, one, 123 and maximum-word overrides
match. The configured bookkeeping fields do not contribute to public balances.

| Token | Separate storage behavior | Qualified rule |
| --- | --- | --- |
| TCOM / `MyToken` | OpenZeppelin `Trace208` voting histories, using block numbers | Delegate-history mapping 9 and total-history root 10, `block_number` clock |
| ARX / `PeerToken` | The same packed history format; `BaseToken` overrides the clock to timestamps | Mapping 9 and root 10, `timestamp` clock |
| S315 / `E315` | `burnUsers` address array, appended when the holder's recorded burn amount is zero | Explicit address-list root 13 |

The source-reviewed voting entries pack a uint48 clock and uint208 value in
one word. Their insertion routine updates the last entry for an equal clock
and appends for a new clock. Fresh `clock()` calls match the configured clock
at both historical boundaries. The source's other fields are explicitly
configured; arbitrary storage writes remain errors.

The original 1,024-block interval contains no writes that require these
checkpoint/list rules. Removing the special rule still passes that interval;
the initially unrecognized writes were resolved by the reviewed ordinary
bookkeeping roots. Separate historical captures therefore exercise the rules.

## Captured mint and burn activity

The [activity search](evidence/voting-burn-activity.json) preserves canonical
storage observations and finds three actual blocks:

| Token | Block | Observed activity | Packaged balance checks |
| --- | ---: | --- | ---: |
| TCOM | 57207279 | Constructor mint; total supply becomes `1000000000000000000000000000` | 1 |
| ARX | 104975334 | Mint; total supply changes from zero to `1000000000000000000` | 1 |
| S315 | 122259869 | Burn-participant list grows from 108,779 to 108,780 entries | 4 |

TCOM's separate activity layout pins the deployment block/hash and passes the
pre-CREATE empty-code/zero-nonce checks. That deployment variant belongs to this
separate activity audit; the original-interval fixture retains the previously
qualified direct layout. ARX and S315 are already deployed at their activity
blocks. Searches retain adjacent observations proving the transitions; they do
not claim the earliest mint or earliest burn.

The actual packaged mapper matches all **six independent RPC balances** in
the [TCOM](evidence/voting-burn-activity-rpc-214.json),
[ARX](evidence/voting-burn-activity-rpc-240.json) and
[S315](evidence/voting-burn-activity-rpc-246.json) captures. These single-block
activity checks are separate from the original interval and are not added to
its aggregate holder counts or presented as complete historical holder replays.

Nine filtered Extended fixtures cover ordinary activity for all six new tokens
and the three older activity blocks. They retain complete token-writing
transactions and preserve the full block's output. Expected amounts come from
completed independent RPC audits: **25 balances** in total. The Rust regressions
reproduce rejection when each active checkpoint/list rule is removed, and both
voting mint captures reject the other clock type.

The first historical fetch used a streaming endpoint that does not expose the
Firehose Fetch service. Its [incomplete report](evidence/voting-burn-activity-initial.json),
[fetch report](evidence/voting-burn-fetch-initial.json) and
[`Unimplemented` service error](evidence/voting-burn-fetch-initial.txt) are retained.
The completed capture used the Firehose block-fetch endpoint. No failed request
was treated as an empty block.

## Three independently pinned proxies

The [proxy qualification](evidence/remaining-proxy-qualification.json) binds
both proxy and implementation source to the historical runtimes. The getters
are direct mapping reads, with no balance calculation or external dependency:

| Token | Implementation | Balance mapping |
| --- | --- | --- |
| GOATED | `0x6227907d94ebe5e9710218ddd07d303c9195a919` (`GoatOFT`) | OpenZeppelin ERC-7201 ERC20 root |
| TAC | `0xc7d1a8ee7d2c1e7e3834d2e85b3ebe88cff7fb33` (`ERC20Plus`) | The same ERC-7201 root |
| USDon | `0x7655539c985743c3d29aff4c8ae2730f9360f7a6` (`USDon`) | Root 201 |

GOATED and TAC return the first mapping in `ERC20Storage`, rooted at
`0x52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00`.
The next root is the nested allowance mapping, followed by total supply, name
and symbol roots. USDon uses the corresponding fields at roots 201–205.
Independent word controls and recorded storage reads agree with these getters.

Each profile pins the implementation pointer and code hash. Existing guards
reject persisted implementation or dependency-code changes, including a change
and restore, even without holder writes. Transparent-proxy admin exceptions
still apply. The profiles explicitly permit only these reviewed ERC20 fields;
unrelated admin, role, bridge or compliance writes and long-string data writes
are not silently ignored. This is qualified behavior over the audited interval,
not a claim to have exercised every administrative or cross-chain path.

## Holder and combined validation

| Evidence over the original 1,024 blocks | Voting/burn cohort | Proxy cohort |
| --- | ---: | ---: |
| Actual WASM emitted balances checked against RPC | 102 | 129 |
| Emitted zero checks | 5 | 23 |
| Initialized observed holders | 60 | 34 |
| Checkpoint balance/storage RPC reads | 120 | 68 |
| Initialized reference observations | 204 | 227 |
| Matches without a new balance event | 102 | 98 |
| Final retained holders checked against RPC | 60 | 34 |
| Final zeros | 39 | 16 |
| Cold unknown observations | 102 | 96 |
| Cold unknown nonzero observations | 23 | 41 |

The [voting/burn](evidence/voting-burn-rpc.json) and
[proxy](evidence/remaining-proxy-rpc.json) emitted audits have zero mismatches.
Actual-WASM retained-state replays for the
[voting/burn](evidence/voting-burn-wasm-holders.json) and
[proxy](evidence/remaining-proxy-wasm-holders.json) cohorts match every initialized
observation and final holder. Reference rows never repair state and processing
does not make balance RPC calls. Cold unknowns remain unknown, including the
64 nonzero observations across the cohorts. These are observed-holder sets,
not an enumeration of every possible holder.

The [fresh combined capture](evidence/voting-proxy-combined.json) preserves
every protobuf field against the prior 233-profile capture and these two
disjoint cohorts. **238 of 239 profiles emit**; hLBP remains quiet in the original
interval. The evidence covers **105,337 previously RPC-verified emitted balances**,
including **14,739 zero checks**, and identical event histories supporting
**172,681 initialized observations** and **67,345 initialized carry-forward matches**.
The combined check binds immutable evidence, unchanged package bytes and fresh
canonical headers. It does not repeat those balance RPC calls or create a new
full-239 checkpoint/final snapshot. The six older activity checks stay separate.

All **226 workspace Rust library/binary tests** pass, together with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
Repacking preserves the exact audited artifacts:

- SPKG: `d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`.
- WASM: `861a879353a7fc7163a26c80f671fc3011e9ca3ee32a65dfc0badd99ac9794b6`.

The remaining ranks in 201–250 are **203, 204, 209, 210, 220, 224, 236, 238,
239 and 248**. LBP (143) remains outside production despite its host-only reward
model; YBC (238) still has an unqualified external reward calculation. Continue
those reviews, then apply independent qualification to
[Ethereum, Base, HyperEVM and Arc](network-expansion.md).
