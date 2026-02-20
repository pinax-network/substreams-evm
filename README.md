# EVM `Substreams`

> **Extended Blocks**: Ethereum, Base, BSC, Polygon, ArbitrumOne, Optimism, Avalanche, Tron EVM

## Directory Structure

```
evm-dex/          # DEX aggregator (db_out) + clickhouse/ + postgres/
evm-transfers/    # Transfer aggregator (db_out) + clickhouse/ + postgres/
evm-balances/     # Balance aggregator (db_out) + clickhouse/ + postgres/
evm-supply/       # Supply aggregator (db_out) + clickhouse/ + postgres/
blocks/           # Block metadata + clickhouse/
dex/              # Individual DEX map_events modules
erc20/            # ERC-20 modules (transfers/ balances/ supply/ tokens/)
native/           # Native transfer/balance modules (transfers/ balances/)
erc1155/          # ERC-1155 multi-token events
proto/            # Protobuf definitions
common/           # Shared Rust helpers
spkg/             # Pre-built Substreams packages
```

## DEX Modules (`/dex`)

### Uniswap Family
| Module | Protocol | Events |
|--------|----------|--------|
| `uniswap-v1` | Uniswap V1, JustSwap V1 | `TokenPurchase`, `EthPurchase` |
| `uniswap-v2` | Uniswap V2, SunSwap V2 + forks | `Swap`, `Sync`, `Mint`, `Burn` |
| `uniswap-v3` | Uniswap V3 + forks | `Swap`, `Mint`, `Burn`, `Flash` |
| `uniswap-v4` | Uniswap V4 | `Swap`, `ModifyLiquidity`, `Initialize` |

### Standalone DEX Protocols
| Module | Protocol | Events |
|--------|----------|--------|
| `aerodrome` | Aerodrome / Velodrome | `Swap`, `Fees`, `Mint`, `Burn` |
| `balancer` | Balancer V2/V3 | `Swap` (WeightedPool, StablePool, Vault) |
| `bancor` | Bancor V3 | `TokensTraded` (StandardPoolConverter) |
| `cow` | CoW Protocol | `Trade` (GPv2Settlement) |
| `curvefi` | Curve Finance | `TokenExchange`, `TokenExchangeUnderlying` |
| `dca-dot-fun` | DCA.fun | `FillOrder`, `CreateOrder`, `CancelOrder` |
| `dodo` | DODO | `DODOSwap` |
| `kyber-elastic` | KyberSwap Elastic | `Swap` (Pool) |
| `sunpump` | SunPump | `Traded` |
| `traderjoe` | Trader Joe V2 (Liquidity Book) | `Swap` (packed bytes32 amounts) |
| `woofi` | WOOFi | `WooSwap` (WooPPV2) |

### EVM DEX Factory Addresses

<details>
<summary>ETH Mainnet</summary>

| DEX | Protocol | Factory Address |
|-----|----------|----------------|
| Uniswap V2 | uniswap_v2 | `0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f` |
| Uniswap V3 | uniswap_v3 | `0x1f98431c8ad98523631ae4a59f267346ea31f984` |
| Uniswap V4 | uniswap_v4 | `0x000000000004444c5dc75cb358380d2e3de08a90` |
| SushiSwap V2 | uniswap_v2 | `0xc0aee478e3658e2610c5f7a4a2e1777ce9e4f2ac` |
| SushiSwap V3 | uniswap_v3 | `0xbaceb8ec6b9355dfc0269c18bac9d6e2bdc29c4f` |
| PancakeSwap V2 | uniswap_v2 | `0x1097053fd2ea711dad45caccc45eff7548fcb362` |
| Balancer V2 | balancer | `0xba12222222228d8ba445958a75a0704d566bf2c8` |
| Curve Finance | curvefi | Various pool contracts |
| Bancor V3 | bancor | `0xeef417e1d5cc832e619ae18d2f140de2999dd4fb` |
| CoW Protocol | cow | `0x9008d19f58aabd9ed0d60971565aa8510560ab41` |
| DODO | dodo | Various pool contracts |
| WOOFi | woofi | Various WooPPV2 contracts |
| Trader Joe V2 | traderjoe | `0x8e42f2f4101563bf679975178e880fd87d3efd4e` |
| KyberSwap Elastic | kyber_elastic | Various pool contracts |

