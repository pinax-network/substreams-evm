# Proto schema notes

## Shared/common `Transaction` / `Log` / `Call` evaluation

This repository now keeps the active DEX and ERC-20 transfer/token protobufs aligned on the same rich `Transaction`, `Log`, `Call`, and `CallType` shapes.

Before deduplicating those messages into a shared/common proto, we prototyped that approach against the current Substreams packaging and committed-Rust-binding workflow. The concrete outcome is:

- `Log` cannot be moved into a shared/common proto without changing the existing shape because every protocol owns a different `oneof log { ... }` payload.
- `Transaction` therefore cannot be shared either, because it contains `repeated Log logs`.
- `Call` and `CallType` are theoretically shareable, but switching the current package-local definitions over to imports would create broad generated-module namespace churn across sinks and helper code for limited immediate gain.

Because that shared/common move is not currently low-friction, the repository keeps the duplicated in-scope definitions aligned instead of introducing a partially shared model.
