# Minimal proxies and holder replay from creation

The explicit [25-token fixture](../tests/fixtures/bsc-clone-family-layouts.json)
adds four ranked BSC tokens backed by the same verified implementation:

| Rank | Contract | Qualification |
| ---: | --- | --- |
| 26 | `0xcccbd077b612a42f896b53e24a11c36d32117777` | First CREATE at block 122288172, initialization and initial distribution |
| 28 | `0x8e31486873aa8a20cf5706d8445496e6d5f17777` | Existing runtime, getter controls and historical boundaries |
| 37 | `0x1dc6cbcf7da108e58f144f30d47d5e579d467777` | Existing runtime, getter controls and historical boundaries |
| 40 | `0x3e26ff1d7c6f72c5dbb6f6ac3833ccffec0f7777` | Existing runtime, getter controls and historical boundaries |

All four contain the exact standard 45-byte
[ERC-1167 forwarder](https://eips.ethereum.org/EIPS/eip-1167), pointing to
`0x8b4329947e34b6d56d71a3385cac122bade7d78d`. The clone runtime Keccak-256 is
`0xa5541d9f45d6c04ba0111363e5d8bcf65eb2de9b037fd9b1d6cb0fc733b63b8f`;
the implementation runtime is
`0x852cb4241a5b2210d523900e6a0af0a52c5c5bccbf4fbb236cd2cf068b8b3ab8`.

Sourcify's verified `TokenV2` source and compiler layout match the historical
implementation runtime exactly. `TokenV2` inherits OpenZeppelin's `balanceOf`,
which returns mapping base **51** without a conversion or fallback. Allowances
use 52 and permit nonces 153. Initialization, supply, ownership, permit-domain
strings, metadata and transfer restrictions occupy reviewed non-balance fields.
The two long-string words for `metaURI` are explicitly listed; the verified code
assigns this string only during its single initialization. Unknown writes still
fail instead of being silently discarded. [Source and storage layout](evidence/clone-source-layout.json),
[bound implementation inspection](evidence/clone-implementation-inspection.json),
[clone getter controls](evidence/clone-getter-inspection.json),
[three additional clone inspections](evidence/clone-family-getter-inspections.json).

## Mapper behavior

The new optional `minimal_proxy` field contains only the implementation address
and code hash. Parsing constructs the exact standard forwarding runtime and
requires its hash to equal the configured token hash. It cannot be combined with
another proxy kind. The RPC qualifier checks both runtimes; the map rejects
implementation code changes, including changes without token writes and
change-and-restore. Nonstandard or vanity clone variants are not silently accepted.

Minimal proxies may use the explicit `deployment` configuration introduced in
the [previous follow-up](deployment-holder-coverage.md). At block 122288172,
the CREATE spans ordinals 7077–7081 and installs code at 7080. Initialization
and distribution follow within the successful transaction. The map emits all
**143 initial holder balances**, whose raw amounts sum to the minted
`1000000000000000000000000000`. Every first holder storage word is zero.
The committed protobuf regression retains that actual transaction and original
header, omitting unrelated transactions/system changes. The WASM tests stream
full blocks. [Fixture/artifact digests](evidence/clone-artifacts.json).

There remains one RPC-free `map_events`, the shared `evm.balances.v1.Events`
protobuf, and an empty default layout list. No intermediate map, cache, protobuf
or ingestion RPC was added.

## Deployment comparisons

`test-ranked` now runs runtime qualification for caller-reviewed layouts,
including the explicit deployment boundary and proxy dependencies. It no longer
mistakes the parent's empty runtime for an incorrect declared post-deployment
hash. Empty successful `balanceOf` calls immediately before a qualified CREATE
are recorded with null `rpc`/`match` values and classification
`unavailable_before_qualified_deployment`. They are excluded from successful
balance comparisons. Provider errors and malformed/nonempty responses remain
errors; an unqualified hypothesis receives no exception.

The [two-deployment native replay](evidence/clone-deployments-survey.json) spans
298 consecutive blocks, with **496 matching before/after value checks**,
**144 unavailable pre-deployment comparisons**, zero RPC errors, zero value
mismatches and zero mapper errors. The 144 are the previously diagnosed absent
contracts: 143 clone holders and one direct-token holder. Original failed reports
are retained. The [three existing-clone replay](evidence/clone-family-survey.json)
adds **840 matching before/after checks** over the same blocks, also with zero
RPC or mapper errors. Both retain `coverage_gap` because unchanged reference
participants do not produce storage events.

## Actual WASM audits

| Inclusive range | Configured tokens | Tokens with emitted rows | RPC balances checked | Zeros | Mismatches |
| --- | ---: | ---: | ---: | ---: | ---: |
| 122288160–122288671 | 22 | 22 | 38,052 | 5,060 | 0 |
| 122288672–122288927 | 25 | 24 | 17,572 | 2,992 | 0 |

These two windows are consecutive and do not overlap each other. The earlier
window used the [22-token fixture](../tests/fixtures/bsc-clone-layouts.json);
the three additional clones are tested in the second. The four clones contribute
212 checks in the second window (75, 54, 43 and 40 respectively).
Fruit (`0x45056c2627c9e60753aeef604ee9575709f8e88f`) emitted no rows in the
second window; it was exercised in the first and in the holder replay.
Reports: [first](evidence/clone-rpc.json), [second](evidence/clone-family-rpc.json),
[first token counts](evidence/clone-rpc-token-counts.json),
[second token counts](evidence/clone-family-rpc-token-counts.json).

Both audits ran the same WASM. A README update explaining the 25-token fixture
changed package metadata between runs. The final package digest is
`88248f3f1e9296ea543ff8253cd106f6f122131887b27c597e875397f3d2bbb0`;
the WASM digest is
`10d5f5d7200a38c61ad12ef403e181895cd114f25d7164e26873bd7b39325aba`.

## Retained holder state

The [25-token holder replay](evidence/clone-family-holder-coverage.json) covers
**298 consecutive blocks, 122288160–122288457**. All **39,347 reference
observations across all 25 tokens match**, with zero unknown or mismatching
initialized balances. Existing tokens use a 17,394-holder test checkpoint
(34,788 setup balance/storage RPC reads). The two new deployments instead
initialize **19 and 184 observed holders** from verified empty initial storage.
No reference values repair state and ongoing processing uses no balance RPC.
Runtime and canonical-header checks still use RPC.

The new clone's **432 observations across 184 holders** all match, including
132 carried-forward observations with no write in that block. It requires no
RPC balance checkpoint. The earlier [22-token replay](evidence/clone-holder-coverage.json)
matched 38,609 observations; those are included in the expanded result, not
additional independent coverage. Without checkpoints for the existing tokens,
15,383 observations remain unknown, including 8,401 nonzero balances. This is
bounded observed-holder coverage, not global holder enumeration or a deployed
production bootstrap.

## Validation and remaining work

**117 workspace Rust library/binary tests pass**, including captured clone
initialization, exact runtime/target binding, exclusive proxy configuration,
implementation changes and restore, runtime qualification, and strict handling
of pre-deployment responses. The known generated-proto doctest issue is separate
from the CI-equivalent library/binary check. Clippy with warnings denied,
workspace WASM compilation, targeted formatting and diff checks also pass.
All executable tooling is Rust.

The rank-7 computed token was outstanding in this batch; the
[address-derived follow-up](computed-holder-coverage.md) adds its explicit rule.
Shared-default changes, remaining unreviewed top-50 layouts and full holder
bootstrap remain outstanding. Passing these windows
does not establish universal ERC-20 behavior or identical raw event-row coverage.

Reproduce with the existing `audit-rpc`, `test-ranked` and `holder-coverage`
commands using `tests/fixtures/bsc-clone-family-layouts.json`. The consecutive
holder capture combines blocks 122288160–122288287, 122288288–122288329 and
122288330–122288457. The original ranking/reference pair remains
`out/top50-1024/report.json` and `reference.jsonl`; their recorded digest must
match. Supply new output directories and the appropriate Substreams endpoint.
