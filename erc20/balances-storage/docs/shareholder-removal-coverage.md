# Shareholder-list removal regression

The mapper previously rejected a real SpaceX Meme shareholder removal at BSC
block **122264505** with `address-list length must append one`. It now validates
the removal and emits all six independently verified balances. Production
still has one RPC-free `map_events`, the shared `evm.balances.v1.Events` schema,
explicit qualified layouts and default parameters `[]`.

This fixes bookkeeping validation for an already qualified token; the campaign
still has **188 qualified profiles and 12 unqualified candidates**. It does not
resolve [LBP's reward-dependent balances](lbp-reward-mismatch.md).

## Captured failure and correction

The token is `0x28e69efcf168813113e039341cba335d1303167e`. Its reviewed getter
returns mapping 1 unchanged; its shareholder address array is rooted at 28.
Runtime-bound source contains both tail-pop and swap-and-pop removal paths.

[Historical length sampling](evidence/shareholder-removal-search.json) located
the decrease from 157 to 156. The adjacent canonical blocks have the qualified
runtime `0xfdaa778653b5a8155aeb0998faf66d36be6e6cde562a02b4d19de7dbd8b45369`.
The captured transaction clears element 156 at ordinal 3891, then decrements
the length at ordinal 3892. It is a **tail pop without a swap**.

The [original failure and independent before/after RPC balances](../tests/fixtures/shareholder-removal/before.json)
are preserved alongside the captured Extended-block fixture. Their status is
the pre-fix result, not the current implementation's status. The regression
binds the fixture to the canonical block number and hash and checks all six
resulting balances.

The validator now requires a one-entry length decrease and the exact cleared
tail before that decrease. An optional swap must copy that same old tail
address into one earlier element before clearing the tail. Every accepted
write needs a distinct ordinal witness. Repeated writes must preserve old/new
continuity. Unknown writes, malformed addresses, missing clears, reused
witnesses and failed/reverted witnesses remain rejected. Explicit array roots
are still required; this does not enable arbitrary dynamic storage ranges.

Synthetic Rust cases cover swap-and-pop, append/pop/reappend, null-address
removal and fourteen malformed removal variants. Swap behavior has synthetic
coverage; a real swap transaction has not yet been captured.

## Packaged WASM and holders

The [fresh RPC audit](evidence/shareholder-removal-rpc.json) executes the rebuilt
package over **122264480–122264543**, 64 consecutive finalized blocks. All
**six emitted balances match**, with zero mismatches. Canonical clock evidence
covers the other 63 blocks with empty output.

The [native holder replay](evidence/shareholder-removal-holder-coverage.json)
initializes six observed holders with 12 explicit balance/storage RPC reads.
The [independent replay from actual WASM](evidence/shareholder-removal-wasm-holders.json)
matches all six reference observations and the final RPC snapshot of all six
retained holders. This window has no unchanged reference observations for
these six holders, so it does not add carry-forward evidence for them.

The [removed holder's separate audit](evidence/shareholder-removed-holder.json)
initializes `0x543ab323ea485fcdb08847ec68eed5ae8352b00a` independently. This
address is absent from those six reference holders and emits no balance event.
Its retained balance of **3251823000000000000000 base units** matches canonical
RPC balance and storage in **all 64 blocks**, including before and after its
list removal. Two checkpoint reads and 128 validation reads are recorded.
Reference reads never repair state. Removing a bookkeeping-list entry does
not delete the holder's ERC-20 balance.

## Compatibility with the 188-profile campaign

A [new combined 188-profile WASM run](evidence/shareholder-removal-188-regression.json)
covers the original **122288006–122289029** interval. Every protobuf output
field matches the union of the five prior disjoint audits: **103,022 balances**.
The check verifies the old evidence digests and binds each old RPC row to fresh
canonical headers. It reuses those historical balance RPC results; it makes
**zero fresh balance RPC calls** for this compatibility comparison.

The same event histories preserve the prior 168,800 initialized holder
observations, including 65,779 matches without a new event. This is output
compatibility evidence, not a fresh 188-profile checkpoint or final-holder
snapshot audit. The [previous cohort report](more-qualified-coverage.md) retains
its original package identities and scope.

Current package SHA-256:
`d5dbc5922fd01a7337d03f2ca827fd06b810e3e3a311bc2545494a5fb12ace40`.
Current WASM SHA-256:
`861a879353a7fc7163a26c80f671fc3011e9ca3ee32a65dfc0badd99ac9794b6`.

All **210 workspace Rust library/binary tests pass**. Clippy with warnings
denied, workspace WASM compilation, targeted formatting and diff checks pass.
The final repack preserves the exact package and WASM digests audited above.

This remains bounded token and observed-holder qualification. The requested
network sequence is [Ethereum, Base, HyperEVM and Arc](network-expansion.md)
after BSC, with separate runtime, Extended-block, RPC and holder evidence.
