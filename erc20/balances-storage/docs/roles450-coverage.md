# NXT role metadata and holder coverage

NXT (rank 427, `0xae7484d162ba80b340eba7769a7a67838b1c16c1`) passed standalone production-WASM and initialized observed-holder checks across finalized BSC blocks **122288006–122289029**, a complete 1,024-block interval. The baseline was the 421-profile `bsc-source450-layouts.json` fixture.

[Final qualification](evidence/roles450-final-qualified.json) binds the completed gates; [summary](evidence/roles450-summary.json) retains their counts. No production code, additional map, protobuf or RPC dependency was added.

## Getter and metadata

The complete NEXST child inherits the unmodified OpenZeppelin `balanceOf(address)` getter, which returns `_balances[account]` at root 0. The pause, owner, role, blacklist and cap checks govern mutations and are absent from the getter path. A canonical trace confirms one balance SLOAD and no external calls. The source review covers inherited contracts, selector dispatch, nonpayable/address validation and unchanged uint256 return.

[Primary-source evidence](evidence/roles450-source-review.json) retains the complete verified source, compiler layout, runtime bytes and getter path. The two declared cap immutable placeholders at offsets 790 and 3923 reconstruct the captured parent/final runtime exactly. The artifact was compared, not locally recompiled. Fresh canonical runtime checks accompany final qualification.

| Storage | Meaning and configured behavior |
| --- | --- |
| Root 0 | Address-to-balance mapping |
| Root 1 | Nested allowances |
| Slots 2, 3, 4 | Supply, name head, symbol head |
| Slot 5 | Pause byte at offset 0; owner address at offset 1 |
| Root 6 | Verified nested role-membership mappings only |
| Slot 7 | Custom-decimals byte |
| Root 8 | Address-to-blacklist boolean mapping |

AccessControl declares a two-word `RoleData`: membership mapping at offset 0 and `adminRole` at offset 1. Grant, revoke and renounce expose membership writes. The internal `_setRoleAdmin` function has no caller in the complete child or inherited runtime paths. The final layout therefore uses `other_mapping_slots` for root 6 and **rejects admin-role word writes**.

The initial proposal used a two-word ignore rule. A synthetic review showed that the generic recursive rule would also accept `membership_hash + 1`, outside the nested boolean value. That proposal and its passing live runs were preserved, then superseded with a narrower layout and fresh native, packaged and holder checks. [Guard evidence](evidence/roles450-width-review.json) retains 12 NXT boundary cases and reproduces the same extra acceptance in existing MUSD and OLY configurations. Those existing profiles were not changed in this cohort; this is a concrete unknown-write guard follow-up, with no demonstrated historical balance mismatch.

## Independent controls and completed gates

[Exact RPC requests and responses](evidence/roles450-rpc-requests.json) retain 102 hash-pinned read-only controls:

- 20 raw-word cases across null, burn, maximum, contract and observed-holder addresses, with 0, 1, 123 and MAX.
- 52 metadata independence cases: 40 configured cases and 12 genuine admin-role overrides whose synthetic writes are deliberately rejected.
- Nine packed-field getter cases for pause, owner and decimals; 18 role-member/admin getter cases across default, minter and arbitrary roles.
- Three rejected calls covering short calldata, call value and dirty address bits.

| Completed gate | Result |
| --- | ---: |
| Native blocks without unresolved writes | 1,024 |
| Actual packaged balances checked against RPC | 14 |
| Emitted zero balances | 0 |
| Initialized holder observations matched | 28 |
| Initialized carry-forward matches | 14 |
| Explicit parent-checkpoint holders / reads | 4 / 8 |
| Fresh final holder balances / zeros | 4 / 1 |
| Mismatches | 0 |

Cold replay retains 14 unknown observations, seven nonzero. Reference rows never seed or repair retained state. Processing uses no balance RPC calls. These are four observed holders, not global holder enumeration.

Successful runs are `out/roles450-qualified-v3`, `out/roles450-rpc-v2`, `out/roles450-holder-coverage-v2`, `out/roles450-wasm-holders-v2` and `out/roles450-final-qualified-v2`. Compact reports are [native/control](evidence/roles450-qualified-v3.json), [RPC](evidence/roles450-rpc-v2.json), [holder coverage](evidence/roles450-holder-coverage-v2.json) and [actual-WASM retention](evidence/roles450-wasm-holders-v2.json).

`tests/fixtures/roles450` contains one captured Extended block with two independent RPC expectations and all controls. The immutable original capture was rebound to the fresh narrow-layout audited output. Seven focused Rust tests pass, covering raw and captured balances, null filtering, exact metadata/preimages, required packed and mapping rules, metadata-only writes, rejected admin/member offsets and missing/corrupt preimages:

`cargo test -p erc20-balances-storage --lib roles450_tests --locked`

Original preflight, helper and asynchronous-export failures remain recorded under `out/roles450-*`; no historical mismatch was discarded. The initial broader qualification, fixtures and evidence are preserved under the original run directories and `out/roles450-initial-*`. The final report lists these attempts. Scratch helpers are archived under `out/next-candidate-rust-scripts/roles450-followup/`.

- Final layout SHA-256: `805b3e47a92e408f02ab82d9243cee818b4e84c72d9f76dc665304f181b5c22a`.
- Final qualification SHA-256: `f1e00595e3f545a0e112797694b3c8ecd7324b60ad35b6307a5ebd103bc45709`.
- Production SPKG SHA-256: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`.

Deployment, future code changes, long-string payload writes, other role-record layouts and combined-cohort results are outside this standalone qualification.
