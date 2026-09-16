# Isolated comparison sink

Build with `make -C evm-balances-storage clickhouse` from the repository root.
Use the resulting package with a **new, dedicated ClickHouse database** and
finalized blocks only. Table names match the existing balances interface, but
this schema retains block history for parity investigations. It must not be
applied over the production balance tables.

There is no automatic setup or live-sink deployment in this package. The local
comparison runner can qualify a range without writing to a ClickHouse server.
The schema has been exercised with ClickHouse local; a real SQL-sink run remains
a separate qualification step.
