# Pricing

> Monthly cost estimates for running `db-evm-dex` substreams per chain.

## Rates

| Metric | Cost |
|---|---|
| Processed bytes | $150.00 / TB / month |
| Processed blocks | $1.75 / 1M blocks / month |

## Chain Profiles

| Chain | Bytes / 10K blocks | Block time | Blocks / month |
|---|---|---|---|
| ETH Mainnet | 1.1 GB | 12s | 216,000 |
| BSC | 284 MB | 0.45s | 5,760,000 |
| Polygon | 2.5 GB | 2s | 1,296,000 |
| Arbitrum | 0.8 GB | 0.25s | 6,480,000 |
| Optimism | 0.4 GB | 2s | 1,296,000 |
| Base | 1.9 GB | 2s | 1,296,000 |
| Avalanche | 1.0 GB | 2s | 1,296,000 |
| Unichain | 40 MB | 1s | 2,592,000 |
| HyperCore | 28 MB | 0.2s | 12,960,000 |

## Monthly Cost Estimates

| Chain | Data / month | Bytes cost | Blocks cost | **Total / month** |
|---|---|---|---|---|
| ETH Mainnet | 23.8 GB | $3.56 | $0.38 | **$3.94** |
| BSC | 163.6 GB | $24.54 | $10.08 | **$34.62** |
| Polygon | 324.0 GB | $48.60 | $2.27 | **$50.87** |
| Arbitrum | 518.4 GB | $77.76 | $11.34 | **$89.10** |
| Optimism | 51.8 GB | $7.78 | $2.27 | **$10.04** |
| Base | 246.2 GB | $36.94 | $2.27 | **$39.20** |
| Avalanche | 129.6 GB | $19.44 | $2.27 | **$21.71** |
| Unichain | 10.4 GB | $1.56 | $4.54 | **$6.09** |
| HyperCore | 36.3 GB | $5.44 | $22.68 | **$28.12** |
| | | | | |
| **Total** | | | | **$283.70** |

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
