# WIN and THE public-mapping getter coverage

WIN (rank 409, `0xb10cb07ca2cdac77fbb5707f6690301f9d036f45`) and THE (rank 428, `0xf4c8e32eadec4bfe97e0f595add0f4450a863a11`) passed standalone production-WASM and initialized observed-holder checks over finalized BSC blocks **122288006–122289029**, a complete 1,024-block window.

The final qualification is [public450-final-qualified.json](evidence/public450-final-qualified.json); [public450-summary.json](evidence/public450-summary.json) contains compact counts. No production code, protobuf, RPC dependency or additional map module was added.

## Getter and metadata review

Both contracts declare `balanceOf` as a public `mapping(address => uint256)`. Their complete source contracts inherit interfaces only, so Solidity generates an unmodified direct getter. The returned Sourcify compiler runtime equals the historical on-chain runtime byte-for-byte; no linking, immutable or metadata transformations are needed. The source artifacts were compared, not locally recompiled.

[Source evidence](evidence/public450-source-review.json) retains complete primary verified source files, runtime bytes, compiler storage layout and executed getter paths. Fresh canonical checks bind the runtime at parent block 122288005 and final block 122289029. Sample getter traces show one balance SLOAD and no external calls. The review includes selector dispatch, nonpayable guard, canonical address/length validation and unchanged uint256 ABI return.

| Contract | Balance root | Nested allowance root | Reviewed fixed metadata |
| --- | --- | --- | --- |
| WIN | 5 | 6 | Owner 0, pending owner 1, issuer/decimals 2, supply 3, cap 4, name 7, symbol 8 |
| THE | 1 | 2 | Supply 0, initialMinted/minter 3, redemption receiver 4, merkle claim 5 |

WIN slot 2 packs the issuer in the low 160 bits and decimals at byte offset 20. THE slot 3 packs `initialMinted` at byte offset 0 and minter at byte offset 1. THE's name, symbol and decimals are constants, so they need no storage ignore rules. The compiler-generated balance getter reads none of these fields.

Independent canonical RPC controls cover:

- 40 raw-word cases: five addresses per token, each with 0, 1, 123 and MAX. Null-address getter results are captured; production event filtering remains covered separately.
- 52 metadata cases: every reviewed fixed/allowance key set to zero or MAX with raw balance zero or 123. Exact override payloads and allowance preimages are retained.
- Nine packed-field getter cases covering zero, MAX and a mixed word, with independently checked decimals, boolean and address projections.
- Six rejected calls: short calldata, nonzero call value and noncanonical address bits for each token.

Unknown fixed or mapping roots remain errors. Dynamic name/symbol payloads outside their fixed roots are not qualified. No general deployment or future-code-change claim is made.

## Completed results

| Gate | WIN | THE | Total |
| --- | ---: | ---: | ---: |
| Emitted balances checked against RPC | 18 | 19 | 37 |
| Emitted zero balances checked | 0 | 2 | 2 |
| Observed-holder reference rows | 30 | 28 | 58 |
| Explicit parent-checkpoint holders | 16 | 16 | 32 |
| Cold unknown reference rows | 12 | 8 | 20 |
| Cold unknown rows with nonzero RPC balance | 1 | 7 | 8 |

All emitted and initialized holder values matched. The parent checkpoint used 64 explicit balance/storage reads. Retaining actual production-WASM output matched all 58 reference observations, including 21 initialized carry-forward matches. A fresh final RPC snapshot matched all 32 initialized holders, including five zeros.

The 20 cold unknown observations are preserved; eight are nonzero. Reference rows never seed or repair holder state. These checks cover the 32 observed holders, not an exhaustive holder universe.

Successful local runs are `out/public450-rpc`, `out/public450-holder-coverage` and `out/public450-wasm-holders`. The finalizer independently binds every emitted RPC row, all 1,024 clocks, full native block identities, package/layout/reference/checkpoint/snapshot hashes, and fresh canonical parent/first/final headers. Compact reports are [RPC](evidence/public450-rpc.json), [holder coverage](evidence/public450-holder-coverage.json) and [WASM retention](evidence/public450-wasm-holders.json).

## Captures and qualification artifacts

`tests/fixtures/public450` contains two captured Extended blocks with four independently audited RPC expectations plus all raw, metadata, packed and invalid-call controls. Five focused Rust regressions pass, covering captured values, raw zero/MAX values, null filtering, exact metadata/preimage payloads, required packed-slot rules and unknown-root rejection:

`cargo test -p erc20-balances-storage --lib public450_tests --locked`

`out/public450-qualified` and its [compact report](evidence/public450-qualified.json) preserve the earlier source/control/native-only qualification. `out/public450-final-qualified/added.json` is the completed standalone layout input for later combined checks. The initial scratch-helper syntax failure is retained in `out/public450-qualify.log`; the corrected execution is `out/public450-qualify-v2.log`. No live gate failed in this cohort.

- Layout SHA-256: `ebd3b0dc4303ea5c5f5efcdc0a11db561b6c5e048fe50bc6a96734d1a093ac77`.
- Production package SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`.
- Final qualification report SHA-256: `2c4a637cd209f39644b1733350744e0311e06dc1334bdfef807c762c2568585a`.

Scratch Rust helpers are archived under `out/next-candidate-rust-scripts/public450-followup/`. The baseline for this work was the 412-profile fixture; combined-cohort qualification is separate.
