# YBC reward trace recovery

YBC remains unqualified. Its actual historical `balanceOf` includes a reward
that is absent from the raw holder word. The earlier detailed trace exceeded
the provider's 167,772,160-byte response limit; omitting per-instruction memory
and storage snapshots now recovers the execution. The original failure remains
in [the earlier evidence](evidence/ranks201-250-mismatches.json).

[Current evidence](evidence/ybc-compact-trace.json) binds the token, dependency
and pool code to the same canonical historical block. The production map,
protobuf output and configured-token defaults are unchanged.

## Actual counterexample

At block **122288005**, hash
`0xdf9d02ee1c9e345bfcdb971e488a20f7f51dc03d3d8e2e7bbc8fc9f0bb0814cf`:

- Token: `0xebc2d768147f2d058f4266bb57e34ca1b6ef1319`.
- Holder: `0x2aafce28b5552e216811fb5a1cb015178a29b461`.
- Raw balance word: `1000000000`.
- External reward: `12059327966974017973509`.
- RPC balance: `12059327966975017973509`, exactly raw plus reward.

The second `calculateReward` tuple word is **stopping hour 1935**. Earlier
diagnostics called it `dynamic_reward`; that label was incorrect. The verified
token source consumes it as `stopedhour` in `_claimStaticReward`, then uses it
to advance `userLasts.lastCycle`. `balanceOf` adds only the first tuple word.
The new report corrects the label without modifying historical reports.

## Recovered dependencies

The dependency `0xe8058da568eb94194588bc9bae53a5e584fe7471` calls back into the
token for user state, cycle rates, rewards and burn flags. This observed path
starts at saved cycle 1695, visits 240 hourly periods and returns stopping hour
1935; the current cycle is 4473. It also reads pool
`0x5473f664eaa1c6fea8306133241000b758cb326a` and invokes the token getter for
that pool. The nested reward call for the pool returns zero.

The compact execution contains **446,934 instructions**, **991 storage-read
steps** and **971 call instructions**. Its 95,966,227-byte trace fits beneath
the provider limit. Account attribution checks **1,962** read/call instruction
positions against canonical historical code. All **986 distinct storage
words**—984 in the token and two in the pool—also match independent canonical
`eth_getStorageAt` reads and `prestateTracer`. No dependency-account storage
word is read on this particular execution path.

The [Rust regression fixture](../tests/fixtures/ybc-reward-trace/case.json)
retains 5,868 trace steps: every load and following result, every call, all
frame transitions and the root boundaries. It retains only the top two stack
words needed for attribution. This is an explicit projection of the complete
trace, whose digest is recorded; it is not a replacement full execution trace.
Attribution on the projection equals attribution on the original trace.

## Reusable Rust inspection

Both `inspect-balance` and `inspect-ranked` accept `--compact-trace`. It disables
per-step memory, storage snapshots and return-data capture while retaining
stack and depth. The default detailed format remains available for mapping
preimages. The selected trace configuration is recorded in the report.

```sh
cargo run --locked -p erc20-balances-storage-tools -- inspect-balance \
  --contract 0xebc2d768147f2d058f4266bb57e34ca1b6ef1319 \
  --address 0x2aafce28b5552e216811fb5a1cb015178a29b461 \
  --balance-slot 0x0000000000000000000000000000000000000000000000000000000000000000 \
  --block 122288005 --compact-trace --output out/ybc-inspection
```

For this reward-bearing holder, the trace succeeds but the inspection exits
incomplete at the final word control. Raw words zero, one and 123 return that
word plus the same reward. A maximal uint256 raw word causes Solidity arithmetic
panic `0x11`; a separate canonical read-only call preserves the exact revert
response. Earlier successful controls now remain in the incomplete report.
No transaction is submitted and no mismatch is promoted to a direct layout.

## Remaining work

The trace establishes an actual dependency path, not the complete reward
formula. Qualification still needs all arithmetic and branch behavior,
dependency-change guards, durable initialization and affected-holder output
when rewards change without a raw balance write. The dependency source was
unavailable in the earlier source lookup. BSC holder coverage must continue to
exclude YBC until those requirements are met; this diagnostic adds no store or
extra map module.
