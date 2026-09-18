# SLX and PIN: permits, allowances and null-address burns

SLX (rank 436) and PIN (rank 444) pass independent RPC and initialized-holder
checks across BSC blocks **122288006–122289029**, inclusive. Ranks count
observations in this sampled RPC stream, not market capitalization.

| Token | Contract | Emitted RPC checks | Initialized observations |
| --- | --- | ---: | ---: |
| SLX | `0x8a063a9ff4de28dcb87117cc759be6ce70e09f81` | 18 | 27 |
| PIN | `0xb86126b872e3f23bc62d6a4a3152ec65202a2073` | 12 | 26 |
| **Total** | | **30** | **53** |

## Getter and metadata review

SLX inherits the direct OpenZeppelin ERC20 getter at mapping root 0. Its
constructor-only child also inherits ERC20Permit; permits change allowances
at root 1 and nonces at root 7 without changing the getter. Supply and string
heads occupy slots 2–6. Seven declared EIP712 immutable replacements were
checked before reconstructing the complete historical runtime exactly.
Four independent nonce getter calls bind the overridden word and its preimage.
Nonce-only writes emit no balance and require no cache.

PIN's getter reads mapping root 1 directly. Root 2 stores nested allowances;
slots 0 and 3–10 contain the owner, supply, string heads, fee and routing
metadata. Its compiler runtime reconstructs the complete historical runtime
after one declared CBOR suffix replacement. The original sampled strict
rejections were allowance writes outside the tentative configuration, not
balance value mismatches. The captured block at 122288204 binds the original
rejected key to root 2 and reproduces rejection when that allowance rule is
removed.

PIN's sell fee subtracts tokens from the pair and credits the null address
without reducing total supply. At block 122288204, the captured null balance
`113119930682987880420421` matches an independent canonical `balanceOf(0)`
call. Both the canonical RPC module and storage output filter that address;
the two nonnull output balances still match RPC.

Parent, sample and final runtime checks bind these reviews to historical
code. The source comparison uses returned Sourcify compiler artifacts, not a
claimed local compiler rerun. Fresh getter traces each read one balance word
without external calls. Forty raw-word controls cover five addresses per
token at zero, one, 123 and uint256 maximum. Sixty-eight exact metadata
overrides preserve the controlled balance. Dynamic string payloads and
unknown fixed fields or roots remain unqualified and stop processing.

## Packaged output and holders

The full native replay has no errors. The first packaged RPC audit captured
all 1,024 blocks but stopped on a transport error after 62 checked blocks and
two balance checks. That failed report remains preserved. A separate complete
audit rechecked the exact SHA-bound captured output, all 1,024 canonical
headers and all 30 emitted balances, with zero mismatches or emitted zeros.

An independent parent checkpoint initializes 12 observed token/holder pairs
using 24 balance/storage reads. Actual packaged-WASM events then match all
53 observations, including 23 without a new event. Fresh final RPC balances
match all 12 retained holders, including one zero. Processing uses no balance
RPC calls and reference rows never repair state. Without initialization,
there are **23 unknown observations, including 19 nonzero balances**. This
does not establish global holders or identical cold-stream coverage.

Nine Rust regressions retain two captured transaction fixtures with four
independent nonnull RPC expectations plus the independently checked null
balance. They cover raw words, metadata isolation, nonce-only writes, the
original allowance rejection, null filtering and unknown-storage guards.

Evidence:

- [Source, runtime, controls and native replay](evidence/permit450-qualified.json)
- [Preserved incomplete audit](evidence/permit450-rpc-incomplete.json) and [complete RPC recheck](evidence/permit450-rpc.json)
- [Checkpoint and cold coverage](evidence/permit450-holder-coverage.json)
- [Actual-WASM retained holders and final snapshot](evidence/permit450-wasm-holders.json)
- [Summary](evidence/permit450-summary.json), [fixtures](../tests/fixtures/permit450/cases.json) and [Rust tests](../src/permit450_tests.rs)

The explicit test layouts use the unchanged production package and its one
RPC-free `map_events`, shared protobuf and empty default configuration.
