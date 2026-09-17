# BSC top 300: 17 more qualified layouts and captured holder-list activity

The explicit test configuration now contains **296 qualified profiles** from
the original top 300 RPC-stream candidates. All 50 candidates from ranks
251–300 are qualified within the tested window. LBP (143), TITAN (203), ORD
(209) and YBC (238) remain excluded because their calculated balances need
additional treatment. This is bounded runtime qualification, not universal BSC
support or complete holder enumeration.

Production remains one RPC-free `map_events`, the shared
`evm.balances.v1.Events` protobuf and default parameters `[]`. This follow-up
adds test configuration, captured Rust regressions and evidence; it does not
change the production mapper or introduce a store.

## Reviewed layouts

The [qualification evidence](evidence/pending300-qualification.json) and
[candidate summary](evidence/pending300-summary.json) cover these additions:

| Getter/storage family | Candidates |
| --- | --- |
| Source-bound namespaced ERC-20 proxies | STABLE, UUSD, SOON, GAIB, SENT, ENSO |
| Previously reviewed proxy/beacon families, with fresh dependency checks | XAN, GRVT, SOXSB, XLM |
| Source-bound direct/public mappings | WKC, DIA, ETZ |
| Historical bytecode-reviewed direct mappings | CATLIFE, 屎壳郎, AGG, CDXR |

Every layout has a complete 1,024-block native scan without unresolved writes.
The four bytecode-only candidates have reviewed selector dispatch, address
decoding, unconditional balance mapping reads and returns. Zero, one, 123 and
maximal-word controls match; caller/holder controls and changes to the explicit
non-balance fields do not alter the returned balance. WKC additionally has
verified source bound to its historical runtime; its older compiler does not
provide a storage-layout artifact, so named getter traces establish roots.

Proxy qualification binds implementation and beacon dependencies at both
canonical boundaries. Namespaced profiles configure only the reviewed ERC-20
balance, allowance, supply, name and symbol fields. Other administration,
bridge or permit writes remain strict unknowns. A shared shell does not qualify
a dependency, and a future protected change still rejects processing.

DIA and ETZ use source-reviewed address lists. The main window has no list
membership changes; the separate older captures below exercise those paths.
An initial list diagnostic rejected differently cased source-address strings.
The corrected check compares decoded 20-byte identities and retains chain and
runtime binding. The failed report is preserved in the qualification evidence.

## Packaged output and initialized holders

The [17-token packaged-WASM audit](evidence/pending300-rpc.json) covers the
original **122288006–122289029** interval and matches all **528 emitted
balances**, including **44 zeros**, against canonical historical RPC.

The [actual-WASM holder replay](evidence/pending300-wasm-holders.json) starts
from **363 observed holders**, initialized with **726 balance/storage reads**:

- All **869 reference observations** match, including **341** without a new event.
- All **363 final balances** match a fresh RPC snapshot; **122 are zero**.
- Processing makes no balance RPC calls, and reference values never repair state.
- Cold state has **335 unknown observations**, including **134 nonzero**.
  These remain unknown rather than being treated as zero.

The [fresh combined capture](evidence/pending300-combined.json) uses
[bsc-pending300-layouts.json](../tests/fixtures/bsc-pending300-layouts.json).
It preserves every protobuf event field for **107,277 previously RPC-verified
emitted balances**, including **14,915 zeros**, with fresh canonical-header
and runtime checks. **295 profiles emit**; hLBP is quiet in this interval and
has separately documented older activity. The comparison reuses immutable RPC
evidence, makes no new balance RPC calls and does not create a full-296 holder
checkpoint or global snapshot. Identical event histories preserve evidence for
**175,769 initialized observations** and **68,493 carry-forward matches**.

Seventeen captured transaction fixtures preserve **45 independently checked
balances** for Rust regressions. The unchanged production SPKG digest is
`f13ce00a08e3610ed607a168c1f2afac6fb92ac63561f945e97a55d7b4801503`.

Validation passes **247 workspace Rust library/binary tests**, Clippy with
warnings denied, workspace WASM compilation, scoped formatting and diff checks.

## Captured append and swap-and-pop

These older one-block experiments have separate hashes, initialization and
RPC snapshots; their counts are not added to the original-window totals.

| Activity | Block | Emitted balances | Initialized/final holders | Carry-forward matches |
| --- | ---: | ---: | ---: | ---: |
| DIA appends member 228; length 228 → 229 | 122037457 | 2 | 230 | 228 |
| ETZ moves member 76 into slot 64, clears slot 76; length 77 → 76 | 122279966 | 2 | 78 | 76 |

Both packaged audits match RPC: [DIA](evidence/dia-append-rpc.json) and
[ETZ](evidence/etz-swap-rpc.json). The holder experiments initialize the parent
array members plus emitted addresses and compare every retained final balance
with RPC: [DIA holders](evidence/dia-append-holders.json) and
[ETZ holders](evidence/etz-swap-holders.json). Neither array is represented as
the full token-holder universe. Initialization/final balance reads are explicit;
processing itself makes none.

ETZ supplies the previously missing **real swap-and-pop capture**. Rust tests
bind the block, transaction, storage keys, old/new words and ordinals, compare
all emitted balances against RPC, and reject corrupted or missing witnesses.
DIA has equivalent captured append checks. Removing each list rule causes the
strict mapper to reject its captured block. No production logic change was needed.

The first ETZ holder diagnostic incorrectly expected empty-output clock data
for a one-block capture that contained output. Its failure is preserved. The
corrected run checks complete event delivery, the audited row set, canonical
block identity and fresh holder snapshots; it does not infer missing output.

## Remaining work

The four computed-balance candidates remain described in
[LBP](lbp-reward-model.md), [TITAN/ORD](external-getter-contexts.md) and
[YBC](ybc-reward-trace.md). Matching raw words or a few reward paths does not
establish their complete getter semantics or affected-holder emission rules.

The requested next network sequence is [Ethereum, Base, HyperEVM and
Arc](network-expansion.md), with separate chain, runtime, complete Extended-block,
token and holder evidence for each. Preliminary RPC probes on those networks
do not yet establish ERC-20 parity.
