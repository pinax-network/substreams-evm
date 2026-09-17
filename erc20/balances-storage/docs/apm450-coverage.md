# BSC APM rank 447: protected zero-word fallback

APM (`0x1d5beb0a44c4283b993665ca3f8b5d4a74088293`) passed standalone emitted-balance and initialized observed-holder checks across finalized BSC blocks **122288006–122289029** (1,024 blocks). The production package emitted 16 balances; all 16 matched independent canonical RPC calls. The final qualification is [apm450-final-qualified.json](evidence/apm450-final-qualified.json), with compact totals in [apm450-summary.json](evidence/apm450-summary.json).

This is one pinned direct runtime and a bounded historical window. It does not establish every holder, deployment initialization, all custom administrator operations, or support for future code changes.

## Why the earlier plain-storage comparison differed

The valid `balanceOf` body reads the address mapping at root 0. If that raw word is zero, it returns scalar slot 7 instead. In this window slot 7 equals **7,000,000,000**. A plain mapping read therefore incorrectly reports zero for accounts whose public balance is the configured fallback.

The two earlier mismatches were the prestates of holders `0x18868500c856c5eb263363b87b62e30ef682f4b8` and `0x395297f5499fc0dee834ec7b6e8033ce03aeb993`, before blocks 122288465 and 122288603 respectively. Fresh hash-bound RPC checks confirmed raw zero and public balance 7,000,000,000 in both cases. These cases are retained in [the native candidate report](evidence/apm450-native-candidate.json) and `tests/fixtures/apm450/prestate.json`.

The existing `zero_balance` layout projection handles this behavior. Slot 7 remains a protected dependency: any persisted change rejects processing, including a change restored later in the same block. No production code or additional map module was required.

## Reviewed behavior and metadata

The getter review covers the valid raw-zero and nonzero paths, ABI argument validation, nonpayable guard and null-address revert. The valid body is at bytecode offsets 1234–1280: mapping SLOAD at 1259, then fallback SLOAD at 1269 only for a zero raw word. Neither valid branch reads caller, origin, clock or external state. Runtime length is 4,687 bytes, with Keccak-256 `0x07d7a76fabdd920d0c5be9ec6b2097b65792b82b89d4306f0dde14dce3634a03`.

The reviewed layout permits ordinary metadata at fixed slots 3 (name), 4 (symbol), 5 (decimals), 6 (supply), and 8 (owner), plus nested owner-to-spender allowance root 1. The first strict replay rejected an allowance write at block 122288187; its owner/spender preimages and independent getter were reviewed before permitting root 1. That original rejection is preserved in [getter review evidence](evidence/apm450-getter-review.json).

The ordinary getters and executed paths were reviewed independently even where those fields were unchanged in the replay window. Independent RPC controls include:

- 36 raw/fallback combinations over four nonzero addresses, raw values 0/1/MAX and fallback values 0/7,000,000,000/MAX.
- 20 ordinary metadata controls, each field set to zero or MAX with raw balance zero or 123.
- Four allowance controls covering zero/MAX allowance with raw balance zero or 123.

[Field evidence](evidence/apm450-fields.json) and [allowance evidence](evidence/apm450-allowance.json) preserve these controls. Per-control fallback overrides represent independently configured baselines; they do not authorize slot 7 changes during ingestion.

Custom administrator state remains unreviewed and strict. In particular mapping root 16, scalar slot 30 and other custom fields are not ignored. Long-string data beyond the reviewed fixed roots is also outside this qualification. The null address reverts at the getter and is already filtered from emitted balances.

## Completed gates

| Gate | Result |
| --- | --- |
| Complete native replay | 1,024 blocks, 16 emitted rows, no errors |
| Production WASM versus RPC | 16/16 values match, no emitted zero balances |
| Explicit parent checkpoint | 11 observed holders, 22 RPC/storage reads |
| Observed-holder parity | 25/25 reference observations match |
| Initialized carry-forward | Nine reference-only observations match retained state |
| Fresh final snapshot | All 11 initialized holders match RPC; none are zero |
| Captured Rust fixtures | Four blocks, eight independent RPC expectations |
| Focused Rust regressions | Five tests covering captures, controls, original mismatches and rejection boundaries |

The explicit checkpoint contains four raw-zero holders whose public balances use the fallback. Without initialization, nine reference-only observations remain unknown; **all nine are nonzero**. Reference events never seed or repair retained state. The holder checks cover the 11 observed holders, not an exhaustive holder universe.

The finalizer binds actual emitted output to every audited RPC row, all 1,024 captured clocks to complete native block identities, the package, reviewed layout, reference, checkpoint and final snapshot hashes. Fresh canonical checks cover parent block 122288005, first block 122288006 and final block 122289029, with runtime and fallback qualification repeated at the window boundaries.

## Evidence and reproduction

Successful local runs are `out/apm450-rpc`, `out/apm450-holder-coverage` and `out/apm450-wasm-holders`. Their compact reports are [RPC](evidence/apm450-rpc.json), [holder coverage](evidence/apm450-holder-coverage.json) and [actual WASM retention](evidence/apm450-wasm-holders.json). `out/apm450-final-qualified/added.json` is the reviewed layout for later combined-cohort checks.

The earlier [native-only qualification](evidence/apm450-qualified.json) is immutable provenance; its wording correctly says the live gates were still separate at that stage. The [final qualification](evidence/apm450-final-qualified.json) records their completed status and hashes. Original inspection rejects and setup/build failures remain in their original local paths.

- Layout SHA-256: `8cba3c750badbf286f9c7ae706802d707c0df9ba3974cd9b73a07c15be86128f`.
- Production package SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`.
- Final qualification report SHA-256: `253bed1eda948532902b7adde5fbd98d9a127994992a223ef65e72291ddee3f9`.

Captured input, independent controls and checkpoint values are in `tests/fixtures/apm450`. Run `cargo test -p erc20-balances-storage --lib apm450_tests --locked` from the repository root. Scratch Rust helpers are archived under `out/next-candidate-rust-scripts/apm450-followup/` after use. No RPC is used by the production projection or these captured tests.
