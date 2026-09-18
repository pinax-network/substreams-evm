# Two short BSC forwarding contracts

USDT (rank 406, `0x5591a8e9a5001f6052d9d749518ae1e6287b7955`) and TCF
(rank 422, `0x5b0c8df579af6f32f8c6d2b0d908d68c54acac39`) have short proxy
runtimes, not standalone token implementations. Ranks and labels come from
the sampled RPC stream, not market capitalization or issuer identification.

Both forward calldata through the EIP-1967 implementation slot and return
the implementation's return or revert bytes unchanged. USDT resolves to
`0x87313388276a317539bb96297523353ed3ef5149`; TCF resolves to
`0x710f61ad214ccee7c46069381a048aa9eab7dcf9`. Outer and effective runtime
hashes and implementation pointers are checked at the parent, sample and
final blocks. Neither proxy has a caller-dependent admin dispatch.

Verified source is unavailable for both proxies and implementations. The
review instead follows the complete historical bytecode paths, ABI validation
and return encoding. USDT reads mapping root 9; TCF reads root 51. Each
implementation reads one balance word and returns its unmodified uint256.
The getter has no caller, time, external-call or balance-dependent branch.
Ordinary nested allowances use roots 10 and 52, respectively.

TCF initially refused all seven active blocks because scalar 352 was unknown.
That word contains a privileged address in its low 160 bits and a temporary
transfer flag in byte 20. The captured writes set and clear the flag around
the transfer-to-dead path. Its separate address getter and complete balance
path establish that this word is unrelated to balance calculation. Four
mixed packed-word overrides check the balance and address getter together.
All other unknown scalar and mapping writes remain errors. No upgrade
pointer or implementation code change is ignored.

Forty raw-word controls test zero, one, 123 and uint256 maximum at five
addresses per token, including the null address. Four allowance controls and
four packed-field controls retain the controlled balance; eight independent
field-getter calls check their storage interpretation. Six invalid calldata
or nonzero-callvalue controls reject. The exported controls are matched to
the exact state overrides sent to RPC. Null balance words remain filtered
from production output.

All gates pass for blocks **122288006–122289029**, inclusive:

| Token | Emitted RPC checks | Initialized observations | Carry-forward matches | Final holders | Final zeros |
| --- | ---: | ---: | ---: | ---: | ---: |
| USDT | 20 | 30 | 10 | 6 | 1 |
| TCF | 14 | 28 | 14 | 4 | 1 |
| **Total** | **34** | **58** | **24** | **10** | **2** |

The full 1,024-block strict native replay has no errors. The actual packaged
stream has complete canonical clocks, and all 34 emitted balances match
hash-pinned RPC; none is zero. An explicit parent checkpoint initializes ten
observed token/holder pairs with twenty balance/storage reads. Actual WASM
events then drive retained state through all 58 reference observations,
including 24 without a new event. Fresh final RPC checks match every retained
holder, including two zeros. Reference rows never repair state; processing
uses no balance RPC calls.

Without initialization, cold replay retains 24 unknown observations,
including seven nonzero balances. This evidence covers observed holders in
the fixed window, not global enumeration or identical cold stream coverage.
It does not establish a new combined-profile checkpoint or first-deployment
initialization.

Two captured transaction fixtures retain four independent RPC expectations.
All seven Rust regressions pass, covering those captures, exact raw/metadata controls,
null filtering, unknown fields, and pointer or proxy/implementation code
changes—including change and restoration without holder writes.

The initial strict refusal is preserved, as is a premature qualifier
invocation that stopped before RPC because the initial scan report was not
yet available. The subsequent qualifier uses a separate output directory.
Source absence is not treated as a source mismatch.

Evidence:

- [Complete runtime and bytecode review](evidence/short450-bytecode-review.json)
- [Qualification](evidence/short450-qualified.json) and [sent RPC requests](evidence/short450-rpc-requests.json)
- [Packaged RPC audit](evidence/short450-rpc.json)
- [Checkpoint and cold coverage](evidence/short450-holder-coverage.json)
- [Actual-WASM holders and final snapshot](evidence/short450-wasm-holders.json)
- [Summary](evidence/short450-summary.json), [fixtures](../tests/fixtures/short450/cases.json) and [Rust tests](../src/short450_tests.rs)

The unchanged production package retains one RPC-free `map_events`, the
shared balance protobuf and default parameters `[]`. These additions are
explicit test layouts; they add no default token, module or production rule.
