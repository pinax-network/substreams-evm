# ERC-20 Balances

> Substreams for tracking ERC-20 token balances via RPC calls for EVM blockchains.

## Overview

This Substreams module fetches ERC-20 token balances by making batched `balanceOf` RPC calls. It processes events from the `erc20-transfers` module and retrieves the resulting balances for all affected addresses.

**Tracked address sources:**
- Transfer events: `from`, `to`
- WETH Deposit events: `dst`
- WETH Withdrawal events: `src`
- Approval events: `owner`, `spender`
- USDC Mint events: `minter`, `to`
- USDC Burn events: `burner`
- stETH events: `account`, `from`, `to`, `sender`
- Transaction `from` address
- Token contract address (`log.address`)

## Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `CHUNK_SIZE` | `100` | Number of `balanceOf` calls to batch in a single RPC request |

## Dependencies

This module imports and depends on:

- [`erc20_transfers`](../erc20-transfers/) - Provides the source events to process
