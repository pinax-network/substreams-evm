# VSD proxy and holder qualification

VSD, rank **179** (`0x404ad8a572bb30bf22c5228e1e210a4a0162399f`), now passes
the BSC campaign over **122288006–122289029**, inclusive. The
[combined fixture](../tests/fixtures/bsc-vsd-layouts.json) contains **198 profiles**.
Only LBP (143) and hLBP (189) remain outside the active fixture from the reviewed
top 200. Production code and package bytes are unchanged: one RPC-free
`map_events`, shared `evm.balances.v1.Events`, explicit layouts and default `[]`.

## Historical runtime and fields

[Qualification evidence](evidence/vsd-qualification.json) preserves both runtime
bytecodes, instruction review, execution traces, controls and original failures.
Sourcify has no verified source for either address. This qualification uses the
historical runtime and canonical calls; it does not claim source verification.

- Proxy runtime hash:
  `0x00063bb4348b6738f2fa41db52ea164b81be26f5f01832f631f9350cd8039abe`.
- Implementation: `0x5bdf6b0a9b666268a4a9f2cf60d3c47a074693ab`, runtime hash
  `0x777935e954b7b889cfe3192be0d6731186ede4f524678e6a10abc8c8de66aa52`.
- The proxy reads the ERC-1967 implementation slot at PC 136 and delegates at
  PC 158. The immutable proxy admin takes a separate upgrade-only path:
  `balanceOf` from that address reverts without delegation, confirmed by a
  separate trace. The reference and audit use ordinary balance calls.
- Implementation selector `70a08231` routes to PC 1392, validates the canonical
  address argument and returns the single SLOAD at PC 1452 unchanged. The root
  copied from runtime offset `0x1c1e` is the ERC20 namespace
  `0x52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00`.
  The successful balance path has no external dependency or state-selected
  alternative amount.

Four balance-word controls (zero, one, 123 and uint256 maximum), four holder/caller
controls and eight controls varying the explicit non-balance words all pass.
The proxy, implementation pointer and implementation runtime are checked at both
historical boundaries. Existing protected-dependency checks remain in force.

The initial strict scan rejected **13 blocks** on an unconfigured nested mapping.
Selector `dd62ed3e` proves it is the allowance field: the owner hash uses namespace
offset one, followed by the spender hash; PC 472 returns the resulting word.
The actual allowance key at block **122288093** is exactly the key rejected by
the mapper. The earlier apparent roots `d6a01d…` and `e91ebd…` were intermediate
hashes, not independent storage roots. Configuring the reviewed allowance root
resolves all 13 blocks; no individual hashed key is placed on an ignore list.

The other explicit fields come from their own traced getters: total supply at
namespace offset two, token name at scalar slot two and symbol at scalar slot
three. Metadata is not assumed to use the ERC20 namespace. Other custom fields,
including unreviewed administrative or mint/burn bookkeeping, remain rejected.
Passing this interval does not qualify every custom execution path.

## RPC and holder results

The native scan produces **72 balances in 24 blocks** without errors. The actual
packaged-WASM capture covers all 1,024 blocks; canonical clock evidence confirms
the other **1,000 blocks have empty output**.

The [first RPC audit](evidence/vsd-rpc-incomplete.json) stopped on a transport
failure after 357 blocks and 24 matching balance checks. It is preserved as
incomplete. The [completed audit](evidence/vsd-rpc.json) checks every row of that
same immutable capture against fresh canonical headers and historical RPC:
**72 checks, 25 zeros, zero mismatches**. It records the original failure's
digest; there is no recapture or reference-state repair.

The [native holder replay](evidence/vsd-holder-coverage.json) initializes
**22 observed holders** with **44 explicit checkpoint balance/storage reads**.
All **96 reference observations** match. Cold state has **24 unknown zero-value
observations**, which are reported as unknown rather than silently set to zero.

The [independent actual-WASM holder replay](evidence/vsd-wasm-holders.json)
matches all 96 observations, including **24 initialized carry-forward matches**.
Its final canonical RPC snapshot matches all **22 holders**, including **three
zeros**. Reference observations never repair retained state, and replay performs
no balance RPC calls. This is observed-holder coverage, not a global holder list.

Two captured Rust cases at **122288057** and **122288093** reproduce six
independent historical RPC balances. Removing the qualified allowance mapping
reproduces the original unresolved-storage error in the latter case.
All **215 workspace Rust library/binary tests pass**, together with Clippy with
warnings denied, workspace WASM compilation, targeted formatting and diff checks.
Repacking preserves the exact audited SPKG and WASM:

- SPKG: `d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`.
- WASM: `861a879353a7fc7163a26c80f671fc3011e9ca3ee32a65dfc0badd99ac9794b6`.

## Combined scope

The [new 198-profile WASM capture](evidence/vsd-combined.json) matches every
protobuf event field in the union of the prior 197-profile capture and VSD's
audited output: **103,647 previously verified balances**. It verifies artifact
digests and fresh canonical headers, using the same package for both captures.
This comparison reuses historical balance evidence; it performs no new balance
RPC calls and is not a fresh combined-holder checkpoint or snapshot.

The underlying disjoint audits contain **14,573 emitted zero checks** and
**169,766 initialized holder observations**, including **66,120 initialized
carry-forward matches**, with identical event histories preserved by the new
capture. The prior [197-profile report](pending-direct-coverage.md) retains its
source cohorts and both historical package identities.

[LBP's reward-driven drift](lbp-reward-mismatch.md) remains unresolved, and
[hLBP's quiet-holder evidence](hlbp-quiet-holder-coverage.md) remains separate
until a changed-balance case is qualified. The shareholder-removal window is
also separate; no new captured swap-and-pop is claimed here. After BSC, the
requested sequence is [Ethereum, Base, HyperEVM and Arc](network-expansion.md),
with independent runtime, Extended-block, RPC and holder evidence per network.
