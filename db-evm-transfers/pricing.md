# Pricing

> Monthly cost estimates for running `db-evm-transfers` substreams per chain.

## Rates

| Metric | Cost |
|---|---|
| Processed bytes | $150.00 / TB / month |
| Processed blocks | $1.75 / 1M blocks / month |

## Chain Profiles

> Measured via `substreams run` with [evm-clickhouse-transfers-v0.3.3.spkg](https://github.com/pinax-network/substreams-evm/releases/download/evm-transfers-v0.3.3/evm-clickhouse-transfers-v0.3.3.spkg) (100 block samples, ~1000 blocks behind head, Feb 2026)

| Chain | Bytes / 10K blocks | Block time | Blocks / month |
|---|---|---|---|
| ETH Mainnet | 950 MB | 12s | 216,000 |
| BSC | 410 MB | 0.45s | 5,760,000 |
| Base | 490 MB | 2s | 1,296,000 |
| Polygon | 1.50 GB | 2s | 1,296,000 |
| Arbitrum | 19 MB | 0.25s | 10,368,000 |
| Optimism | 87 MB | 2s | 1,296,000 |
| Avalanche | 42 MB | 2s | 1,296,000 |
| Unichain | 11 MB | 1s | 2,592,000 |
| HyperEVM | 12 MB | 0.2s | 12,960,000 |

## Monthly Cost Estimates

| Chain | Data / month | Bytes cost | Blocks cost | **Total / month** |
|---|---|---|---|---|
| ETH Mainnet | 20.5 GB | $3.08 | $0.38 | **$3.46** |
| BSC | 236.2 GB | $35.42 | $10.08 | **$45.50** |
| Base | 63.5 GB | $9.53 | $2.27 | **$11.80** |
| Polygon | 194.4 GB | $29.16 | $2.27 | **$31.43** |
| Arbitrum | 19.7 GB | $2.95 | $18.14 | **$21.10** |
| Optimism | 11.3 GB | $1.69 | $2.27 | **$3.96** |
| Avalanche | 5.4 GB | $0.82 | $2.27 | **$3.09** |
| Unichain | 2.9 GB | $0.43 | $4.54 | **$4.97** |
| HyperEVM | 15.6 GB | $2.33 | $22.68 | **$25.01** |
| | | | | |
| **Total** | | | | **$150.32** |

## Calculator

```bash
# Single chain
python3 scripts/pricing-calculator.py --chain eth-mainnet

# All chains
python3 scripts/pricing-calculator.py --all

# Custom inputs
python3 scripts/pricing-calculator.py --bytes-per-10k 1.1 --blocks-per-month 216000

# Override rates
python3 scripts/pricing-calculator.py --all --cost-per-tb 120 --cost-per-1m-blocks 1.50
```
