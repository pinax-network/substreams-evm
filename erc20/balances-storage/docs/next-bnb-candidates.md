# Additional BNB candidates and RPC compatibility fixes

The reviewed configuration now covers eight tokens: USDT, WBNB, USDC, BTCB,
ETH, BUSD, CAKE and USD1. Use the explicit
[expanded fixture](../tests/fixtures/bsc-expanded-layouts.json); it does not change
the empty default configuration. The package still exposes only `map_events`
with the shared `evm.balances.v1.Events` protobuf and no ingestion RPC calls.

## Fixed RPC return decoding

All 136 previously unresolved vUSDT checks were caused by an overly strict test
decoder. The verified delegator's `delegateToViewAndReturn` skips the wrapper's
64-byte prefix but returns the original return length. Its `balanceOf` therefore
returns 96 bytes, with the balance in the first word. The RPC reference's
`BalanceOf::output` uses `ethabi::decode(Uint(256))`, which accepts trailing bytes.
Our validator incorrectly required exactly 32 bytes.

The validator now reads the complete leading word and accepts trailing bytes,
while rejecting truncated words, malformed hex and empty output. Native balance
quantity decoding remains unchanged. A Rust regression compares return lengths
0–100 with the reference's actual decoder, and a
[captured vUSDT response](../tests/fixtures/vusdt-return-data.json) checks the
historical value `21323187432458722`.

The new `recheck-rpc` command preserves the original run and diagnoses its null
results at their original canonical block hashes. The
[280-check recheck](evidence/top50-rpc-recheck.json) found:

| Original unresolved checks | Diagnosis |
| ---: | --- |
| 136 | vUSDT now decodes and exactly matches the recorded storage word |
| 143 | `0xcccbd077b612a42f896b53e24a11c36d32117777`: no code at the parent boundary; deployed during the current block |
| 1 | `0x29a864a830528667e571c7a8f5f722cf704753c2`: no code at the parent boundary; deployed during the current block |

The 144 predeployment calls return `0x`. They are unavailable `balanceOf`
comparisons, not zeros or passing value checks. Original results remain intact.
Future surveys also retain the actual failed RPC response, rather than only a
null decoded value. The vUSDT storage layout is **not yet qualified**: it still
has unresolved writes and needs implementation/source review beyond its wrapper.

## Four additional reviewed layouts

[Source/runtime evidence](evidence/expanded-source-qualification.json) binds each
Sourcify record to the historical runtime at block 122288005, including USD1's
proxy implementation. Reviewed `balanceOf` paths return the balance mapping
directly. All four passed read-only zero, 1, 123 and uint256-max controls.

| Token | Contract | Balance base | Other reviewed storage |
| --- | --- | ---: | --- |
| ETH | `0x2170ed0880ac9a755fd29b2688956bd959f933f8` | 1 | Owner, total supply, allowances |
| BUSD | `0xe9e7cea3dedca5984780bafc599bd69add087d56` | 1 | Owner, total supply, allowances |
| CAKE | `0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82` | 1 | Owner, total supply, allowances, delegates, governance checkpoints/counts, nonces |
| USD1 | `0x8d0d000ee44948fc98c9b98a4fa4921476f08b0d` | 51 | Allowances, supply, permit nonces, owner/pending owner, pause/freeze state, EIP-3009 authorization state |

USD1's fixture pins implementation
`0x694aa534bdef8ed63244eb902e7914e527891f08` and both runtime hashes, using the
existing proxy upgrade guards. Its ERC-7201 authorization mapping root is derived
from the reviewed `openzeppelin.storage.StablecoinV2` namespace. Transfers,
mint/burn and administrative frozen-fund operations still update the ordinary
balance mapping; the other state does not change `balanceOf`.

CAKE exposed a mapper limitation: each governance checkpoint contains a block
number and a vote value in **two storage words**. Only its first field has a
direct mapping-key preimage. The new optional `other_mapping_words` configuration
maps an explicitly reviewed non-balance mapping base to its word count. The
mapper verifies a preimage for `key - offset` and follows its nested mapping chain
back to that base. Counts are limited to 1–32 words; balance and implementation
slots cannot be ignored. This handles the second CAKE checkpoint word without
allowing arbitrary unknown writes. Regressions cover nested fields, adjacent
out-of-range writes, arithmetic carry/wrap and protected slots.

## Live validation

Actual WASM execution of all eight configurations matched **9,127 emitted
balances** against historical RPC, including **1,407 zeros**:

| Inclusive BSC range | Emitted balances | Mismatches |
| --- | ---: | ---: |
| 122288006–122288069 | 4,292 | 0 |
| 122284478–122284541 | 4,835 | 0 |

