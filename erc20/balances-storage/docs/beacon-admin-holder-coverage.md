# BSC beacon admin correction and five more profiles

The qualified BSC fixture now contains **246 of the original top 250 profiles**.
This follow-up adds BLESS (204), SAI (210), ALD (224), BY (239) and 金色派对
(248), and fixes a missing beacon admin dependency. LBP (143), TITAN (203), ORD
(209) and YBC (238) remain unqualified. The
[summary](evidence/beacon-admin-summary.json) binds the reports and remaining
addresses; the [fixture](../tests/fixtures/bsc-beacon-admin-layouts.json) records
the explicit layouts. These results cover **122288006–122289029**, inclusive.

The production interface remains one RPC-free `map_events`, shared
`evm.balances.v1.Events` and default `[]`. No production store or token allowlist
is added. All executable diagnostics and regression tests are Rust.

## BLESS: the beacon admin changes getter behavior

BLESS uses a beacon that is itself a transparent proxy. Its normal caller path
delegates `implementation()` to `DeBridgeTokenDeployer`, which reads token
implementation slot **151** in beacon storage. If the token becomes the beacon's
admin, that same selector instead returns the outer proxy's implementation.
The token then delegates to the wrong implementation for `balanceOf`.

A [read-only state override](evidence/bless-admin-probe.json) pins the holder's
raw balance to 123. With the historical admin, RPC returns 123; making the token
the admin makes RPC revert; a zero admin returns 123. Both implementation
pointers and the balance word remain unchanged. An injected Extended admin
write was accepted by the prior layout, demonstrating the missing dependency.
This is a simulated counterexample, not an observed on-chain admin change.

The optional `beacon_proxy.proxy_admin` now pins the reviewed admin slot and
address. Qualification checks both interval boundaries with canonical block
hashes. Processing rejects every persisted admin change, including a change
restored in the same block and a block without holder writes. Reverted writes
and unchanged words do not invalidate the layout. The slot belongs to the
beacon account, must differ from both beacon implementation pointers, and
requires the existing forwarding layer. The token cannot be its beacon's admin.

The [qualification report](evidence/bless-qualification.json) distinguishes
historically bound verified source for the token proxy, beacon delegate and
token implementation from manual historical-bytecode review of the outer
beacon, whose source was unavailable. The reviewed selector dispatch includes
both admin and non-admin branches, the forwarding call and return/revert paths.
Runtime hashes, both implementation pointers and the admin are pinned. Balance
mapping 151, allowance mapping 152, nonce mapping 302 and the listed scalar
fields are qualified. Unreviewed roles, initialization and long-string writes
remain errors.

## Four direct getters and transaction flags

The [direct-profile qualification](evidence/remaining250-qualification.json)
records the complete historical runtime and reviewed getter paths. Source was
unavailable for these four contracts; qualification explicitly uses bytecode,
traces and independent controls. Each reviewed getter unconditionally reads
one address-keyed mapping, without an external balance calculation.

| Rank | Token | Balance mapping | Separately reviewed bookkeeping |
| ---: | --- | ---: | --- |
| 210 | SAI | 1 | Allowance 2, supply 3, metadata 4–5 |
| 224 | ALD | 0 | Allowance 1, standard scalars 2–5, packed transaction flag at 7 |
| 239 | BY | 0 | Allowance 1, standard scalars 2–4, transaction guard at 10 |
| 248 | 金色派对 | 1 | Allowance 0, metadata 2–3, supply 4 |

SAI's older address decoder masks to 160 bits; the other reviewed paths validate
the canonical address encoding. Standard getter traces and word controls bind
the listed allowance, supply and metadata fields. Sixteen raw balance controls,
16 caller/holder context controls and 32 additional non-balance controls support
the four reviewed paths.

ALD's captured transaction at **122288026** sets and clears bit 160 at slot 7,
preserving the lower address bits. BY's captured transaction at **122288099**
changes guard slot 10 from 1 to 2 and back to 1. Reviewed write instructions and
Extended data agree. These explicit fields resolve the initial unknown writes;
removing either field makes its captured regression fail. This does not permit
arbitrary storage changes or qualify unobserved administrative paths.

## Packaged RPC and holder results

