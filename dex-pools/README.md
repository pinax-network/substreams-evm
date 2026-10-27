# EVM pool metadata decoding

`dex-pools` exposes the existing pool collectors as a normal Rust library. The
Substreams wrapper uses it for both legacy and Hosted Store outputs. Native
consumers can call the same collectors without a store or RPC service.

This is a compatibility extraction, not a new pool-admission policy. It preserves
the legacy v0.8.0 store's metadata, event precedence and observation order. It also preserves the
existing Curve deployment heuristics described below.

## API

| Function | Input | Result |
| --- | --- | --- |
| `project(&Block)` | Ethereum 0.11 owned block | Ordered `Vec<PoolEntry>` |
| `decode_log(&Log, create_address)` | Ethereum 0.11 owned log and optional Curve context | First matching `Option<PoolEntry>` |
| `decode_log_bytes(address, topics, data, create_address)` | Borrowed bytes from any model | Same result as `decode_log` |
| `decode_curvefi_init(&TransactionTrace)` | Ethereum 0.11 transaction | Existing direct Curve constructor-tail result |

`PoolEntry` has public `address: Vec<u8>`, `tokens: Vec<Vec<u8>>`, and
`factory: Vec<u8>` fields. It derives `Debug`, `Clone`, `PartialEq`, and `Eq`.
`Block`, `Log`, `TransactionTrace`, and the `ethereum` model module are reexported
for callers using that input model. The library returns ordinary Rust values and
does not depend on the workspace's output protobuf crate.

`address` preserves the existing key representation: normally a 20-byte contract
address, but a 32-byte pool ID for Uniswap V4. Empty token lists, zero/native
currency representations and absent factories are preserved. Callers must not
assume that every key is an account address or every token list is complete.

## Using another input model

Parquet or Streams consumers with a different protobuf model can borrow its
address, topics and data and call `decode_log_bytes`. For example, given an
existing decoded log whose fields expose byte slices:

```rust,ignore
let topics: Vec<&[u8]> = log.topics.iter().map(|topic| topic.as_ref()).collect();
let candidate = dex_pools::decode_log_bytes(
    log.address.as_ref(),
    &topics,
    log.data.as_ref(),
    create_address,
);
```

This does not encode or decode the enclosing block. ABI 1.5 requires an owned
Ethereum 0.11 `Log`, so the adapter copies this one log's emitter, topics and data.
It defaults unused log index/ordinal fields to zero, which only changes the index
in an ABI failure diagnostic. It is not a zero-copy ABI decoder.

Raw decoding does not establish execution success, factory authenticity,
transaction provenance or first-write persistence. A caller supplies committed
logs and any Curve creation-address context. The transaction helper for direct
Curve deployments remains tied to the Ethereum 0.11 input model; it does not
require output protobuf types or a whole-block conversion.

## Dependency boundary

The crate deliberately uses the workspace's existing `substreams-abis` **1.5** and
`substreams-ethereum` **0.11** dependencies. These transitively include Substreams
**0.7**, so this is a native-callable library, not an SDK-free library. No optional
feature removes those dependencies. The library defines no handlers, stores or RPC
calls. Existing `Event::match_and_decode` error diagnostics retain SDK logging
when compiled for WASM; native SDK logging is a no-op.

Do not combine a second incompatible Substreams SDK into one binary without
checking linkage: the SDK exports allocator symbols. ABI 2.x and Ethereum 0.12
support borrowed `LogLike`, but require a separate compatibility decision. This
refactor does not migrate existing modules or protobuf versions.

## Preserved projection behavior

- Visit successful transactions in the block's supplied order.
- Consider the direct Curve constructor result before logs in each transaction.
- With no calls, use receipt logs in supplied order. With calls, use the SDK's
  non-reverted call logs sorted by ordinal and ignore the receipt copy.
- Use the transaction's first CREATE address as Curve factory context, as before.
- Return duplicate keys in observation order. The caller applies first-write
  storage; the library does not deduplicate or reject conflicting candidates.
- Filter empty keys in `project`, matching the old store's write guard. The raw
  decoding helpers preserve collector results, including an empty supplied key.

Receipt-only successful transactions must contain a receipt, as required by the
existing SDK helper. Curve's factory context is not associated to the emitting
call and can select an unrelated or reverted CREATE. Its direct deployment
heuristic decodes the final eight ABI words without authenticating Curve bytecode.
These are inherited compatibility limits, not newly strengthened evidence rules.

## Validation

Run `cargo test -p dex-pools` for borrowed-byte/owned-model parity and observation
semantics. Wrapper tests cover all collectors against a frozen pre-refactor
baseline. The eight protocol collector files are moved unchanged from the legacy
wrapper, so ABI interpretation remains in the existing shared dependency.
