# Uniswap V2 events

`map_events` emits the existing transaction-level `uniswap.v2.Events` output.

Complete-block pool-state extraction is maintained in
[`substreams-evm-extended/dex/pool-state`](https://github.com/pinax-network/substreams-evm-extended/tree/main/dex/pool-state).
Its `map_events` module requires Extended blocks and emits the combined
`dex.pool_state.v1.BlockPoolState` envelope for both V2 and V3.
The previous `map_pool_closes` module is no longer built here. Previously pinned
packages remain historical artifacts; consumers migrating to the new package must
select its module, verify Extended input availability and pin its new digest.

Shared contract ABI types remain in
[`substreams-abis`](https://github.com/pinax-network/substreams-abis).
