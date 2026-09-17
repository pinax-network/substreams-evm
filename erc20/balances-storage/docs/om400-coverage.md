# OM minimal-proxy balance coverage

OM, ranked 376 in the BSC reference sample, passes emitted-balance and
initialized-holder checks for blocks **122288006–122289029**. The reviewed
contract is `0x1abe0ec4829f606c392327fb7dc9c9a5733dd9f4`.
The result applies to its pinned runtime and the tested interval; it does not
qualify arbitrary proxies or enumerate all holders.

The [completed standalone qualification](evidence/om400-final-qualified.json)
binds the reviewed layout to the completed package, RPC, initialized-holder,
checkpoint and capture gates. It also refreshes the canonical parent, first
and final headers and both runtime boundaries. The earlier
[inspected qualification](evidence/om400-qualified.json) is retained as
provenance from before those gates ran; its outstanding-gate wording records
that earlier stage, not the final result. The final layout is
`out/om400-final-qualified/added.json` and has the same digest as the tested
layout.

| Check | Result |
|---|---:|
| Canonical production-WASM blocks | 1,024 |
| Independently checked emitted balances | 22 |
| Emitted zero balances | 0 |
| Observed holders initialized at the parent block | 13 |
| Parent checkpoint RPC reads | 26 |
| Reference observations reproduced by retained state | 33 |
| Initialized carry-forward observations | 11 |
| Final holders checked against fresh RPC | 13 |
| Final zero balances | 1 |
| Value mismatches | 0 |

The [reference-only observations](evidence/om400-reference-gaps.json) are eleven
reads of the token contract's unchanged zero balance. A cold mapper cannot infer
that value from an absent write. The independently checked parent checkpoint
provides it to the off-chain holder experiment. Reference rows never repair
retained state, and production `map_events` remains RPC-free and stateless.

The [bytecode review](evidence/om400-review.json) binds the exact 45-byte
ERC-1167 forwarder to implementation
`0xa98315cb8a3b48ffb940c2402e13c0d04f543dfa`. The implementation is 4,275 bytes;
its `balanceOf` body hashes an address with root slot **0**, performs one
`SLOAD` at program counter 548, and returns that word directly. The complete
path has only selector, nonpayable and ABI-validity branches. It has no supply,
owner, allowance, initializer, other-holder or external-call dependency.

The explicit fixture layout also reviews allowance mapping root **1** and fixed
slots **2** (supply), **5** (owner), **6** (short name), **7** (short symbol),
and **8** (initialization flag). The review includes 20 raw balance controls,
12 independent metadata controls, three invalid-call/revert controls, and a
read-only simulation of initialization. Long-string data outside the reviewed
fixed words remains unsupported and fails closed. No verified source or
deployment coverage is claimed.

The [first packaged audit](evidence/om400-initial-rpc-incomplete.json) ended on an
RPC transport error after 998 blocks and 20 matching balances. Its complete
WASM capture was preserved. The [successful audit](evidence/om400-rpc.json)
rechecked the exact capture, all canonical clocks and all 22 emitted balances
through the final block; the report retains the earlier failure's digest and
counts. The [holder report](evidence/om400-wasm-holders.json) binds the same
capture and package to the parent checkpoint and final fresh RPC snapshot.

The Rust regression fixture is in `tests/fixtures/om400`. It contains one
captured Extended block with two independent RPC expectations plus the raw
and metadata control values. `src/om400_tests.rs` also covers unreviewed mapping
writes and implementation-code changes. This cohort's standalone evidence is
summarized in [om400-summary.json](evidence/om400-summary.json); combined-cohort
replay and repository-wide checks are separate gates.
