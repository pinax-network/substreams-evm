# Expanding toward all ERC-20 balances

The Rust `probe-erc20` command now runs discovery directly over captured Extended
Block protobuf files. It is excluded from WASM and creates no Substreams map or
cache. Hypotheses never enter `map_events` unless separately qualified and
supplied as an explicit token-layout configuration.

The [expanded BSC qualification](holder-coverage.md) adds reviewed USDT and BTCB
layouts, a USDC configuration with pinned proxy implementation guards, and native
holder-state tests with explicit checkpoints. The top-50 survey is candidate
evidence only; it does not automatically qualify the remaining layouts.

## Historical multi-map experiment

Over BSC blocks 122260950–122260965, the module observed **239 contracts** emitting
ERC-20-shaped Transfer logs and **522 candidate mapping layouts**. For **215
contracts**, at least one mapping had matching before/after `balanceOf` values
throughout the observed writes, including at least two distinct holders with
nonzero balances and two changed observations. The remaining 24 contracts did
not meet that evidence gate. See [the recorded results](evidence/erc20-discovery-16.json).

These counts are not chain-wide coverage. Some contracts have multiple matching
layouts, and even a unique match can be a mirror, a conditional code path or a
short-window coincidence. No additional adapter has been promoted automatically.
The contract set is derived from log shape, not a verified token registry.

The native extractor reads persisted storage writes and verified Keccak preimages across
all observed token-like contracts. It emits the holder, full 256-bit mapping
base, actual storage key, first old value and final new value. This handles
candidate bases outside a small numeric slot range. Scalar writes, missing
preimages and other layouts are counted as unclassified instead of guessed.
Code changes are flagged, reverted logs do not enter the inventory, and malformed
preimages or discontinuous writes fail the extraction.

The probe evaluates each hypothesis using EIP-1898 block hashes at the parent and
current block. Allowance or unrelated mappings normally disagree and are retained
as rejected hypotheses; zeros alone cannot pass the evidence gate. Contract/RPC
errors remain unresolved. Full before/after check records are retained locally,
with their digest in the summary. The extractor is an ordinary Rust function, not a map module.

```sh
cargo run --locked -p erc20-balances-storage-tools -- probe-erc20 \
  --block-file erc20/balances-storage/tests/fixtures/bsc-122260950.pb \
  --output erc20/balances-storage/out/my-erc20-probe
```

## What makes broader support correct

[ERC-20](https://eips.ethereum.org/EIPS/eip-20) standardizes the interface, including
`balanceOf`, rather than a storage layout. Solidity [storage layout](https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html)
depends on declarations, inheritance and custom bases. A mapping-shaped write
does not prove which value a contract returns.

1. Promote direct-mapping adapters only after verifying runtime identity and
   `balanceOf` semantics, resolving multiple candidate mappings, and testing
   independent windows, zero balances, mint/burn and non-Transfer mutations.
   Store contract/code identity, mapping slot and evidence in a versioned registry.
2. The explicit proxy configuration now pins implementation identity and rejects
   implementation-slot writes or code changes during continuous processing.
   New proxy types, implementation versions and layouts still need separate
   review and qualification; arbitrary proxies are not supported automatically.
3. Handle computed/rebasing/reflection balances with contract-specific rules or
   local EVM execution over complete reconstructed state. A storage word may be
   shares rather than the externally visible token balance. Shared-state changes
   may affect many holders without a per-holder storage write or Transfer event.
4. Bootstrap a complete holder/state checkpoint, verify continuity and account
   lifecycle, and invalidate dependencies correctly. Missing code, storage,
   external calls or holder coverage must remain unknown rather than zero.

The next adapter qualification candidates should come from the probe's strongest
and most frequently observed matches. Universal ERC-20 support remains unfinished;
the current public event output supports changed holders of explicitly configured,
qualified direct-mapping layouts.
