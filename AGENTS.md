# AGENTS guidance for `substreams-evm`

For repository orientation and task routing, read:

- `docs/repo-navigation.md`

Use that guide as the default map for:

- locating protocol modules (`dex/`, `erc20/`, `native/`, `erc1155/`, `dex-nfts/`)
- locating aggregator modules (`evm-*`)
- finding sink/schema files (`*/clickhouse/`, `*/postgres/`)
- choosing build/run entrypoints (`Makefile`, `substreams.yaml`, workspace `Cargo.toml`)

Complete-block V2/V3 pool-state extraction is maintained in
[`substreams-evm-extended`](https://github.com/pinax-network/substreams-evm-extended/tree/main/dex/pool-state).
Do not reintroduce its old `dex-pool-state`, `map_pool_closes` or
`map_pool_changes` modules here. Shared ABI structs and generated bindings belong
in [`substreams-abis`](https://github.com/pinax-network/substreams-abis).
