# ERC-20 Transfers

> Substreams for extracting ERC-20 & WETH events from EVM blockchains.

## Overview

This Substreams module extracts token-related events from Ethereum and EVM-compatible blockchains, including:

- **ERC-20 Events**: `Transfer`, `Approval`
- **WETH Events**: `Deposit`, `Withdrawal`
- **USDC Events**: `Mint`, `Burn`
- **USDT Events**: `Issue`, `Redeem`
- **stETH Events**: `TokenRebased`, `SharesBurnt`, `TransferShares`, `ExternalSharesBurnt`

## Quickstart

```bash
# Build the substreams
make build

# Run with GUI
make gui

# Package the substreams
make pack
```

## Modules

| Module | Type | Description |
|--------|------|-------------|
| `map_events` | Map | Extracts ERC-20 & WETH events from blocks |

## Usage

```bash
substreams gui -e eth.substreams.pinax.network:443 \
  substreams.yaml map_events \
  -s 23000000
```

## Protobuf Schema

The output follows the `erc20.transfers.v1.Events` protobuf schema containing:

- **Transaction**: hash, from, to, nonce, gas info, value
- **Log**: address, ordinal, topics, data, and typed event data
- **Events**: Transfer, Approval, Deposit, Withdrawal, Mint, Burn, etc.

## Links

- [Substreams Documentation](https://substreams.streamingfast.io)
- [GitHub Repository](https://github.com/pinax-network/substreams-evm)
