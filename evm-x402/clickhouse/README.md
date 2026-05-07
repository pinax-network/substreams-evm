# evm-x402 ClickHouse

ClickHouse sink package for settled x402 EVM payments.

The sink writes individual settlements into `x402_payments` and maintains materialized rollups for daily volume, facilitators, and payer/recipient addresses.
