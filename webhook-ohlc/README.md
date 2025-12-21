# webhook-ohlc

A Substreams module that calculates OHLCV (Open, High, Low, Close, Volume) data for DEX trading pairs across multiple protocols.

## Overview

The `webhook-ohlc` crate processes swap events from various DEX protocols and aggregates them into OHLCV candles grouped by pool. The data is structured for easy consumption via webhooks or other downstream systems.

## Features

- **Multi-Protocol Support**: Aggregates data from multiple DEX protocols:
  - Uniswap (V1, V2, V3, V4)
  - SunPump (Tron)
  - Balancer
  - Bancor
  - Curve.fi
  - CoW Protocol

- **Standardized Token Ordering**: Tokens are automatically sorted alphabetically (token0 < token1) for consistent pair representation

- **Quantile-Based High/Low**: 
  - High: 95th percentile (quantile 0.95)
  - Low: 5th percentile (quantile 0.05)
  - This approach provides more robust price ranges by filtering out extreme outliers

- **Comprehensive Metrics**:
  - Open, High, Low, Close prices
  - Gross volume (token0 and token1)
  - Net flow (token0 and token1)
  - Transaction count
  - Unique users and transaction senders

## Data Structure

The OHLCV protobuf message includes:

```protobuf
message OHLCV {
  uint64 timestamp = 1;
  uint64 block_num = 2;
  bytes block_hash = 3;
  string protocol = 4;
  bytes factory = 5;
  bytes pool = 6;
  bytes token0 = 7;      // Lexicographically smaller
  bytes token1 = 8;      // Lexicographically larger
  double open = 9;
  double high = 10;      // quantile 0.95
  double low = 11;       // quantile 0.05
  double close = 12;
  string gross_volume0 = 13;
  string gross_volume1 = 14;
  string net_flow0 = 15;
  string net_flow1 = 16;
  uint64 transactions = 17;
  uint64 unique_users = 18;
  uint64 unique_tx_from = 19;
}
```

## Implementation Details

### Price Calculation

Prices are calculated as the ratio of output to input amounts for each swap:
```
price = output_amount / input_amount
```

### Volume Tracking

- **Gross Volume**: Absolute sum of all token amounts swapped
- **Net Flow**: Directional flow showing net movement of tokens (positive = flowing in, negative = flowing out)

### Pool Grouping

Data is automatically grouped by:
1. Protocol (e.g., "uniswap_v2", "uniswap_v3")
2. Factory address
3. Pool address
4. Token pair (canonically ordered)

## Building

```bash
cd webhook-ohlc
cargo build --release
```

## Usage

The module exposes a single handler `ohlc_out` that consumes events from all supported DEX protocols and outputs OHLCV data:

```rust
#[substreams::handlers::map]
pub fn ohlc_out(...) -> Result<Events, Error>
```

## Protocol-Specific Notes

### Uniswap V1
- Pairs ETH with a single token per exchange
- ETH is represented with empty bytes for consistent handling

### Uniswap V2 & V3
- Support arbitrary token pairs
- Token ordering is normalized to alphabetical

### Uniswap V4
- Uses pool ID instead of pool address
- Maintains same swap semantics as V3

## Contributing

To add support for additional DEX protocols:

1. Add the protocol events to the `ohlc_out` handler parameters
2. Implement a `process_<protocol>_events` function
3. Extract swap data and call `add_swap` on the appropriate accumulator
4. Ensure tokens are normalized using `normalize_token_pair`

## License

Apache-2.0
