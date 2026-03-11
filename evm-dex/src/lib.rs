mod aerodrome;
mod balancer;
mod bancor;
mod cow;
mod curvefi;
mod dex_pools;
mod dex_swaps;
mod dodo;
mod kyber_elastic;
mod logs;
mod pool_extractors;
mod store;
mod sunpump;
mod swap_extractors;
mod traderjoe;
mod transactions;
mod uniswap_v1;
mod uniswap_v2;
mod uniswap_v3;
mod uniswap_v4;
mod woofi;

use proto::pb::dex::v1 as dex;
use proto::pb::uniswap;
use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams::prelude::*;
use substreams_database_change::pb::database::DatabaseChanges;

// ── Unified pool aggregator ──────────────────────────────────────────────────

/// Aggregates pool creation events from all supported DEX protocols into the
/// unified `DexPools` proto. This intermediate module allows downstream
/// consumers (including `db_out`) to write pool data without depending on
/// every individual protocol map module directly.
#[substreams::handlers::map]
pub fn map_dex_pools(
    // Tron DEX
    events_sunpump: proto::pb::sunpump::v1::Events,
    // Ethereum DEX
    events_balancer: proto::pb::balancer::v1::Events,
    events_bancor: proto::pb::bancor::v1::Events,
    events_cow: proto::pb::cow::v1::Events,
    events_curvefi: proto::pb::curvefi::v1::Events,
    // New DEX
    events_aerodrome: proto::pb::aerodrome::v1::Events,
    events_dodo: proto::pb::dodo::v1::Events,
    events_woofi: proto::pb::woofi::v1::Events,
    events_traderjoe: proto::pb::traderjoe::v1::Events,
    events_kyber_elastic: proto::pb::kyber_elastic::v1::Events,
    // Uniswap DEX
    events_uniswap_v1: uniswap::v1::Events,
    events_uniswap_v2: uniswap::v2::Events,
    events_uniswap_v3: uniswap::v3::Events,
    events_uniswap_v4: uniswap::v4::Events,
) -> Result<dex::DexPools, Error> {
    let mut pools = dex::DexPools::default();

    pool_extractors::extract_sunpump_pools(&mut pools.pools, &events_sunpump);
    pool_extractors::extract_balancer_pools(&mut pools.pools, &events_balancer);
    pool_extractors::extract_bancor_pools(&mut pools.pools, &events_bancor);
    let _ = events_cow; // CoW has no pool factory
    pool_extractors::extract_curvefi_pools(&mut pools.pools, &events_curvefi);
    pool_extractors::extract_aerodrome_pools(&mut pools.pools, &events_aerodrome);
    let _ = events_dodo;  // DODO has no pool factory
    let _ = events_woofi; // WooFi has no pool factory
    pool_extractors::extract_traderjoe_pools(&mut pools.pools, &events_traderjoe);
    pool_extractors::extract_kyber_elastic_pools(&mut pools.pools, &events_kyber_elastic);
    pool_extractors::extract_uniswap_v1_pools(&mut pools.pools, &events_uniswap_v1);
    pool_extractors::extract_uniswap_v2_pools(&mut pools.pools, &events_uniswap_v2);
    pool_extractors::extract_uniswap_v3_pools(&mut pools.pools, &events_uniswap_v3);
    pool_extractors::extract_uniswap_v4_pools(&mut pools.pools, &events_uniswap_v4);

    Ok(pools)
}

// ── Unified swap aggregator ──────────────────────────────────────────────────

