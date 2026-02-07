# ERC-20 Balances

> Substreams for tracking ERC-20 token balances via RPC calls for EVM blockchains.

## Overview

This Substreams module fetches ERC-20 token balances by making batched `balanceOf` RPC calls. It processes events from the `erc20-transfers` module and retrieves the resulting balances for all affected addresses.

**Tracked address sources:**

- Transfer events: `from`, `to`
- Approval events: `owner`, `spender`
- OwnershipTransferred events: `previous_owner`, `new_owner` (shared: USDC, USDT, WBTC)
- WETH Deposit events: `dst`
- WETH Withdrawal events: `src`
- USDC Mint events: `minter`, `to`
- USDC Burn events: `burner`
- USDC Blacklisted events: `account`
- USDC UnBlacklisted events: `account`
- USDT Issue events: `owner` (call.caller)
- USDT Redeem events: `owner` (call.caller)
- USDT DestroyedBlackFunds events: `black_listed_user`
- USDT AddedBlackList events: `user`
- USDT RemovedBlackList events: `user`
- WBTC Mint events: `to`
- WBTC Burn events: `burner`
- SAI Mint events: `guy`
- SAI Burn events: `guy`
- stETH Submitted events: `sender`
- stETH TransferShares events: `from`, `to`
- stETH SharesBurnt events: `account`
- stETH ExternalSharesMinted events: `recipient`
- stETH ExternalSharesBurnt events: `owner`
- Transaction `from` address
- Token contract address (`log.address`)

## Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `CHUNK_SIZE` | `100` | Number of `balanceOf` calls to batch in a single RPC request |

## Dependencies

This module imports and depends on:

- [`erc20_transfers`](../erc20-transfers/) - Provides the source events to process