The actual packaged WASM passes both the [BLESS](evidence/bless-rpc.json) and
[four-profile](evidence/remaining250-rpc.json) RPC audits across 1,024 consecutive
canonical blocks. Explicit checkpoints come from independent historical balance
and storage reads. The [BLESS](evidence/bless-wasm-holders.json) and
[four-profile](evidence/remaining250-wasm-holders.json) retained-state replays use
the audited output and complete clocks. Reference rows never repair state.

| Measurement | BLESS | Four direct profiles | Total |
| --- | ---: | ---: | ---: |
| Emitted balances checked against RPC | 40 | 162 | 202 |
| Emitted zeros | 1 | 0 | 1 |
| Initialized observed holders | 18 | 56 | 74 |
| Checkpoint balance/storage reads | 36 | 112 | 148 |
| Initialized reference observations | 80 | 277 | 357 |
| Matches without a new balance event | 40 | 115 | 155 |
| Final holders checked against RPC | 18 | 56 | 74 |
| Final zero balances | 10 | 14 | 24 |
| Cold unknown observations | 40 | 112 | 152 |
| Cold unknown nonzero observations | 20 | 5 | 25 |

All emitted, initialized and final-holder checks match. Processing makes no
balance RPC calls. Cold unknowns remain unknown. The five captured Rust fixtures
retain complete token-writing transactions and reproduce the full blocks'
output, with **11 independently checked balances**. Additional regression tests
protect the admin dependency across reverted, failed, system and restored-write
cases and validate qualification at both boundaries.

All **235 workspace Rust library/binary tests** pass, together with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
Repacking reproduces both audited artifact hashes below.

The [combined capture](evidence/beacon-admin-combined.json) runs all 246 profiles
on the corrected package, comparing every protobuf field with the prior
241-profile capture plus these five new audits. It verifies fresh canonical
headers and layout dependencies. There are **245 emitting profiles**; hLBP's
older mint evidence remains separate. The combined output preserves:

- **105,621** RPC-verified emitted balances, including **14,746 zeros**.
- **173,176** initialized holder observations, including **67,556** matches
  without a new event, supported by identical event histories.

The package changed for the admin correction. This comparison binds both the
prior and current artifact digests; it does not claim identical package bytes,
repeat every balance RPC call, or create a fresh full-246 holder checkpoint or
final snapshot. Current artifact hashes are:

- SPKG: `f13ce00a08e3610ed607a168c1f2afac6fb92ac63561f945e97a55d7b4801503`
- WASM: `b5393ded0b6b30b6a7c79436abf421a616f4e8ebc6c430f4ce073a03e037fb6a`
- Prior SPKG: `d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`

## TITAN and ORD: matching samples conceal external dependencies

The [external-getter diagnostic](evidence/external250-diagnostic.json) checks
every observed reference holder at both boundaries: 50 observations for TITAN
and 60 for ORD. All **110 sampled raw-storage balances match RPC**. Both getters
nevertheless call another contract and remain unqualified.

TITAN reads its dependency address from slot 17 and calls two external getters.
With raw balance fixed at 123, making each external getter return 0, 1 or 17
produces 123, 125 or 157. These read-only simulations prove that external values
can change the public balance; they are not observed historical mismatches.

ORD reads a dependency address from slot 4. Its getter returns a four-word tuple.
Initial one-word mocks reverted because the response was incomplete. The
[ABI-shaped follow-up](evidence/ord-tuple-controls.json) uses two holders and
12 controls: raw 123 with a unit value in any of the first three tuple positions
returns 124; a unit value only in the fourth returns 123; four values of 17
return 174. Those results establish dependency in the probed paths, not complete
qualification of the external formula, state changes or affected-holder output.

LBP's [reward model](lbp-reward-model.md) and YBC's
[external reward mismatch](ranks201-250-coverage.md) also remain outside the
production fixture. Completing this bounded candidate set will not establish
support for every BSC token or enumerate all holders. Ethereum, Base, HyperEVM
and Arc follow the same [network qualification sequence](network-expansion.md).

## Preserved incomplete diagnostics

The [initial disassembly failure](evidence/remaining250-review-initial.txt)
attempted to decode a nested external call against TITAN's runtime. The corrected
diagnostic identifies call depth and does not assign external instructions to
the token's code. The [initial qualification](evidence/remaining250-qualification-initial.json)
stopped with `missing reviewed writes`: its trace-key comparison failed to pad
RPC stack quantities such as `0x7`. The corrected run normalizes those quantities
before matching them. Neither failure supplies successful qualification evidence;
both original records remain preserved.