/// Normalizes swap events from all supported DEX protocols into the unified
/// `DexSwaps` proto. This replaces the ClickHouse materialized-view layer:
/// the swap conversion logic now lives in Substreams so it is versioned,
/// tested, and cached at the protocol layer.
#[substreams::handlers::map]
#[allow(clippy::too_many_arguments)]
pub fn map_dex_swaps(
    // Tron DEX
    events_sunpump: proto::pb::sunpump::v1::Events,
    store_sunpump: StoreGetProto<proto::pb::sunpump::v1::StorePool>,
    // Ethereum DEX
    events_balancer: proto::pb::balancer::v1::Events,
    events_bancor: proto::pb::bancor::v1::Events,
    store_bancor: StoreGetProto<proto::pb::bancor::v1::StorePool>,
    events_cow: proto::pb::cow::v1::Events,
    events_curvefi: proto::pb::curvefi::v1::Events,
    store_curvefi: StoreGetProto<proto::pb::curvefi::v1::StorePool>,
    // New DEX
    events_aerodrome: proto::pb::aerodrome::v1::Events,
    store_aerodrome: StoreGetProto<proto::pb::aerodrome::v1::StorePool>,
    events_dodo: proto::pb::dodo::v1::Events,
    events_woofi: proto::pb::woofi::v1::Events,
    events_traderjoe: proto::pb::traderjoe::v1::Events,
    store_traderjoe: StoreGetProto<proto::pb::traderjoe::v1::StorePool>,
    events_kyber_elastic: proto::pb::kyber_elastic::v1::Events,
    store_kyber_elastic: StoreGetProto<proto::pb::kyber_elastic::v1::StorePool>,
    // Uniswap DEX
    events_uniswap_v1: uniswap::v1::Events,
    store_uniswap_v1: StoreGetProto<uniswap::v1::StorePool>,
    events_uniswap_v2: uniswap::v2::Events,
    store_uniswap_v2: StoreGetProto<uniswap::v2::StorePool>,
    events_uniswap_v3: uniswap::v3::Events,
    store_uniswap_v3: StoreGetProto<uniswap::v3::StorePool>,
    events_uniswap_v4: uniswap::v4::Events,
    store_uniswap_v4: StoreGetProto<uniswap::v4::StorePool>,
) -> Result<dex::DexSwaps, Error> {
    let mut out = dex::DexSwaps::default();

    swap_extractors::extract_sunpump_swaps(&mut out, &events_sunpump, &store_sunpump);
    swap_extractors::extract_balancer_swaps(&mut out.swaps, &events_balancer);
    swap_extractors::extract_bancor_swaps(&mut out.swaps, &events_bancor, &store_bancor);
    swap_extractors::extract_cow_swaps(&mut out.swaps, &events_cow);
    swap_extractors::extract_curvefi_swaps(&mut out.swaps, &events_curvefi, &store_curvefi);
    swap_extractors::extract_aerodrome_swaps(&mut out, &events_aerodrome, &store_aerodrome);
    swap_extractors::extract_dodo_swaps(&mut out.swaps, &events_dodo);
    swap_extractors::extract_woofi_swaps(&mut out.swaps, &events_woofi);
    swap_extractors::extract_traderjoe_swaps(&mut out, &events_traderjoe, &store_traderjoe);
    swap_extractors::extract_kyber_elastic_swaps(&mut out.swaps, &events_kyber_elastic, &store_kyber_elastic);
    swap_extractors::extract_uniswap_v1_swaps(&mut out.swaps, &events_uniswap_v1, &store_uniswap_v1);
    swap_extractors::extract_uniswap_v2_swaps(&mut out, &events_uniswap_v2, &store_uniswap_v2);
    swap_extractors::extract_uniswap_v3_swaps(&mut out.swaps, &events_uniswap_v3, &store_uniswap_v3);
    swap_extractors::extract_uniswap_v4_swaps(&mut out.swaps, &events_uniswap_v4, &store_uniswap_v4);

    Ok(out)
}

// ── Database sink ────────────────────────────────────────────────────────────

