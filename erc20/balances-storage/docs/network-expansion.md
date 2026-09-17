# Network qualification sequence

After the BSC candidate review, repeat the same qualification on **Ethereum,
Base, HyperEVM and Arc**, in that order. These networks are requested work;
they are not yet qualified by the BSC evidence.

[Preparatory RPC probes](evidence/network-rpc-prerequisites.json) now pass on all
four requested networks: live chain identity matches the configured registry,
the finalized head is available with a matching canonical parent, and canonical
block-hash code/storage/call requests succeed. These probes use the empty
address in a recent finalized block. They do not establish historical ERC-20
correctness, Extended-block/preimage availability or token/holder coverage.

The production interface stays one RPC-free `map_events` with shared
`evm.balances.v1.Events`, explicit verified layouts and default parameters `[]`.
All executable validation tooling and regressions stay in Rust.

For each network:

1. Verify the RPC chain identity, finalized-range support, canonical historical
   balance/storage reads, and availability of complete Extended blocks with
   storage writes and Keccak preimages. Record unsupported provider capabilities.
2. Capture the RPC reference stream and rank its active contracts. Keep the
   immutable capture, range, block hashes and ranking separate from other chains.
3. Qualify each token's getter and dependencies against its historical runtime.
   A shared address, symbol or implementation family is not a cross-chain proof.
4. Audit the actual packaged WASM against historical RPC, then replay retained
   holder state across consecutive blocks. Count initialized parity, cold-start
   unknowns, unchanged holders and any mismatches separately.
5. Add captured Rust regressions for new storage/getter behavior, and repeat the
   affected network's audits after each correction. Preserve incomplete runs.

Before those runs, replace the Rust tools' BSC-specific chain-ID and source
checks with an explicit network selection shared by ranking, inspection,
capture, audit and holder replay. Preserve wrong-chain rejection, canonical
block binding and finality checks. A provider that lacks a required feature
must not silently receive weaker validation.

The BSC campaign now includes 246 qualified profiles: the original top 100 plus
49 from ranks 101–150, all 50 from ranks 151–200 and 47 from ranks 201–250. The
[latest candidate follow-up](beacon-admin-holder-coverage.md) leaves three candidates
from ranks 201–250 and LBP (143) outside the qualified fixture. hLBP has a captured
mint in a separate older interval; 245 profiles emit in the original 1,024-block
interval. The [removal correction](shareholder-removal-coverage.md) passes a captured
shareholder tail pop; the latest combined capture preserves all 246 profiles'
audited output. Swap-and-pop has synthetic coverage but no captured swap
transaction yet. This is a bounded campaign. Completing it
does not establish support for every BSC token or complete global holder
enumeration. Broader EVM coverage will similarly be reported per network,
qualified runtime and tested interval, with checkpoints or full-history replay
required for holders whose prior balances cannot be derived from the window.

The remaining [LBP reward-state diagnostic](lbp-reward-model.md) now matches
594 sampled RPC observations and 34 simulated-state cases. It does not add
production LBP support: durable holder and reward state still requires a design
decision. The other networks remain at preliminary RPC prerequisite probes.

[hLBP's quiet-holder audit](hlbp-quiet-holder-coverage.md) confirms 88 initialized
observations and four final balances. Its new mint audit checks one emitted
balance, four observations and four final holders in a distinct older 64-block
interval. The two intervals remain separately measured. Empty output alone
cannot establish a holder's balance.
