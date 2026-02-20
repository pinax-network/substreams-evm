# Trader Joe V2 — Liquidity Book

Substreams module for [Trader Joe V2](https://traderjoexyz.com) Liquidity Book DEX events.

## Events Captured

| Event | Source | Description |
|-------|--------|-------------|
| `Swap` | LBPair | Trade executed through a Liquidity Book pool |
| `DepositedToBins` | LBPair | Liquidity added across bin IDs |
| `WithdrawnFromBins` | LBPair | Liquidity removed from bin IDs |
| `CompositionFees` | LBPair | Fee composition per bin |
| `LbPairCreated` | LBFactory | New pool created (tracked via `store_pool`) |

## Packed `bytes32` → Decoded `uint128` Fields

Trader Joe LB packs **two `uint128` values** into each `bytes32`:

```
bytes32 layout: [ upper 128 bits (tokenX) | lower 128 bits (tokenY) ]
```

This module **decodes** them into separate fields:

| Raw `bytes32` | Decoded Fields |
|---------------|---------------|
| `amountsIn` | `amount_in_x` (tokenX) + `amount_in_y` (tokenY) |
| `amountsOut` | `amount_out_x` (tokenX) + `amount_out_y` (tokenY) |
| `totalFees` | `total_fees_x` + `total_fees_y` |
| `protocolFees` | `protocol_fees_x` + `protocol_fees_y` |

### Swap Direction

In a swap, only **one side** is non-zero:

```
AVAX → USDC:
  amount_in_x  = 100000000 (AVAX in)     amount_in_y  = 0
  amount_out_x = 0                        amount_out_y = 200000000 (USDC out)

USDC → AVAX:
  amount_in_x  = 0                        amount_in_y  = 200000000 (USDC in)
  amount_out_x = 100000000 (AVAX out)     amount_out_y = 0
```

Fees are denominated in the **input token** (same side as amountsIn).

## Protobuf Schema

```protobuf
message Swap {
  bytes sender = 1;
  bytes to = 2;
  uint32 id = 3;                    // active bin id
  string amount_in_x = 4;           // uint128
  string amount_in_y = 5;           // uint128
  string amount_out_x = 6;          // uint128
  string amount_out_y = 7;          // uint128
  uint32 volatility_accumulator = 8;
  string total_fees_x = 9;          // uint128
  string total_fees_y = 10;         // uint128
  string protocol_fees_x = 11;      // uint128
  string protocol_fees_y = 12;      // uint128
}

message LbPairCreated {
  bytes token_x = 1;
  bytes token_y = 2;
  uint32 bin_step = 3;
  bytes lb_pair = 4;
  uint32 pid = 5;
}

message CompositionFees {
  bytes sender = 1;
  uint32 id = 2;
  string total_fees_x = 3;          // uint128
  string total_fees_y = 4;          // uint128
  string protocol_fees_x = 5;       // uint128
  string protocol_fees_y = 6;       // uint128
}

message DepositedToBins {
  bytes sender = 1;
  bytes to = 2;
  repeated uint64 ids = 3;
  repeated bytes amounts = 4;        // bytes32[] (packed x|y per bin)
}

message WithdrawnFromBins {
  bytes sender = 1;
  bytes to = 2;
  repeated uint64 ids = 3;
  repeated bytes amounts = 4;        // bytes32[] (packed x|y per bin)
}
```

## Notes for `db-evm-dex`

When implementing Trader Joe in the DEX database:
- Determine swap direction by checking which `amount_in_x`/`amount_in_y` is non-zero
- The non-zero input side tells you which token was sold; the non-zero output side tells you which was bought
- `id` is the active bin — can be used to derive the price (bin price = `(1 + binStep/10000) ^ (id - 8388608)`)
- `tokenX`/`tokenY` addresses come from `LbPairCreated` events (or the `store_pool`)

### ⚠️ Flash Loan Swaps

Trader Joe supports flash loans (`FlashLoan` event). During a flash loan, **both** `amount_in_x` and `amount_in_y` can be non-zero, which breaks the normal swap direction logic.

**Filter these out** — if both `amount_in_x > 0` AND `amount_in_y > 0`, skip the swap. Flash loans are too much of an edge case to support as regular swaps. Same approach as Uniswap V2 flash swaps ([#66](https://github.com/pinax-network/substreams-evm/issues/66)).

## Links

- [Trader Joe Docs](https://docs.traderjoexyz.com)
- [LBPair ABI](https://github.com/pinax-network/substreams-abis/tree/main/abi/dex/traderjoe)
- [Joe V2 Source](https://github.com/traderjoe-xyz/joe-v2)