#[substreams::handlers::map]
#[allow(clippy::too_many_arguments)]
pub fn db_out(
    params: String,
    clock: Clock,

    // Unified pipelines (write to swaps, swaps_flash, dex_pools tables)
    dex_swaps_out: dex::DexSwaps,
    dex_pools_out: dex::DexPools,

    // Tron DEX
    events_sunpump: proto::pb::sunpump::v1::Events,
    store_sunpump: StoreGetProto<proto::pb::sunpump::v1::StorePool>,

    // Ethereum DEX
    events_balancer: proto::pb::balancer::v1::Events,
    events_bancor: proto::pb::bancor::v1::Events,
    events_curvefi: proto::pb::curvefi::v1::Events,
    events_cow: proto::pb::cow::v1::Events,
    store_balancer: StoreGetProto<proto::pb::balancer::v1::StorePool>,
    store_bancor: StoreGetProto<proto::pb::bancor::v1::StorePool>,
    store_curvefi: StoreGetProto<proto::pb::curvefi::v1::StorePool>,

    // New DEX
    events_aerodrome: proto::pb::aerodrome::v1::Events,
    events_dodo: proto::pb::dodo::v1::Events,
    events_woofi: proto::pb::woofi::v1::Events,
    events_traderjoe: proto::pb::traderjoe::v1::Events,
    events_kyber_elastic: proto::pb::kyber_elastic::v1::Events,
    store_aerodrome: StoreGetProto<proto::pb::aerodrome::v1::StorePool>,
    store_traderjoe: StoreGetProto<proto::pb::traderjoe::v1::StorePool>,
    store_kyber_elastic: StoreGetProto<proto::pb::kyber_elastic::v1::StorePool>,

    // Uniswap DEX
    events_uniswap_v1: uniswap::v1::Events,
    events_uniswap_v2: uniswap::v2::Events,
    events_uniswap_v3: uniswap::v3::Events,
    events_uniswap_v4: uniswap::v4::Events,
    store_uniswap_v1: StoreGetProto<uniswap::v1::StorePool>,
    store_uniswap_v2: StoreGetProto<uniswap::v2::StorePool>,
    store_uniswap_v3: StoreGetProto<uniswap::v3::StorePool>,
    store_uniswap_v4: StoreGetProto<uniswap::v4::StorePool>,
) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    // Handle support both EVM & TVM address encoding
    let encoding = common::handle_encoding_param(&params);

    // Unified swap pipeline: write swaps and swaps_flash tables directly from
    // DexSwaps - this replaces the ClickHouse MV conversion layer
    dex_swaps::process_dex_swaps(&encoding, &mut tables, &clock, &dex_swaps_out);

    // Unified pool pipeline: write dex_pools table
    dex_pools::process_dex_pools(&encoding, &mut tables, &clock, &dex_pools_out);

    // Protocol-specific tables (individual event tables for raw data access)
    sunpump::process_events(&encoding, &mut tables, &clock, &events_sunpump, &store_sunpump);
    balancer::process_events(&encoding, &mut tables, &clock, &events_balancer, &store_balancer);
    bancor::process_events(&encoding, &mut tables, &clock, &events_bancor, &store_bancor);
    cow::process_events(&encoding, &mut tables, &clock, &events_cow);
    curvefi::process_events(&encoding, &mut tables, &clock, &events_curvefi, &store_curvefi);
    aerodrome::process_events(&encoding, &mut tables, &clock, &events_aerodrome, &store_aerodrome);
    dodo::process_events(&encoding, &mut tables, &clock, &events_dodo);
    woofi::process_events(&encoding, &mut tables, &clock, &events_woofi);
    traderjoe::process_events(&encoding, &mut tables, &clock, &events_traderjoe, &store_traderjoe);
    kyber_elastic::process_events(&encoding, &mut tables, &clock, &events_kyber_elastic, &store_kyber_elastic);
    uniswap_v1::process_events(&encoding, &mut tables, &clock, &events_uniswap_v1, &store_uniswap_v1);
    uniswap_v2::process_events(&encoding, &mut tables, &clock, &events_uniswap_v2, &store_uniswap_v2);
    uniswap_v3::process_events(&encoding, &mut tables, &clock, &events_uniswap_v3, &store_uniswap_v3);
    uniswap_v4::process_events(&encoding, &mut tables, &clock, &events_uniswap_v4, &store_uniswap_v4);

    // ONLY include blocks if events are present
    if !tables.tables.is_empty() {
        set_clock(&clock, tables.create_row("blocks", [("block_num", clock.number.to_string())]));
    }

    substreams::log::info!("Total rows {}", tables.all_row_count());
    Ok(tables.to_database_changes())
}

pub fn set_clock(clock: &Clock, row: &mut substreams_database_change::tables::Row) {
    row.set("block_num", clock.number);
    row.set("block_hash", format!("0x{}", clock.id));
    if let Some(timestamp) = &clock.timestamp {
        row.set("timestamp", timestamp.seconds);
        row.set("minute", timestamp.seconds / 60);
    }
}
