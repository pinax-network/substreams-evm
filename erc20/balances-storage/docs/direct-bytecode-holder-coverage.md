# Direct-bytecode holder coverage

The [97-profile fixture](../tests/fixtures/bsc-direct-bytecode-layouts.json)
extends the preceding 89 with eight contracts from the original BSC top-100
capture. All eight have direct balance getters. Verified source was unavailable;
qualification uses the historical runtime, reviewed bytecode and independent
read-only RPC controls. It does not claim source verification.

| Rank | Token | Contract | Balance mapping base |
| --- | --- | --- | --- |
| 55 | SAC | `0x7cc151c2ee1016d7b586fb818759b6ae07719999` | 1 |
| 62 | 79AU | `0xc35ef0564751a27b468f791113c686c8eebaa9ca` | 0 |
| 67 | 果蝇 | `0xf7c2677146a271cafb928755a221b529b0197777` | 10 |
| 69 | BREW | `0xfa6d9b504848606eb9aec04ccc161d169b3f2159` | 0 |
| 72 | AIT | `0x2ab198e0078a8a233b8e7685356cb76482b9e681` | 1 |
| 79 | QVX | `0xa395999b8f96efbc0ad1cc97edc85304ca3632ef` | 0 |
| 89 | FLY | `0x66ea222a954f2f1090f7a33b1b2572f019147777` | 0 |
| 92 | WCP | `0x0ab69a79507b05bad4e4b13ecde5a5f83f53d274` | 0 |

[Qualification evidence](evidence/direct-bytecode-qualification.json) retains
the full runtime and executed getter instructions, standard getter storage
reads, caller/address controls, layout fields and historical boundary hashes.
The reviewed balance paths decode an address, hash it with the stated mapping
base, load one word and return it unchanged. Their successful paths have no
additional state-dependent balance branch or external call.

Each contract passed raw balance overrides of zero, one, 123 and uint256 maximum,
plus zero/123 controls using null and maximum addresses and different callers.
Allowance traces identify nested mapping bases separately from balances.
Scalar fields are limited to identified getters and the explicitly reviewed
bookkeeping below. Unknown writes remain errors; this is not an automatically
generated ignore list.

## Additional writes found by the full replay

Six candidates passed the consecutive-block replay with only their identified
standard fields. SAC and 79AU exposed additional writes. These were extraction
errors at unclassified storage keys, not observed balance-value mismatches.

- **SAC:** a burn at block **122288579** increases scalar slot 0 by the burned
  amount while decreasing the holder's mapping-1 balance and slot-3 total supply.
  The historical transaction trace confirms the supply write at PC 6208 and the
  separate counter write at PC 6226. Slot 0 does not enter `balanceOf`.
- **79AU:** a transfer at **122288270** updates mapping 15 with the current block
  number (`NUMBER` followed by the mapping write at PC 8355). Transfer restrictions
  consult this mapping; `balanceOf` reads mapping 0 independently. Six other
  blocks set and clear a temporary byte above the low-160-bit address in scalar
  slot 19. The captured **122288404** transaction writes that flag at PCs 3869
  and 3893; the getter does not read it.

[Bookkeeping evidence](evidence/direct-bytecode-bookkeeping.json) retains the
original unresolved-storage errors, reviewed instruction ranges and historical
transaction write locations. Sixteen additional controls set those bookkeeping
words to zero or uint256 maximum while independently varying the holder word
through zero, one, 123 and maximum. Every result remains the raw holder word.

The existing explicit layout fields handle these writes. No production mapper
logic, new module, RPC dependency or protobuf was needed. A second native scan
with all eight final layouts passes all **1,024 consecutive blocks,
122288006–122289029**, producing **1,384 balances** for the additions.

## Validation

The [eleven captured Rust cases](../tests/fixtures/direct-bytecode/cases.json)
cover all eight tokens and the three specific bookkeeping transactions, with
**28 independent historical RPC expectations**. Each fixture retains the original
header and token-relevant transactions; extraction was checked against the full
captured block before saving it. Regressions also require the explicit counter,
tracking and flag fields: removing each one reproduces the storage error.

**157 workspace Rust library/binary tests pass**, along with Clippy with warnings
denied, workspace WASM compilation, targeted formatting and diff checks.

The actual packaged WASM matches all **95,622 emitted balances**, including
**13,360 zeros**, across all 97 tokens and all 1,024 blocks. The eight additions
contribute 1,384 checks. There are **zero RPC mismatches**. The WASM binary is
identical to the preceding voting-token build; its digest matches the retained
[import scan](evidence/voting-wasm-imports.json), which contains only the four
reviewed output/logging/panic/empty-output imports and no RPC imports. The package
itself has a new digest because it embeds the updated README, and the audit
uses that exact new package.

Initialized-holder replay matches all **156,488 reference observations**,
including **4,328 carried-forward matches**, with zero unknown or incorrect
initialized values. The eight additions contribute **2,357 observations** and
**13 carried-forward matches**. Setup verifies **51,053 stored-holder checkpoints**
with **102,106 balance/storage RPC reads**; formula and validated deployment
baselines remain separately counted. Processing makes **zero balance RPC calls**
and 1,024 header verification calls. Reference observations never repair state.

Cold replay retains **56,538 unknown observations**, including **29,897 nonzero
values**, with no incorrect known values. The additions account for 960 unknown
observations, including 346 nonzero values. These gaps remain explicit rather
than becoming invented zeros or a claim of complete holder coverage.

Final packaged-WASM and holder-replay results are recorded in
[the RPC audit](evidence/direct-bytecode-rpc.json),
[per-token audit counts](evidence/direct-bytecode-rpc-token-counts.json),
[the holder replay](evidence/direct-bytecode-holder-coverage.json), and
[artifact identities](evidence/direct-bytecode-artifacts.json).

## Remaining scope

Three of the original top 100 remain unqualified: **4Stock (rank 56)**, **CAP
(rank 85)** and the **log-only zero-balance candidate (rank 70)**. The two proxies
need complete dependency/non-balance-field review. For rank 70, getter controls
pass but the captured activity emits nonzero Transfer logs without balance
storage writes; those logs do not justify inventing holder balances.

These results cover the captured interval and observed holders. They do not
establish complete global holder enumeration or arbitrary-token support. Missing
prior state still requires a verified checkpoint or full-history replay.
Overlapping BSC reports must not be summed as independent history. After BSC,
the [requested network sequence](network-expansion.md) is Ethereum, Base,
HyperEVM and Arc, with separate chain and runtime qualification for each.