Reports: [current](evidence/expanded-rpc-current.json),
[independent](evidence/expanded-rpc-independent.json),
[per-token counts](evidence/expanded-rpc-token-counts.json). Every configured token
is represented; the four new tokens account for 355 emitted checks.

Package SHA-256:
`e5bd26d305143cd4048f77fc44129c57d6b3535ca34def03247dfd483360ce5e`.
WASM SHA-256:
`5990b57a66be7e280f78c1388ac92ee09ec36f3921efb7938d6a33c0f6545159`.

Native holder-state replay with explicit test checkpoints matched all **9,369
reference observations** across the current 64-block and independent 32-block
windows. Both runs had zero seeded unknowns or mismatches. Without a checkpoint,
2,526 observations remained unknown, including 784 nonzero balances. The
checkpoint covers only holders observed in these test windows; global bootstrap,
complete holder enumeration and production sink integration remain unfinished.
Reports: [current](evidence/expanded-holder-current.json) and
[independent](evidence/expanded-holder-independent.json).
An earlier independent-holder invocation used a nonexistent ranking path and
failed before reading data; that incomplete report is retained locally at
`out/expanded-holder-independent/report.json`. The corrected run has its own
output directory.

The [five-token targeted survey](evidence/next-five-survey-summary.json) replays
all 339 captured Extended blocks from the top-50 run, including reference-silent
blocks. ETH, BUSD, CAKE and USD1 have zero strict mapper errors. vUSDT has 64
strict mapper errors and remains unqualified. All **1,636 before/after candidate
checks** match RPC with no decoding errors. None has identical raw event-row
coverage, so the survey reports `coverage_gap` and exits nonzero.

## Remaining fallback-balance mismatches

[Four additional historical inspections](evidence/remaining-default-balance-inspections.json)
confirm nonzero defaults on the tested zero-word paths. Together with the
[original diagnosis](holder-coverage.md#why-the-original-token-mismatched):

| Contract | Balance base | Observed zero-word fallback, raw units |
| --- | ---: | --- |
| `0x45056c2627c9e60753aeef604ee9575709f8e88f` | 8 | Global slot 4: `7000000000000000000` |
| `0x2b74ac567edaa7a65237b4f92042667e784439d3` | 8 | Global slot 4: `8000000000000000000` |
| `0x5c8fb0f805f62cd082c00d9d250a388b543cc453` | 8 | Global slot 4: `8000000000000000000` |
| `0x70a163085db5617b700fafeac1c7a2794e41a724` | 2 | Implementation constant: `320173463400000000000` |
| `0x1922cf6ef99e41591a33687816baee1d5efc6db7` | 0 | Global slot 7: `9000000000` |

Each new inspection returns the supplied mapping word for 1, 123 and uint256 max,
but returns the fallback for zero. The constant-returning proxy reads its
implementation slot then executes `PUSH9` at PC 3342 with the fallback value;
its implementation is `0x75cc906469c92fb3f742a79369a30102d129fae9`.
These are traced code paths, not a complete source qualification. None is silently
promoted to a direct mapping. Remaining work includes a qualified fallback rule
and its shared-state dependencies, handling default changes for all affected
holders, custom/beacon proxy support, and deployment-boundary qualification.

## Reproduction

All executable validation remains Rust. Use the previous
[capture and ranking instructions](holder-coverage.md#reproduce-with-rust-tooling)
with `tests/fixtures/bsc-expanded-layouts.json` for audits and holder replay.
The commands below run from the repository root and preserve earlier output:

```sh
cargo run --locked -p erc20-balances-storage-tools -- recheck-rpc \
  --checks erc20/balances-storage/out/top50-parity/rpc-checks.jsonl \
  --output erc20/balances-storage/out/my-rpc-recheck

cargo run --locked -p erc20-balances-storage-tools -- test-ranked \
  --ranking erc20/balances-storage/out/top50-1024/report.json \
  --block-dir erc20/balances-storage/out/top50-active-blocks \
  --block-dir erc20/balances-storage/out/holder-coverage-blocks \
  --layouts erc20/balances-storage/tests/fixtures/bsc-expanded-layouts.json \
  --contract 0x2170ed0880ac9a755fd29b2688956bd959f933f8 \
  --contract 0xe9e7cea3dedca5984780bafc599bd69add087d56 \
  --contract 0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82 \
  --contract 0x8d0d000ee44948fc98c9b98a4fa4921476f08b0d \
  --contract 0xfd5840cd36d94d7229439859c0112a4185bc0255 \
  --output erc20/balances-storage/out/my-targeted-survey
```

Repeated `--contract` filters must refer to the ranking's selected contracts.
The report records the resulting set, retains rank order, and makes no claim to
have retested tokens excluded by that filter.
