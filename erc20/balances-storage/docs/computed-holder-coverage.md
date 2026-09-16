# Address-derived balances and holders without writes

The explicit [26-token fixture](../tests/fixtures/bsc-computed-family-layouts.json)
adds rank 7 from the captured BSC RPC stream:
`0xec22e64c0a16821dc1b457045936c0219b47155e`. Its historical runtime Keccak-256 is
`0x503d264fcd04b84cc655974ad83814d8850ead0e48e4ec9fdb5c28d896392400`.
This profile has bytecode, execution-trace and read-only state-override evidence;
verified Solidity source was unavailable. It is a caller-supplied test layout,
not a production default or an automatically discovered rule.

## Why storage writes were insufficient

At block **122288080**, this token emits **3,000 Transfer logs and no storage
changes**. The RPC reference observes 3,002 distinct holders. The getter has two
paths:

- Ordinary addresses return
  `(uint256(keccak256(packed_20_byte_holder)) % 9001 + 8434) * 10^18`.
- Addresses selected by the low 160 bits of scalar slots **3 and 4** return
  `mapping(address => uint256)` at base **5**. The selected addresses are
  `0xc54cb14840cabf9a29b43d528af7dea7771f7494` and the null address.

Applying the ordinary formula to everyone caused the one characterized
mismatch: the nonzero special holder returns `5000000000000000000000000000`
from storage, while the formula would return `11848000000000000000000`.
The null address also uses storage and remains excluded from public output,
matching the reference.

The getter entry at bytecode PC 3000 checks both masked selectors. PCs
3174–3235 read the selected holder's mapping word; PCs 3242–3336 calculate the
ordinary path. Reviewed helper paths pack the 20-byte address and perform
checked modulus, addition and multiplication. This getter does not read a
caller, block-dependent value or external contract. Slots 0, 1 and 2 hold
name, symbol and total supply, while the allowance getter uses nested mapping 6.
Their reviewed writes do not change the balance projection.

Read-only `eth_call` state overrides set mapping words to 0, 1 and 123. The
ordinary holder's balance remains `9699000000000000000000`; both special
holders return the overridden word. Selecting an ordinary holder through
either scalar slot switches it to storage. Setting the upper 96 bits of each
selector word to all ones leaves the ordinary balance unchanged. These are
simulations, not on-chain transactions. Earlier characterization and failed
formula-only evidence remain unchanged.

Evidence: [independent getter checks and special-address controls](evidence/computed-address-characterization.json),
[mapping controls and observed events](evidence/computed-mapping-controls.json),
[other getters and packed-selector controls](evidence/computed-getter-qualification.json),
[getter bytecode](evidence/computed-getter-bytecode.txt),
[complete pinned runtime](evidence/computed-runtime.hex).

## Guarded output and retained state

The optional `address_hash_balance` configuration describes the formula and
selector dependencies. Constants must fit uint256, the modulus must be nonzero,
and neither addition nor multiplication may overflow. Selector slots cannot be
ignored or reused as balance/proxy dependencies. Zero-word fallbacks and
deployment baselines cannot be combined with this rule.

The audit tools bind the runtime and selectors before and after the range.
The mapper rejects any persisted selector-address change, including a change
followed by restoration. Unrelated packed upper bits are allowed. Runtime and
proxy dependency guards remain active. A selector change requires requalification
and rebuilding affected holder state; the map cannot silently update every
previous holder.

The map uses the reference ABI decoders and successful, non-reverted logs to
select Transfer/Approval participants, non-indexed OwnershipTransferred
participants, transaction senders and token addresses. It emits computed
balances even without writes, deduplicated with storage-derived rows.
Unreviewed or malformed events on a configured computed token fail. Events
on other contracts cannot create rows for this token, and event amounts never
become balance values.

The captured regression emits **3,001 computed balances**, omitting the special
stored holder whose value cannot be known from this block. The retained-state
test computes ordinary holders' initial values independently of RPC reference
amounts. Special stored holders still need a verified checkpoint or an observed
write. The checkpoint report separates computed initialization from actual
balance/storage RPC reads. A cold stream must preserve unknown stored holders
as unknown rather than invent zero balances.

There remains exactly one RPC-free `map_events`, shared `evm.balances.v1.Events`
output and empty default parameters. All executable tooling and tests are Rust.
This profile covers observed holders under this qualified getter, not all ERC-20
contracts, all addresses, rebases, mutable global defaults or a production
bootstrap. Historical activity for this candidate occurs in just one block of
the 1,024-block reference window; the replay must not be described as independent
activity across hundreds of blocks.

## Measured qualification

The actual WASM streamed **512 blocks, 122288006–122288517**. All **42,059
emitted balances** matched independent hash-pinned RPC, including 5,399 zeros.
All **26 configured tokens** emitted rows; this candidate contributed 3,001
checks. [Audit](evidence/computed-family-rpc.json),
[per-token counts](evidence/computed-family-rpc-token-counts.json).

The [holder replay](evidence/computed-family-holder-coverage.json) spans **452
consecutive blocks, 122288006–122288457**. All **60,885 reference observations
across 26 tokens** match initialized state, with zero unknowns or value
mismatches. Initialization consists of:

- **24,127 existing stored holders**, checked with 48,254 setup RPC reads.
- **3,001 computed holders**, initialized by the qualified formula, with no
  balance/storage RPC reads for those holders.
- **19 and 184 holders** initialized at the two previously qualified deployments.

For the new token, all **3,002 reference observations match**: 3,001 values come
directly from the mapper, and the special holder comes from a one-holder
checkpoint (two setup RPC reads). Without that checkpoint, the special holder
remains unknown. Across the whole fixture, a cold replay leaves 21,970 reference
observations unknown, including 11,912 nonzero balances, and has zero incorrect
known values. These counts preserve the difference between value parity and
complete cold-start holder coverage. Reference amounts never repair ongoing
state, and processing performs no balance RPC calls. Runtime and canonical
header verification still use RPC.

These ranges overlap earlier qualification; their checks are not additional
independent history. They validate the new binary against the same recorded
reference and both deployments. They do not establish identical raw event-row
coverage or global holder enumeration.

The package SHA-256 is
`ae76638f8bc7227a77e9db4541eff8c6bae5a324bc3ab58c4217a9f98e67a65c`;
the WASM SHA-256 is
`420eb6d539c4540ba3a0c5e833859d0f8ca732d59e1d87c9e8fc93722d087454`.
Its only imports are logging, output, panic registration and empty-output
handling: **no RPC imports**. [Artifact and fixture digests](evidence/computed-artifacts.json).

Reproduce with `audit-rpc --start 122288006 --blocks 512` and the 26-token
fixture. For `holder-coverage`, use the original `out/top50-1024/report.json`
ranking and matching reference digest, plus full consecutive Extended blocks
122288006–122288457. Both commands require new output directories; `audit-rpc`
also accepts the appropriate Substreams endpoint. The committed protobuf
regression keeps the actual relevant transactions and header; live validation
uses full blocks.

**125 workspace Rust library/binary tests pass**, along with Clippy with warnings
denied, workspace WASM compilation and targeted formatting/diff checks. The eight
new regressions cover the captured block, independently recorded RPC values,
stored-holder exceptions, event selection, malformed events, selector changes
and restoration, packed flags, raw-word continuity, arithmetic validation and
runtime-boundary qualification. The known generated-proto doctest issue is
separate from this CI-equivalent library/binary check.
