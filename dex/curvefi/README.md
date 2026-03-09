# CurveFi Substreams

Substreams module for extracting CurveFi Pool events from TRON blockchain.

## Events

This module tracks the following CurveFi Pool events based on the latest ABI:

- **TokenExchange**: Token swap events in CurveFi pools
- **AddLiquidity**: Liquidity additions to pools
- **RemoveLiquidity**: Full liquidity removals
- **RemoveLiquidityOne**: Single-sided liquidity removals
- **RemoveLiquidityImbalance**: Imbalanced liquidity removals
- **CommitNewAdmin**: Admin change proposals
- **NewAdmin**: Admin changes
- **CommitNewFee**: Fee change proposals
- **NewFee**: Fee changes
- **RampA**: Amplification parameter changes
- **StopRampA**: Amplification parameter change stops
- **Init**: Pool initialization events decoded from constructor calldata (direct deployments)

### Init Event

The `Init` event tracks CurveFi pool initialization for pools deployed **directly** (not via a factory). It is decoded from the contract creation transaction's `__init__` constructor calldata.

**Parameters captured:**

- `address`: Deployed pool contract address
- `owner`: Contract owner / initial admin address
- `coins`: Array of coin (token) addresses in the pool (e.g., DAI, USDC, USDT for 3Pool)
- `pool_token`: LP token address representing pool shares
- `a`: Amplification coefficient (_A parameter)
- `fee`: Exchange fee (scaled to 1e10)
- `admin_fee`: Admin fee fraction (scaled to 1e10)

**How it works:**

CurveFi Vyper `__init__` constructors do not emit standard EVM events. Instead, the module decodes the ABI-encoded constructor arguments from the end of the deployment bytecode in the contract creation transaction. It attempts to decode for 3-coin, 4-coin, and 2-coin pool layouts (in that order), validating each field as a properly padded ABI address.

**Coverage:**

- ✅ Direct pool deployments (e.g., 3Pool at `0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7`)
- ✅ Factory-deployed pools are already captured via `PlainPoolDeployed` / `MetaPoolDeployed` events

## Store

The module includes `store.rs` which provides store handlers for tracking:
- Pool information (pool address, coins) — populated from `Init`, `PlainPoolDeployed`, `MetaPoolDeployed`, and `CryptoPoolDeployed` events

## Building

```bash
make build
```

## Usage

```bash
substreams pack substreams.yaml
```
