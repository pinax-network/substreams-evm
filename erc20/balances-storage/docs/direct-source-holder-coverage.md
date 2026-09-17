# Thirteen more direct balance layouts

The [39-token fixture](../tests/fixtures/bsc-direct-source-layouts.json) adds 13
ranked BSC contracts whose verified Solidity source matches their historical
runtime. Each `balanceOf` returns a direct mapping word. Transfer restrictions,
fees, permit nonces, mint limits and administrative state do not introduce a
different balance projection for these getters.

| Rank in RPC stream | Contract | Reviewed source contract | Balance base |
| ---: | --- | --- | ---: |
| 18 | `0x9dc44ae5be187eca9e2a67e33f27a4c91cea1223` | PowerWrappedToken | 0 |
| 21 | `0x7d03759e5b41e36899833cb2e008455d69a24444` | FourERC20 Token | 0 |
| 25 | `0x2c3a8ee94ddd97244a93bc48298f97d2c412f7db` | AKEToken | 0 |
| 31 | `0x1ba42e5193dfa8b03d15dd1b86a3113bbbef8eeb` | BEP20Zcash | 1 |
| 34 | `0x6067490d05f3cf2fdffc0e353b1f5fd6e5ccdf70` | MMPRO | 2 |
| 35 | `0x000ae314e2a2172a039b26378814c252734f556a` | AsterToken | 0 |
| 36 | `0x8d65744527f55d0b2338350912d5c99a81ddf0e2` | ProToken Token | 0 |
| 38 | `0x9558a9254890b2a8b057a789f413631b9084f4a3` | AIN | 0 |
| 42 | `0x92aa03137385f18539301349dcfc9ebc923ffb10` | SKYAIToken | 0 |
| 43 | `0xeccbb861c0dda7efd964010085488b69317e4444` | FourERC20 Token | 0 |
| 45 | `0x0e63b9c287e32a05e6b9ab8ee8df88a2760225a9` | PieverseOFT | 5 |
| 47 | `0x595e21b20e78674f8a64c1566a20b2b316bc3511` | Bulla | 0 |
| 50 | `0xcae117ca6bc8a341d2e7207f30e180f0e5618b9d` | ARK | 0 |

Contract addresses identify the profiles; names are descriptive, not identity
checks. Rankings count balance rows in the captured RPC stream, not market cap.
All profiles are explicit test inputs. Production parameters remain `[]`.

## Source and getter qualification

The [qualification record](evidence/direct-source-qualification.json) retains
source URLs and digests, per-source content hashes, compilation metadata,
compiler storage layouts, runtime hashes, canonical block identities, getter
traces and state-override results. The inspection checks that each Sourcify
record has the expected address/chain and verified runtime, and that the supplied
on-chain bytecode equals the historical `eth_getCode` result. Every token passes
independent mapping-word overrides of **0, 1, 123 and uint256 max**, as well as
the observed holder balance. Runtime boundaries are checked across the full
1,024-block reference range. These controls supplement the source review;
they do not automatically promote layouts.

Twelve profiles have compiler storage layouts. MMPRO's older compiler record
does not, so its reviewed inheritance order is corroborated by the getter trace
and mapping controls: total supply is slot 0, owner 1, balances 2, transfer lock
3 and allowances 4. The constant token metadata does not occupy storage.

Reviewed non-balance mappings include ordinary/nested allowances, ASTER/ARK
permit nonces, ProToken whitelist flags, AIN mint counters, POWER role indices,
and PIEVERSE bridge/whitelist/minter configuration. Scalars include supply,
ownership, metadata headers, transfer modes and administration. ProToken's sell
fee calls the inherited balance update twice; the mapper reads the resulting
holder writes rather than reconstructing balances from net Transfer amounts.
ARK's governance and pool operations likewise update its ordinary mapping.

AIN caps its minter AddressSet at six. The fixture explicitly permits the six
array words beginning at `keccak256(word(6))`, along with the array length and
reverse-index mapping. A regression verifies that the adjacent seventh word
still fails. POWER's unbounded role-array elements and long dynamic metadata or
PIEVERSE option payload words are not generally configured by these profiles;
unreviewed writes to them stop processing. No observed unknown slot was added
to an ignore list just to make a test pass. These administrative paths require
further qualification when encountered.

The core balance algorithm is unchanged in this batch. No map module, cache,
custom protobuf, generated binding, RPC import or hardcoded token was added.

## Actual WASM and retained holders

The [WASM audit](evidence/direct-source-rpc.json) covers **1,024 consecutive
blocks, 122288006–122289029**. All **82,289 emitted balances** match independent
hash-pinned RPC, including **11,484 zeros**. All **39 configured tokens** emit
rows. [Per-token counts](evidence/direct-source-rpc-token-counts.json) preserve
each token's coverage rather than inferring activity from aggregate success.
The 13 new profiles contribute **4,839 emitted balance checks**.

The [holder replay](evidence/direct-source-holder-coverage.json) covers **452
consecutive blocks, 122288006–122288457**. All **64,808 reference observations
across all 39 tokens** match initialized state. The 13 additions contribute
**3,923 observations**, including 27 carried-forward values without an emitted
write in that block. No initialized value is unknown or incorrect.

Setup uses **25,088 stored-holder checkpoints** (50,176 balance/storage RPC
reads), 3,001 independently computed initial balances and 19/184 holders
initialized at the two previously qualified deployments. Ongoing processing
makes no balance RPC calls and reference values never repair state. Runtime
qualification and canonical-header verification still use RPC.

Cold replay retains **23,631 unknown observations**, including **12,641 nonzero
balances**, and zero incorrect known values. For the 13 additions, 1,661
observations remain unknown, including 729 nonzero values. Matching initialized
state is therefore distinct from complete cold-start holder coverage. These
ranges overlap earlier tests; their totals are not additive independent history.

## Rust regressions and remaining candidates

The committed [captured cases](../tests/fixtures/direct-source/cases.json) retain
actual relevant transactions and original headers for all 13 additions. The
generator checked that filtering unrelated transactions preserved the token's
full-block output, then independently queried every emitted end-of-block
balance by canonical hash. The Rust regression compares **44 recorded RPC
balances**, rather than deriving expected values from the mapper. Full blocks
are used for the live WASM audit and holder replay. Fixture, layout and package
digests are recorded in [artifacts](evidence/direct-source-artifacts.json).

**127 workspace Rust library/binary tests pass**, including the captured
13-token comparison and AIN's bounded minter-array check. Clippy with warnings
denied, workspace WASM compilation and targeted formatting/diff checks also
pass. The runtime WASM remains identical to the preceding computed-token batch;
the package metadata now describes the 39-token fixture.

The remaining top-50 group contains **11 unqualified tokens**: two source-verified
proxy shells whose implementations still need qualification, and nine for which
this Sourcify lookup did not return verified source. Getter controls alone do
not establish their full semantics. Rank 16 remains in that latter group.
Global holder bootstrap, identical raw event-row coverage and arbitrary future
administrative mutations are also not claimed.

Reproduce the WASM audit with the 39-token fixture and
`audit-rpc --start 122288006 --blocks 1024`. Reproduce holder state with
`holder-coverage`, the original `out/top50-1024/report.json` and digest-matching
reference capture, plus full Extended blocks 122288006–122288457. Use new output
directories and the appropriate Substreams endpoint. All executable tooling and
tests are Rust.
