# stETH Events

> Substreams for extracting Lido stETH protocol events from Ethereum.

## Overview

This Substreams module extracts all stETH (Lido) protocol events including:

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

## Links

- [Lido Documentation](https://docs.lido.fi/contracts/lido)
- [stETH Contract on Etherscan](https://etherscan.io/address/0xae7ab96520de3a18e5e111b5eaab095312d7fe84)
- [Substreams Documentation](https://substreams.streamingfast.io)
- [GitHub Repository](https://github.com/pinax-network/substreams-evm)
