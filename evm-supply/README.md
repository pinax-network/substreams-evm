# EVM Token Supply

> Substreams for tracking ERC-20 token total supply via RPC calls for EVM blockchains.

## Overview

This Substreams module fetches ERC-20 token total supply by making batched `totalSupply` RPC calls. It processes balance change events from the `erc20-balances` module and retrieves the total supply for all affected token contracts.

## Parameters

- `chunk_size` (default: `100`): Number of RPC calls per batch.

## Dependencies

- [`erc20-balances`](../erc20-balances): Provides unique balance changes per address/contract.