</details>

<details>
<summary>Arbitrum One</summary>

| DEX | Protocol | Factory Address |
|-----|----------|----------------|
| Uniswap V3 | uniswap_v3 | `0x1f98431c8ad98523631ae4a59f267346ea31f984` |
| Uniswap V4 | uniswap_v4 | `0x360e68faccca8ca495c1b759fd9eee466db9fb32` |
| SushiSwap V3 | uniswap_v3 | `0x1af415a1eba07a4986a52b6f2e7de7003d82231e` |
| Camelot | uniswap_v2 | `0x6eccab422d763ac031210895c81787e87b43a652` |

</details>

<details>
<summary>Base</summary>

| DEX | Protocol | Factory Address |
|-----|----------|----------------|
| Uniswap V3 | uniswap_v3 | `0x33128a8fC17869897dcE68Ed026d694621f6FDfD` |
| Uniswap V4 | uniswap_v4 | `0x7c5f5a4bbd8fd63184577525326123b519429bdc` |
| Aerodrome | aerodrome | `0x420dd381b31aef6683db6b902084cb0ffece40da` |

</details>

<details>
<summary>TRON (TVM)</summary>

| DEX | Protocol | Factory Address |
|-----|----------|----------------|
| SunSwap V1 (JustSwap) | justswap | `TXk8rQSAvPvBBNtqSoY6nCfsXWCSSpTVQF` |
| SunSwap V2 | sunswap | `TKWJdrQkqHisa1X8HUdHEfREvTzw4pMAaY` |
| SunPump | sunpump | `TTfvyrAz86hbZk5iDpKD78pqLGgi8C7AAw` |

</details>

## Token Modules

### ERC-20 (`/erc20`)
| Module | Description |
|--------|-------------|
| `transfers` | `Transfer` and `Approval` events |
| `tokens` | Protocol-specific events: WETH, USDC, USDT, WBTC, SAI, stETH |
| `balances` | Token balances via batched RPC `balanceOf` calls |
| `supply` | Token supply tracking |

### Native (`/native`)
| Module | Description |
|--------|-------------|
| `transfers` | Block rewards, tx transfers, call transfers, validator withdrawals, self-destructs, genesis balances, DAO fork |
| `balances` | Native ETH balance extraction |

### ERC-1155 (`/erc1155`)
- `TransferSingle`, `TransferBatch`, `ApprovalForAll`, `URI`

## Aggregator Packages

Each aggregator combines individual modules into a single `db_out` for database sinks:

| Package | Description | Sinks |
|---------|-------------|-------|
| `evm-dex` | All DEX swap events | Clickhouse, Postgres |
| `evm-transfers` | ERC-20 + native transfers | Clickhouse, Postgres |
| `evm-balances` | ERC-20 + native balances | Clickhouse, Postgres |
| `evm-supply` | ERC-20 supply | Clickhouse, Postgres |
| `blocks` | Block metadata | Clickhouse |

## Event Signatures

| Event | `topic0` |
|-------|----------|
| Transfer | `ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef` |
| Deposit | `e1fffcc4923d04b559f4d29a8bfc6cda04eb5b0d3c460751c2402c5c5cc9109c` |
| Withdrawal | `7fcf532c15f0a6db0bd6d0e038bea71d30d808c7d98cb3bf7268a95bf5081b65` |

## Schema Concepts

### Call Metadata (EXTENDED Detail Level)

All proto schemas include optional `Call` metadata:

```protobuf
message Call {
  bytes caller = 1;
  uint32 index = 2;
  uint32 depth = 3;
  CallType call_type = 4;
}
```

Available on chains with EXTENDED detail level: Ethereum, Base, BSC, Polygon, ArbitrumOne, Optimism, Avalanche, TRON.

### Transaction Structure

```protobuf
message Events { repeated Transaction transactions = 1; }

message Transaction {
  bytes hash = 1;
  bytes from = 2;
  optional bytes to = 3;
  uint64 nonce = 5;
  string gas_price = 6;
  uint64 gas_limit = 7;
  uint64 gas_used = 8;
  string value = 9;
  repeated Log logs = 10;
}
```

## Links

- [Substreams Documentation](https://substreams.streamingfast.io)
- [Substreams ABIs](https://github.com/pinax-network/substreams-abis)
