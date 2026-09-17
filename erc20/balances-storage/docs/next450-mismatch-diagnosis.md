# Seven balance mismatches in BSC ranks 401–450

The exploratory survey of 50 tokens found **7 mismatches in 1,644 RPC value
checks**, with no RPC value errors. All seven were reproduced at their original
canonical block hashes. RPC storage agreed with Firehose storage in every case.
The mismatches concern two calculated getters; **no layouts are promoted by
this diagnosis**.

The [tracked evidence](evidence/next450-mismatch-diagnosis.json) preserves all
seven original failures with full holder addresses, raw and returned amounts,
canonical hashes, runtime bindings, bytecode paths and controls. The survey
covers blocks **122288006–122289029**. Each failure is a `before` comparison:
its actual RPC height is the survey block minus one, recorded as `fresh_height`.

| Rank | Token contract | Failures | Cause |
| ---: | --- | ---: | --- |
| 411 | OG — `0xe18102869d32181aea317a40c5c4ce90ca591913` | 5 | Raw mapping0 plus two external calculations |
| 447 | APM — `0x1d5beb0a44c4283b993665ca3f8b5d4a74088293` | 2 | Zero mapping words return the value of slot7 |

## APM: existing fallback behavior needs qualification

For a nonzero address, the observed getter returns its mapping0 word when
nonzero, otherwise scalar slot7, currently **7,000,000,000 raw units**. The null
address reverts. The matrix retains **80 controls**: 64 successful combinations
of holder, raw word and fallback value, plus 16 null-address reverts. Values
include zero, one, 123 and uint256 maximum; nonzero raw words are unchanged.

The existing `zero_balance` rule can express this behavior. Next, bind the
runtime and pinned fallback value, review the remaining metadata writes, and
run packaged output and initialized-holder comparisons. Slot7 must remain a
protected dependency: a change requires requalification and rebuilding affected
holder state. This investigation does not establish holder enumeration.

## OG: balances depend on external calculations and time

The executed getter reads mapping0 and adds the first returned word from two
calls to helper `0x1430c0bd0d023690f7aee3666daf265c498cc6a5`, selected through
slot6. The selectors are `0xe8e8fe04` and `0x7ac1f9f8`. Their observed execution
reads token records, clock values and external pool state. No delegate-call
proxy execution was observed.

Across the five original holders, zeroing the raw word leaves a positive
computed amount; setting it to one or 123 adds that amount. Setting it to
uint256 maximum triggers checked-addition overflow. Ten simulated helper-code
controls independently isolate the two contributions: with raw123, both helper
results zero produce123; both results17 produce157.

Clock-only controls change the first holder's balance without changing storage:

| Simulated clock | Returned raw units |
| --- | ---: |
| Original | 1062462782039865649749 |
| One hour later | 1062835281832210590658 |
| One day later | 1071331940430322563175 |

Moving the stored epoch in slot29 backwards by the corresponding duration
produces exactly the same results. Two separate pool-reserve perturbations did
**not** change this sampled result; that negative evidence remains recorded.

OG needs independently reviewed helper arithmetic, rounding and overflow
behavior, runtime bindings, initialized holder records and shared state, and
recalculation when time changes without a mapping write. Adding ignored storage
roots cannot provide that parity. The complete helper arithmetic and unexecuted
branches remain unqualified.

All controls are read-only simulations. Evidence is based on historical
executed bytecode and RPC checks; verified source is not claimed. One initial
memory-enabled trace request failed; compact traces subsequently completed for
all seven cases, and the initial failure remains recorded separately. No
production code, module, protobuf or profile changed in this diagnosis.
