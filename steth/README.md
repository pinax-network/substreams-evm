# stETH Events

> Substreams for extracting Lido stETH protocol events from Ethereum.

## Overview

This Substreams module extracts all stETH (Lido) protocol events including:

**ERC-20 Events:**
- `Transfer`, `Approval`

**Staking Events:**
- `Submitted` - User deposits
- `Unbuffered` - Ether sent to deposit contract

**Rebase Events:**
- `TokenRebased` - Daily rebase when oracle reports
- `TransferShares` - Share transfers
- `SharesBurnt` - Shares burned

**External Shares Events (Lido V3 stVaults):**
- `ExternalSharesMinted`
- `ExternalSharesBurnt`
- `ExternalEtherTransferredToBuffer`
- `ExternalBadDebtInternalized`
- `MaxExternalRatioBPSet`

**Validator/CL Events:**
- `CLValidatorsUpdated`
- `DepositedValidatorsChanged`
- `ETHDistributed`
- `InternalShareRateUpdated`

**Protocol Control Events:**
- `StakingPaused`, `StakingResumed`
- `StakingLimitSet`, `StakingLimitRemoved`

**EL/Withdrawal Events:**
- `ELRewardsReceived`
- `WithdrawalsReceived`

**Locator Event:**
- `LidoLocatorSet`

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
| `map_events` | Map | Extracts all stETH protocol events |

## Usage

```bash
substreams gui -e eth.substreams.pinax.network:443 \
  substreams.yaml map_events \
  -s 13000000
```

## Contract Address

- **stETH**: `0xae7ab96520de3a18e5e111b5eaab095312d7fe84`

## Protobuf Schema

The output follows the `steth.v1.Events` protobuf schema containing:

- **Transaction**: hash, from, to, nonce, gas info, value
- **Log**: address, ordinal, topics, data, and typed event data
- **Events**: All stETH-specific events listed above

## Links

- [Lido Documentation](https://docs.lido.fi/contracts/lido)
- [stETH Contract on Etherscan](https://etherscan.io/address/0xae7ab96520de3a18e5e111b5eaab095312d7fe84)
- [Substreams Documentation](https://substreams.streamingfast.io)
- [GitHub Repository](https://github.com/pinax-network/substreams-evm)
