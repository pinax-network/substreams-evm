/// Processes unified `DexPools` output and writes rows directly to the
/// `dex_pools` table.
///
/// The `dex_pools` table provides a canonical, protocol-agnostic pool registry
/// that can be queried by downstream analytics without joining across many
/// protocol-specific pool tables.
use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::dex::v1::{DexPool, DexPools};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::set_clock;

pub fn process_dex_pools(encoding: &Encoding, tables: &mut Tables, clock: &Clock, dex_pools: &DexPools) {
    for pool in &dex_pools.pools {
        write_pool(encoding, tables, clock, pool);
    }
}

fn write_pool(encoding: &Encoding, tables: &mut Tables, clock: &Clock, pool: &DexPool) {
    let block_num = clock.number.to_string();
    let block_hash = format!("0x{}", &clock.id);
    let log_index = pool.log_index.to_string();

    let key = [
        ("block_num", block_num.clone()),
        ("block_hash", block_hash.clone()),
        ("log_index", log_index),
    ];

    let row = tables.create_row("dex_pools", key);

    // block
    set_clock(clock, row);

    // pool identity
    row.set("address", bytes_to_string(&pool.address, encoding));
    row.set("protocol", &pool.protocol);
    row.set("factory", bytes_to_string(&pool.factory, encoding));

    // generalized coins array (stored as comma-separated hex addresses)
    let coins: Vec<String> = pool.coins.iter().map(|c| bytes_to_string(c, encoding)).collect();
    row.set("coins", coins.join(","));

    // creation context
    row.set("tx_hash", bytes_to_hex(&pool.tx_hash));
    row.set("log_ordinal", pool.log_ordinal);
}
