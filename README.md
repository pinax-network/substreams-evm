# EVM `Substreams`

> **Extended Blocks**: Ethereum, Base, BSC, Polygon, ArbitrumOne, Optimism
> Avalanche, Tron EVM

## Substreams Packages

### `/transfers` - Token & Native Transfers

#### ERC-20 Transfers (`erc20-transfers`)
- [x] `Transfer` event
- [x] `Approval` event

#### ERC-20 Tokens (`erc20-tokens`)
Token-specific events for popular ERC-20 tokens:
- [x] **WETH**: `Deposit`, `Withdrawal`
- [x] **USDC**: `Mint`, `Burn`, `Blacklisted`, `UnBlacklisted`, and more
- [x] **USDT**: `Issue`, `Redeem`, `Deprecate`, `Params`, `DestroyedBlackFunds`, and more
- [x] **WBTC**: `Mint`, `Burn`, `MintFinished`, `OwnershipRenounced`
- [x] **SAI**: `Mint`, `Burn`, `LogSetAuthority`, `LogSetOwner`
- [x] **stETH**: Comprehensive Lido staking events (see `/steth` module)

#### Native Transfers (`native-transfers`)
- [x] Block Rewards
- [x] Transaction Transfers
- [x] Call Transfers
- [x] Validator Withdrawals (post-Shanghai)
- [x] Contract Self-Destructs
- [x] Genesis Balances
- [x] DAO Hard Fork Transfers

### `/steth` - Lido stETH Protocol

Comprehensive stETH (Lido) protocol events:
- [x] Staking: `Submitted`, `Unbuffered`
- [x] Rebase: `TokenRebased`, `TransferShares`, `SharesBurnt`
- [x] External Shares (V3 stVaults): `ExternalSharesMinted`, `ExternalSharesBurnt`, etc.
- [x] Validator/CL: `CLValidatorsUpdated`, `ETHDistributed`, `InternalShareRateUpdated`
- [x] Protocol Control: `StakingPaused`, `StakingResumed`, `StakingLimitSet`
- [x] Withdrawals: `ELRewardsReceived`, `WithdrawalsReceived`

### `/erc1155` - Multi-Token Standard

- [x] `TransferSingle` event
- [x] `TransferBatch` event
- [x] `ApprovalForAll` event
- [x] `URI` event

### `/balances` - Token & Native Balances

#### ERC-20 Balances (`erc20-balances`)
Tracks ERC-20 token balances via batched RPC calls (`balanceOf`)

#### Native Balances (`native-balances`)
Extracts native ETH balances for addresses

### `/dex` - Decentralized Exchanges

- [x] Uniswap <https://app.uniswap.org/>
  - [x] V1
    - [x] JustSwap V1 (now SunSwap)
  - [x] V2
    - [x] SunSwap V2 <https://sunswap.com>
  - [x] V3
  - [x] V4
- [x] SunPump V1 <https://sunpump.meme/>
- [x] Balancer <https://balancer.fi/>
  - [x] V2 WeightedPool
  - [x] V3 StablePool
  - [x] V3 Vault
- [x] CurveFi <https://www.curve.finance/>
- [x] Bancor <https://bancor.network>
  - [x] StandardPoolConverter
- [x] Cow Protocol <https://cow.fi/>
  - [x] GPv2Settlement

## Database Sinks

### ClickHouse
- [x] `db-evm-transfers-clickhouse` - ERC-20 and native transfers
- [x] `db-evm-balances-clickhouse` - Token and native balances
- [x] `db-evm-dex-clickhouse` - DEX events and trades
- [x] `db-blocks-clickhouse` - Block metadata

### PostgreSQL
- [x] `db-evm-transfers-postgres` - ERC-20 and native transfers
- [x] `db-evm-balances-postgres` - Token and native balances
- [x] `db-evm-dex-postgres` - DEX events and trades

## Schema Concepts

### Call Metadata (EXTENDED Detail Level)

All proto schemas now include optional `Call` metadata for events, providing rich call context:

```protobuf
message Call {
  bytes caller = 1;     // Address that made the call
  uint32 index = 2;     // Index of the call within the transaction
  uint32 depth = 3;     // Depth of the call in the call tree (0 = root)
  CallType call_type = 4; // CALL, DELEGATE, STATIC, CREATE, etc.
}
```

This metadata is **only available on chains with DetailLevel: EXTENDED**, including:
- Ethereum
- Base
- BSC (Binance Smart Chain)
- Polygon
- ArbitrumOne
- Optimism
- Avalanche
- TRON

Call metadata enables advanced analysis such as:
- Identifying direct vs. delegated calls
- Tracking call depth and execution flow
- Distinguishing between EOA and contract interactions
- Understanding complex multi-call transactions

### Transaction Structure

All event schemas follow a consistent structure:

```protobuf
message Events { repeated Transaction transactions = 1; }

message Transaction {
  bytes hash = 1;
  bytes from = 2;
  optional bytes to = 3;
  uint64 nonce = 5;
  string gas_price = 6;  // uint256
  uint64 gas_limit = 7;
  uint64 gas_used = 8;
  string value = 9;      // uint256
  repeated Log logs = 10;
}
```

## Links

- [Substreams Documentation](https://substreams.streamingfast.io)
- [GitHub Repository](https://github.com/pinax-network/substreams-evm)
