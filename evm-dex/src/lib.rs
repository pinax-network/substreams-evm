mod aerodrome;
mod balancer;
mod bancor;
mod cow;
mod curvefi;
mod dodo;
mod kyber_elastic;
mod store;
mod sunpump;
mod traderjoe;
mod uniswap_v1;
mod uniswap_v2;
mod uniswap_v3;
mod uniswap_v4;
mod woofi;

use proto::pb::uniswap;
use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams::prelude::*;
use substreams_database_change::pb::database::DatabaseChanges;

#[substreams::handlers::map]
pub fn db_out(
    params: String,
    clock: Clock,

    // Foundational Store
    pools: FoundationalStore,

    // Tron DEX
    events_sunpump: proto::pb::sunpump::v1::Events,

    // Ethereum DEX
    events_balancer: proto::pb::balancer::v1::Events,
    events_bancor: proto::pb::bancor::v1::Events,
    events_curvefi: proto::pb::curvefi::v1::Events,
    events_cow: proto::pb::cow::v1::Events,

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
    events_uniswap_v4: uniswap::v4::Events
) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    // Handle support both EVM & TVM address encoding
    let encoding = common::handle_encoding_param(&params);

    // Tron DEX Substreams
    sunpump::process_events(&encoding, &mut tables, &clock, &events_sunpump, &pools);

    // Ethereum DEX Substreams
    balancer::process_events(&encoding, &mut tables, &clock, &events_balancer, &pools);
    bancor::process_events(&encoding, &mut tables, &clock, &events_bancor, &pools);
    cow::process_events(&encoding, &mut tables, &clock, &events_cow);
    curvefi::process_events(&encoding, &mut tables, &clock, &events_curvefi, &pools);

    // New DEX Substreams
    aerodrome::process_events(&encoding, &mut tables, &clock, &events_aerodrome, &pools);
    dodo::process_events(&encoding, &mut tables, &clock, &events_dodo);
    woofi::process_events(&encoding, &mut tables, &clock, &events_woofi);
    traderjoe::process_events(&encoding, &mut tables, &clock, &events_traderjoe, &pools);
    kyber_elastic::process_events(&encoding, &mut tables, &clock, &events_kyber_elastic, &pools);

    // Uniswap DEX Substreams
    uniswap_v1::process_events(&encoding, &mut tables, &clock, &events_uniswap_v1, &pools);
    uniswap_v2::process_events(&encoding, &mut tables, &clock, &events_uniswap_v2, &pools);
    uniswap_v3::process_events(&encoding, &mut tables, &clock, &events_uniswap_v3, &pools);
    uniswap_v4::process_events(&encoding, &mut tables, &clock, &events_uniswap_v4, &pools);

    // ONLY include blocks if events are present
    if !tables.tables.is_empty() {
        common::clickhouse::set_clock(&clock, tables.create_row("blocks", [("block_num", clock.number.to_string())]));
    }

    substreams::log::info!("Total rows {}", tables.all_row_count());
    Ok(tables.to_database_changes())
}
