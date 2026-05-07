# evm-x402 ClickHouse

ClickHouse sink package for settled x402 EVM payments.

The sink writes individual settlements into `x402_payments` with block, transaction, log, and call metadata.

It maintains materialized aggregate states for facilitator-first totals, recipient-first totals, and time-windowed volume grouped by facilitator, recipient, asset, and settlement type.
