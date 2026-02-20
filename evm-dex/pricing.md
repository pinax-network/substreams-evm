# Pricing

> Monthly cost estimates for running `db-evm-dex` substreams per chain.

## Rates

| Metric | Cost |
|---|---|
| Processed bytes | $150.00 / TB / month |
| Processed blocks | $1.75 / 1M blocks / month |

## Chain Profiles

> Measured via `substreams run` with `clickhouse-evm-dex-v0.2.6.spkg` (1000 block samples, Feb 2026)

| Chain | Bytes / 10K blocks | Block time | Blocks / month |
|---|---|---|---|
| ETH Mainnet | 960 MB | 12s | 216,000 |
| BSC | 285 MB | 0.45s | 5,760,000 |
| Base | 1.53 GB | 2s | 1,296,000 |
| Polygon | 237 MB | 2s | 1,296,000 |
| Arbitrum | 51 MB | 0.25s | 10,368,000 |
| Optimism | 67 MB | 2s | 1,296,000 |
| Avalanche | 51 MB | 2s | 1,296,000 |
| Unichain | 33 MB | 1s | 2,592,000 |
| HyperCore* | 28 MB | 0.2s | 12,960,000 |

> *HyperCore uses a different block type (`pinax.hypercore.v1.Block`) — not directly testable with the EVM DEX spkg.

## Monthly Cost Estimates

| Chain | Data / month | Bytes cost | Blocks cost | **Total / month** |
|---|---|---|---|---|
| ETH Mainnet | 20.7 GB | $3.11 | $0.38 | **$3.49** |
| BSC | 164.2 GB | $24.62 | $10.08 | **$34.70** |
| Base | 198.3 GB | $29.74 | $2.27 | **$32.01** |
| Polygon | 30.7 GB | $4.61 | $2.27 | **$6.88** |
| Arbitrum | 52.9 GB | $7.93 | $18.14 | **$26.08** |
| Optimism | 8.7 GB | $1.30 | $2.27 | **$3.57** |
| Avalanche | 6.6 GB | $0.99 | $2.27 | **$3.26** |
| Unichain | 8.6 GB | $1.28 | $4.54 | **$5.82** |
| HyperCore* | 36.3 GB | $5.44 | $22.68 | **$28.12** |
| | | | | |
| **Total** | | | | **$143.93** |

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
