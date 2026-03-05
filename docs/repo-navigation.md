# substreams-evm Repository Navigation

This guide captures practical repo knowledge for fast orientation and implementation.

## Workspace layout

- `Cargo.toml`: Rust workspace root and crate membership.
- `proto/`: protobuf types used across packages.
- `common/`: shared Rust helpers used by multiple modules.
- `dex/`: protocol-specific DEX extractors (Uniswap, Balancer, Curve, etc.).
- `erc20/`, `native/`, `erc1155/`, `dex-nfts/seaport/`: domain event modules.
- `evm-*` aggregators: database-oriented `db_out` pipelines that compose lower-level modules.
- `blocks/clickhouse/`: block-level dataset package.
- `spkg/`: built and vendored `.spkg` artifacts consumed by aggregator manifests.

## Mental model (data flow)

1. Protocol/token modules emit normalized events.
2. Aggregator crates (for example `evm-dex`, `evm-transfers`, `evm-nfts`) compose those outputs.
3. `db_out` map modules emit `sf.substreams.sink.database.v1.DatabaseChanges`.
4. Engine-specific manifests in `*/clickhouse/` and `*/postgres/` adapt sink execution.

## Where to edit for common tasks

- Add/adjust event decoding for one protocol: `dex/<protocol>/src/lib.rs` and that module's `substreams.yaml`.
- Change shared protobuf fields: `proto/v1/*.proto` then regenerate/build affected modules.
- Update DB schema mapping: `<aggregator>/clickhouse/schema.*.sql` or `<aggregator>/postgres/schema.*.sql` plus aggregator Rust mapping code.
- Adjust package wiring/import versions: target package `substreams.yaml` (imports/modules/params/network).
- Tune execution defaults for local development: module `Makefile` (`ENDPOINT`, `START_BLOCK`, `PARALLEL_JOBS`).

## Fast command patterns

- Build workspace wasm artifacts:
  - `cargo build --target wasm32-unknown-unknown --release`
- Build one package and run noop sink locally:
  - `make -C evm-dex noop`
  - `make -C evm-nfts noop`
- Package a specific manifest:
  - `make -C evm-transfers pack`
- Inspect package metadata quickly:
  - `substreams info evm-dex/substreams.yaml`

## File hotspots

- `README.md`: top-level module inventory and protocol coverage.
- `scripts/pricing-calculator.py`: rough cost estimation utility.
- `scripts/compare-metadata.sh`: metadata comparison utility.
- `evm-nfts/substreams.yaml`: good example of composing multiple imported SPKG/native modules.
- `evm-dex/substreams.yaml`: broad DEX aggregation example.

## Notes on artifact folders

- `erc20-balances/`, `erc20-supply/`, `erc20-tokens/`, `erc20-transfers/`, `native-balances/` are artifact-oriented directories containing prebuilt `.spkg` files and replay logs.
- Active source crates for these domains live under `erc20/*` and `native/*`.
