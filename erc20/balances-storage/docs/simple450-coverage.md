# MGOL and NFTC: source-bound direct balances

MGOL (rank 413) and NFTC (rank 430) pass independent RPC and initialized-holder
checks across BSC blocks **122288006–122289029**, inclusive. These ranks measure
observations in the sampled RPC stream, not market capitalization.

| Token | Contract | Emitted RPC checks | Initialized observations | Final holders |
| --- | --- | ---: | ---: | ---: |
| MGOL | `0x750aed042ce71808bc1af9d06b5f2fa17edea3c7` | 16 | 29 | 4 |
| NFTC | `0x1868d0c4d8cabf094d76f5293cf2b2662e8cee43` | 21 | 27 | 16 |
| **Total** | | **37** | **56** | **20** |

The complete child contracts contain constructors only and inherit the
OpenZeppelin `balanceOf` getter, which returns mapping root 0 directly. The
Sourcify compiler runtime equals the historical runtime byte for byte, without
immutable or metadata substitutions. Fresh getter traces show one balance
storage read and no external calls. Parent, sample and final runtime bindings
all match. This comparison uses Sourcify's returned compiler output; it does
not claim a separate local compiler rerun.

The reviewed metadata consists of allowance root 1, total supply at slot 2,
name at slot 3 and symbol at slot 4. Forty independent raw-word controls cover
zero, one, 123 and uint256 maximum at five distinct addresses per token,
including the null address. Sixteen exact metadata state overrides preserve
the controlled balance. Null balance words are tested internally but remain
filtered from production output. Dynamic string payloads and unknown roots
remain outside the configuration and stop processing when encountered.

The native replay covers all 1,024 complete Extended blocks with no errors.
The fresh packaged-WASM capture has complete canonical clocks, and all 37
emitted balances match hash-pinned RPC. None of those emitted balances is zero.
An independent parent checkpoint initializes 20 observed token/holder pairs
with 40 balance/storage reads. Actual packaged output then matches all 56
observations, including 19 without a new event. A fresh final RPC snapshot
matches all 20 retained holders, including one zero balance. Processing uses
no balance RPC calls, and reference rows never repair state.

Without that checkpoint, cold replay retains **17 unknown observations,
including eight nonzero balances**. This is bounded observed-holder coverage,
not global enumeration or identical cold event coverage.

Five Rust regressions retain two captured transaction fixtures and five
independent RPC balance expectations. They reproduce all 40 raw and 16
metadata controls and keep unknown fixed fields and mapping roots guarded.
The first audit invocation used an invalid subcommand and made no RPC audit;
its local log is preserved separately from the successful run.

Evidence:

- [Source, runtime, controls and native qualification](evidence/simple450-final-qualified.json)
- [Packaged RPC audit](evidence/simple450-rpc.json)
- [Checkpoint and cold coverage](evidence/simple450-holder-coverage.json)
- [Actual-WASM retained holders and final snapshot](evidence/simple450-wasm-holders.json)
- [Summary](evidence/simple450-summary.json), [fixtures](../tests/fixtures/simple450/cases.json) and [Rust tests](../src/simple450_tests.rs)

These explicit test layouts use the unchanged production package and its
single RPC-free `map_events`; they add no production token default, protobuf,
cache, store or getter rule.
