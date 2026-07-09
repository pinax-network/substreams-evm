# evm-x402

`evm-x402` maps normalized `x402` EVM payment events into database changes.

The event extractor lives in the sibling `x402` package. This package imports `x402-v0.1.0.spkg` and exposes `db_out` for SQL sinks.
