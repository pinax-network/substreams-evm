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
- **Init** ⚠️: Pool initialization events (infrastructure ready, requires ABI addition)

### Init Event

The `Init` event tracks CurveFi pool initialization with the following parameters:

- `owner`: Contract owner address
- `coins`: Array of coin addresses in the pool (e.g., DAI, USDC, USDT for 3Pool)
- `pool_token`: LP token address representing pool shares
- `a`: Amplification coefficient (A parameter)
- `fee`: Exchange fee
- `admin_fee`: Admin fee

#### Activation Requirements

The Init event infrastructure is fully implemented in:
- Protocol buffers definition (`proto/v1/dex/curvefi.proto`)
- Database schema (`clickhouse-evm-dex/schema.1.table.curvefi.sql`)
- Event processing logic (`clickhouse-evm-dex/src/curvefi.rs`)

However, the event handler is currently commented out in `dex/curvefi/src/lib.rs` because CurveFi's Vyper `__init__` constructor doesn't emit a standard event.

**To activate Init event tracking**, one of the following is required:

1. Add the Init event ABI definition to the `substreams-abis` repository
2. Modify CurveFi contracts to emit a custom Init event during deployment
3. Implement constructor call decoding to extract initialization parameters from contract creation transactions

Once the event source is available, uncomment the Init event handler in `src/lib.rs` (lines ~175-189) to enable tracking.

## Store

The module includes `store.rs` which provides store handlers for tracking:
- Pool information (pool address, provider, total liquidity)

## Building

```bash
make build
```

## Usage

```bash
substreams pack substreams.yaml
```
