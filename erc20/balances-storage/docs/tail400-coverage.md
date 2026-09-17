# Ten more BSC token layouts: 393 configured profiles

Ten additional profiles pass separate production-WASM and initialized-holder
checks over **1,024 blocks, 122288006–122289029**. Adding them to the published
383-profile LP baseline gives **393 distinct configured contracts**.
The [combined replay](evidence/tail400-combined.json) passes for all 393 profiles.
The table below reports only the ten newly checked profiles, not a fresh audit
of every existing profile.

| Cohort | Profiles | Emitted RPC checks | Emitted zeros | Initialized holder observations | Carry-forward matches | Final holders | Final zeros |
|---|---:|---:|---:|---:|---:|---:|---:|
| GSC, WDATAIP, VKY, MOS, IFP, SPX, RADR | 7 | 163 | 15 | 239 | 76 | 91 | 26 |
| OM | 1 | 22 | 0 | 33 | 11 | 13 | 1 |
| GALAXA, LK | 2 | 50 | 0 | 65 | 15 | 24 | 1 |
| **New profiles total** | **10** | **235** | **15** | **337** | **102** | **128** | **28** |

All completed cohort checks have **zero value mismatches**. The holder
experiments initialize 128 observed `(token, holder)` pairs at the parent block
using **256 independent storage/RPC reads**, then replay actual packaged output
and check all 128 final balances against fresh RPC. Reference observations do
not seed or repair retained state. See the
[seven-token qualification](evidence/seven400-final-qualified.json),
[OM evidence](om400-coverage.md), and
[GALAXA/LK summary](evidence/packed400-summary.json).

## What stopped the earlier runs

The rejected blocks exposed unreviewed metadata. The mapper refused to emit
potentially incomplete results; these were not silently accepted wrong amounts.
The explicit fixture layouts now include separately reviewed allowance and
other ordinary metadata fields, with these additional corrections:

- **MOS slot 11:** an address in the low 160 bits, a processing guard in byte 20 and
  `swapAndLiquifyEnabled` in byte 21. The original seven-token attempt still
  records its three rejected blocks. The final review re-executed both
  `balanceOf` and the packed-field getters using exactly the displayed override
  payloads before adding this slot.
- **GALAXA slots 32/33:** a per-block branch resets slot 32 and writes the current
  block number to slot 33. Both scalar getters were checked independently.
  The original eleven rejected blocks remain in the
  [review evidence](evidence/packed400-reviewed.json).
- **LK slot 29:** an address in the low 160 bits shares a word with the processing lock byte
  at bits 160–167. Historical successful transactions show that byte being set
  and cleared while preserving the address. Four original rejected blocks and
  the [transaction evidence](evidence/packed400-transactions.json) are retained.

Nine profiles have reviewed direct `balanceOf` paths: one address-mapping read
and an unmodified uint256 return. OM uses a pinned canonical 45-byte ERC-1167
forwarder and a separately pinned implementation with the same direct getter
behavior. The reviews bind historical runtime hashes and include independent
raw-word and metadata-isolation RPC controls. OM's first packaged audit ended
on a transport error; the successful run rechecked the complete preserved
capture and retains that failure's provenance.

Fifteen new Rust regressions retain ten captured transaction fixtures with
40 independent RPC expectations, 200 raw-word controls, 88 metadata controls,
original rejection cases and unknown mapping rejection. GALAXA/LK additionally check
unknown scalar rejection; OM checks implementation-code changes even without
holder writes. Unreviewed writes still fail closed. OM long-string storage
outside the reviewed fixed name/symbol words remains unsupported. Production
continues to use the single RPC-free `map_events` and shared Events protobuf.

All **289 workspace Rust library/binary tests** pass, together with Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.

## Combined packaged replay

The fresh capture uses
[`bsc-tail400-layouts.json`](../tests/fixtures/bsc-tail400-layouts.json) and the
unchanged production package. Every protobuf event field matches for
**109,470 previously RPC-verified emitted balances**, including **15,182 zeros**.
There are **392 emitting profiles**; hLBP retains separate older mint evidence.
Identical event histories preserve **179,362 initialized observations**,
including **69,893 without a new event**.

This comparison binds the prior 383-profile capture and the three newly audited
cohorts to fresh canonical headers. It reuses immutable RPC evidence; it does
not establish a new full-393 holder checkpoint or global enumeration. The
[seven-token summary](evidence/seven400-summary.json) also lists the seven
remaining top-400 candidates and links the original failed qualification.

Production artifact SHA-256 values remain unchanged:

- SPKG: `f1d57bdff549947cd69e47d117ad8cd74933c1e065b687f958fea4de39004d81`
- WASM: `36f5c502ec6546fc842ffaf8cfc692140d10e7f8cb58ff731c259a1b6d48c117`

## Holder and qualification limits

Emitted-value parity and initialized-holder parity are separate results.
Without the bounded parent checkpoint, these new cohorts have **100 unknown
reference rows, including 40 nonzero balances**. An unchanged or missing storage
write does not establish a holder's balance. Null-address mapping controls are
checked internally, while public events retain the canonical null-address
filter. These results do not enumerate all holders or establish identical cold
stream row coverage.

The layouts apply to the pinned runtimes and tested interval. They do not claim
verified source, every future metadata path, deployment coverage, arbitrary
proxy-family support or qualification on other EVM networks.

Seven contracts from the original top400 ranking remain unqualified:

| Rank | Token | Contract |
|---:|---|---|
| 143 | LBP | `0x88886f0fd371dff856291badced45922bc888888` |
| 203 | TITAN | `0xe0ab66985c8d18fba4cbc080024341750e997777` |
| 209 | ORD | `0x885584e6ee2f8bdcdd7be5898da49f63050e8899` |
| 238 | YBC | `0xebc2d768147f2d058f4266bb57e34ca1b6ef1319` |
| 338 | 钻石 | `0x26bfefbad1bc1f6979ad92e544171f0b600c8888` |
| 372 | BabyDoge | `0xc748673057861a797275cd8a068abb95a902e8de` |
| 377 | 10SET | `0x1f64fdad335ed784898effb5ce22d54d8f432523` |

The [ranks 401–450 mismatch diagnosis](next450-mismatch-diagnosis.md)
separately investigates APM and OG. Their plain mapping hypotheses are rejected;
that diagnostic work adds no qualified profiles to the counts above.
