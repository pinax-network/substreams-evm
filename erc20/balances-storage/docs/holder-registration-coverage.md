# BSC holder registration and launch replay

4Stock's initial mint and new-holder registrations now use an explicitly
qualified profile in [the updated 100-token fixture](../tests/fixtures/bsc-holder-registration-layouts.json).
The earlier top-100 window did not exercise its first deployment or membership
list. Those paths failed closed under the earlier profile. This extension keeps
one RPC-free `map_events`, shared `evm.balances.v1.Events` and production defaults
`[]`. It adds no token-specific production code.

## Qualification

The contract `0xd270d4e1ec6e6e0d28c0ecb8be966ec75997ffff` is a pinned
45-byte ERC-1167 clone. Its implementation
`0xd442ecf61c6742167600bcf063e1d761304f8014` has a direct mapping-1
balance getter: the complete selector/ABI path reaches its only storage read at
PC 9263 and returns that word at PC 9270. Verified source was unavailable; this
qualification uses the pinned bytecode, execution traces and independent RPC
controls.

The [initialization evidence](evidence/4stock-registration-qualification.json)
binds first CREATE to block **120607788**, hash
`0x321db532bb8b02f762789bd75969fe4600ae668dfebc669aea6e745a10eda61c`.
Canonical RPC confirms empty code and nonce zero at its parent, then the clone
and implementation runtimes at deployment and audit boundaries. The complete
captured block includes initialization and later distribution. Its two initial
holder balances match RPC and sum to the raw RPC supply of **10^27**.

Initialization writes include configuration scalars, mapping-17 flags and the
two explicit words of a 54-byte metadata string at `keccak256(slot 8)`. The
encoded string length is 109 (`54 * 2 + 1`). Each of 51 non-balance words was
independently overridden while checking both initial holders. Eight additional
raw balance-word controls cover zero, one, 123 and uint256 maximum. These 110
controls support the complete getter review; they do not substitute for it.

The first registration grows slot 35 from zero to one, writes the address at
`keccak256(slot 35)`, and sets mapping-36 field 4. The implementation explicitly
embeds the array base, so a 32-byte Keccak preimage is absent from the capture.
Mapping-36 fields 0–3 already covered reward accounting; the newly reviewed
membership flag extends its configured width to five words.

The longer launch capture exposed three additional reward fields, slots
**38, 41 and 44**. The mapper rejected them until their qualification. The
[follow-up evidence](evidence/4stock-launch-field-qualification.json) includes
the accumulator writes, complete balance-getter path and 18 independent
zero/123/uint256-max overrides. None changes the getter's result. These slots
are explicit entries, not an ignored scalar range.

## Address-list validation

The optional `address_lists` rule accepts only a reviewed append-only address
array. Each length write must increment by one from a value below `2^64`.
Its exact element at `keccak256(root) + old_length`, modulo `2^256`, must start
empty and be written once after the length increment and before the next one.
Multiple appends require continuous lengths. The final valid length can equal
`2^64`. The element must fit an address, and roots cannot overlap other configured
fields. The implementation's relevant instructions are retained in the
qualification evidence.

Zero-address appends require their persisted zero-to-zero element witness.
That separate validation path does not emit ordinary balance no-ops. Reverted
and failed execution cannot authorize a persisted array element. Unknown writes,
orphaned elements, missing witnesses, overwrites, deletions, malformed values and
changed runtimes still fail. Membership itself never supplies a balance.

## Launch results

Across **256 consecutive blocks, 120607788–120608043**, the updated profile
accepts **315 membership appends** and all reviewed storage changes. The actual
packaged WASM passes **1,400 emitted-balance RPC checks**, including **300 zeros**,
with no mismatches. The [RPC audit](evidence/4stock-launch-rpc.json) verifies delivery for all blocks, including
62 empty outputs, using consecutive canonical block clocks.

The [native retained-holder replay](evidence/4stock-launch-holder-coverage.json) passes all **1,787 reference observations**,
including **387 carried-forward values**, with zero unknown or incorrect values.
The 343 observed holders initialize at the validated CREATE, then receive only
storage-derived updates. There are **zero checkpoint balance/storage RPC reads**
and **zero processing balance RPC calls**; 256 header calls verify block identity.

The [independent final snapshot](evidence/4stock-final-holder-snapshot.json)
reconstructs retained state from the **actual WASM stream**, checks all 1,787
reference observations again, and compares **all 343 observed holders** with
canonical RPC at the final block. All match, including **126 zeros**. It also
checks all **315 address-list entries**, their membership flags and the final
list length against RPC. The list members are included in the tested holder set.

Raw output row counts remain different: the reference includes 387 observations
that need no changed-balance output. The retained values match; this is not a
claim of exact raw event-row parity. A reward membership list also does not
establish a global enumeration of all possible token holders.

## Regression validation

Ten new Rust tests cover witnessed multiple appends, missing and malformed
elements, execution order, reverted/failed writes, null-address no-ops, the
`u64::MAX` boundary, conflicting configuration roots, captured deployment and
initial supply, and the first trading block. Removing any newly qualified
reward field makes the captured trading block fail again. The complete
workspace has **181 passing library/binary tests**; Clippy with warnings denied,
workspace WASM compilation, targeted formatting and diff checks also pass.
Generated-protobuf doctests are outside this library/binary test result.

The original **1,024-block top-100 interval** is [audited again](evidence/registration-top100-rpc.json)
with the new package and extended profile. Its **96,269 emitted balances**,
including **13,714 zeros**, still match RPC with no mismatches. The
[complete output comparison](evidence/registration-top100-output-parity.json)
checks every row and value against the preceding top-100 storage package.
The [full holder replay](evidence/registration-top100-holder-coverage.json)
and [artifact digests](evidence/holder-registration-artifacts.json) are retained
with this follow-up; earlier reports keep their original package identities
and scope. The [WASM import scan](evidence/registration-wasm-imports.json)
contains no RPC functions.

The original top-100 review remains a bounded BSC campaign. Further candidates,
untested code paths and the requested [Ethereum, Base, HyperEVM and Arc
qualification](network-expansion.md) are separate work. RPC prerequisite checks
on those networks are not token or holder parity evidence.
