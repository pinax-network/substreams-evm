# Five direct getters from ranks 401–450

Five additional BSC profiles pass production-WASM and initialized-holder checks
over **1,024 blocks, 122288006–122289029**. Their complete historical runtime
bytes match previously reviewed direct getters, and each candidate has fresh
getter traces, raw-word controls, metadata controls and boundary bindings.

| Rank | Token | Contract | Balance root |
| ---: | --- | --- | ---: |
| 402 | GULD | `0x14d6c649292c5c9790c5196d1a9ea039307947a1` | 4 |
| 403 | 人生K线 | `0x1a1e69f1e6182e2f8b9e8987e83c016ac9444444` | 0 |
| 417 | BORT | `0x2a846aaaf896ef393ccb76398c1d96ea97374444` | 0 |
| 419 | BNBHolder | `0x44440f83419de123d7d411187adb9962db017d03` | 0 |
| 431 | 145-NO | `0x3477725399b706efabce710fcc54271db8b00c49` | 0 |

The [native qualification](evidence/direct450-final-qualified.json) retains the prior
source/bytecode review and its digest. No proxy pointer, deployment witness or
calculated-balance dependency is inherited. Every valid getter reads exactly
one address-mapping word and returns its unmodified uint256 value. The three
FourERC20 runtimes match the source-bound template; the GULD and 145-NO
templates have bytecode reviews. Newly fetched verified source is not claimed.

The initial survey's empty metadata lists rejected allowance and bookkeeping
writes. The reviewed layouts cover GULD's scalar roots 3/6/7 and allowance
root 5; the three FourERC20 layouts cover fixed roots 2–7 and allowance root 1.
145-NO covers fixed roots 2–5 and the previously reviewed three-level mapping
at root 16 with exactly four record words. Its balance getter never reads that
record. Unknown roots and a fifth record word still reject processing.

**100 independent raw-word controls** cover five addresses per token with zero,
one, 123 and uint256 maximum. **66 independent metadata controls** hold the
balance at 123 while setting each reviewed scalar or mapping-record word to
zero and maximum. The exact RPC override payloads and mapping preimages are
retained. A [preserved setup failure](evidence/direct450-initial-setup.json)
detected a duplicated control address before qualification; the final run uses
five distinct addresses per token.

| Measurement | Five new profiles |
| --- | ---: |
| Emitted balances checked against fresh RPC | 74 |
| Emitted zeros | 23 |
| Observed holders initialized independently | 84 |
| Parent-checkpoint balance/storage reads | 168 |
| Initialized reference observations | 143 |
| Observations matched without a new event | 69 |
| Final holders checked against fresh RPC | 84 |
| Final zero balances | 52 |
| Cold unknown observations | 69 |
| Cold unknown nonzero observations | 12 |

The [packaged audit](evidence/direct450-rpc.json),
[native holder replay](evidence/direct450-holder-coverage.json) and
[packaged holder replay](evidence/direct450-wasm-holders.json) have no mismatches.
Canonical clocks cover empty outputs. Processing makes no balance RPC calls,
and reference rows never initialize or repair retained state.

Five Rust regressions retain five captured transaction fixtures with thirteen
independent RPC expectations, all raw and metadata controls, null-address output
filtering, unknown-root rejection and the four-word record boundary. The
[summary](evidence/direct450-summary.json) reports this cohort only; combined
configuration validation is a separate gate. This does not establish global
holder enumeration, identical cold stream row coverage, arbitrary family
support or every future metadata path.
