# TITAN and ORD: wider sampling and account-aware traces

TITAN (203) and ORD (209) remain unqualified. This follow-up expands historical
sampling and fixes a limitation in the Rust balance inspector: a storage read
previously showed only its call depth, instruction offset, key and value. Those
fields do not identify the account that owns the storage, especially through
`DELEGATECALL`. The inspector now also reports the executing code address,
storage address and call transitions in `execution_contexts`.

The [evidence summary](evidence/external250-context-summary.json) binds the
reports and captured Rust fixtures. The production package and **246-profile**
fixture are unchanged. This work does not add production reward support or
increase the combined holder-parity totals.

## Wider historical holder sampling

The [candidate scan](evidence/external250-holder-candidates.json) covers all
1,024 original Extended blocks, **122288006–122289029**, validates their chain
continuity and hashes, and uses persisted writes plus hash-verified preimages.
Candidate addresses are exploratory: an address-shaped mapping key need not
represent a holder, and these observations cannot enumerate all holders.

The [canonical RPC audit](evidence/external250-candidate-audit.json) checks each
candidate at the initial checkpoint and final block. External calls use the
token as caller, matching the context inside its balance getter.

| Token | Candidate addresses | Beyond original reference | Boundary observations | Raw-storage mismatches |
| --- | ---: | ---: | ---: | ---: |
| TITAN | 25 | 0 | 50 | 0 |
| ORD | 126 | 96 | 252 | 0 |

The newly added ORD addresses supply **192 additional boundary observations**.
The other observations repeat earlier samples. Raw balance plus the queried
external contributions matches these observations, but those contributions do
not change the sampled raw balances. This is evidence about the sampled states, not proof that
the getters always return raw storage. The earlier
[external return controls](beacon-admin-holder-coverage.md#titan-and-ord-matching-samples-conceal-external-dependencies)
still demonstrate that the dependencies can affect public balances.

## TITAN storage belongs to the external proxy

TITAN's observed path calls the proxy
`0x4dd028f2cddeb2e3da83efce41703844dfdb4b64` twice, which delegates to
`0x0541727a60316913f78eb7fd5524700acf89264c`. The trace has **29 storage reads**
and four call transitions. Code running at the delegate reads the proxy's
storage, not the delegate's storage or the token's storage.

The new [live inspector report](evidence/titan-context-attribution.json) reproduces
the captured contexts and retains the existing balance/word controls. The fixture
checks 16 unique storage words independently against canonical historical RPC.
The call sequence also agrees with an independent `callTracer` result, and every
read's instruction offset is an `SLOAD` in its attributed historical runtime.

## ORD activates additional dependencies

In the [participation probe](evidence/ord-participation-probe.json), ORD's external
getter first checks its configured token in storage slot 3 and the holder's word
in mapping 6. The sampled holder's zero word takes the early return. Read-only
overrides of that word to 1 or 17 activate a longer path: the dependency calls
back into ORD, which calls `0xca8bf23801341c3ba2e696d06e9679647f5b0f9b` to resolve
another address. A separate branch calls router
`0x10ed43c718714eb63d5aa57b78b54704e256024e`, which reads pool reserves at
`0x16b9a82891338f9ba80e2d6970fdda79d1eb0dae`.

The simulated path has **26 reads**, four call transitions and four distinct
accounts with storage reads. Independent checks bind 22 unique words: 21 come
from canonical RPC and one is the explicit simulation override. The router has
no storage reads in this path. The independent call tree and runtime instruction
checks corroborate the account attribution.

Both simulated nonzero participation words return a four-word tuple whose first
three values are zero and whose fourth is one. ORD's raw balance pinned to 123
therefore still returns 123. This is neither a historical mismatch nor a complete
model of the dependency. It demonstrates that qualification must review the
callback state and price dependencies reached beyond the early return. Mapping
6's full business meaning and reward arithmetic remain unqualified.

## Inspector behavior and regression scope

`inspect-balance` preserves the legacy `storage_reads` field and adds
`execution_contexts`, plus canonical runtime checks in `execution_code`.
`CALL` and `STATICCALL` switch storage to the target;
`DELEGATECALL` and `CALLCODE` retain the caller's storage while switching code.
Returning from a nested or sibling call restores the appropriate account.
Calls without child execution do not change context. Targets follow the EVM's
low-160-bit address interpretation.

Malformed depth transitions, missing stack/results, traces ending inside a child,
and entered creation frames without a resolvable address are errors. The inspector
does not invent an account for incomplete evidence. It reports only executed
paths and never promotes a token to a qualified layout.

Historical runtime checks require each attributed instruction to be an actual
opcode boundary, not a byte inside PUSH data. Unresolved EIP-7702 code designations
are rejected because they can redirect code without an explicit delegate frame.
The [fresh runtime validation](evidence/external250-runtime-contexts.json) binds
all 55 reads and eight call instructions to their historical runtimes, including
ORD's router, across both captured cases.

Six Rust tests cover nested proxy storage, sibling/unwind behavior, call targets,
incomplete traces and the two captured cases. Compact fixtures retain all call,
depth-transition and storage-read evidence; their attribution equals the full
traces, whose digests are retained. Tests reproduce all **55 attributed reads**
and compare them with the independently checked storage words.

All **241 workspace Rust library/binary tests** pass, along with Clippy with
warnings denied, workspace WASM compilation, formatting and diff checks. The production SPKG/WASM
hashes remain those recorded in the [246-profile audit](beacon-admin-holder-coverage.md).

## Source availability and remaining work

The [dependency inventory](evidence/external250-dependencies.json) pins historical
runtime hashes at both boundaries. Sourcify returned 404 for these five token,
proxy and dependency addresses; no source-qualified claim follows. Attempts to
retrieve the two dependency metadata files from the CIDs in their runtime CBOR
received HTTP 429 from the gateway. The
[metadata attempt report](evidence/external250-metadata.json) retains those
responses; they are not evidence that the metadata does not exist.

Production support still needs complete getter/dependency review and a durable
way to initialize and update every affected holder when external or global state
changes. LBP and YBC retain their separate reward-model work. No store has been
added. Broader BSC candidates and the requested
[Ethereum, Base, HyperEVM and Arc qualification](network-expansion.md) remain
part of the continuing work.
