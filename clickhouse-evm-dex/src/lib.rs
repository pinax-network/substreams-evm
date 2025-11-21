mod balancer;
mod bancor;
mod cow;
mod curvefi;
mod foundational_stores;
mod justswap;
mod logs;
mod sunpump;
mod sunswap;
mod transactions;
mod uniswap_v1;
mod uniswap_v2;
mod uniswap_v3;
mod uniswap_v4;

use common::Encoding;
use proto::pb::uniswap;
use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams::prelude::*;
use substreams_database_change::pb::database::DatabaseChanges;

#[substreams::handlers::map]
pub fn db_out(
    params: String,
    clock: Clock,

    // TVM events
    events_justswap: proto::pb::justswap::v1::Events,
    events_sunswap: proto::pb::sunswap::v1::Events,
    events_sunpump: proto::pb::sunpump::v1::Events,

    // EVM events
    events_balancer: proto::pb::balancer::v1::Events,
    events_bancor: proto::pb::bancor::v1::Events,
    events_cow: proto::pb::cow::v1::Events,
    events_curvefi: proto::pb::curvefi::v1::Events,

    // Uniswap events
    events_uniswap_v1: uniswap::v1::Events,
    events_uniswap_v2: uniswap::v2::Events,
    events_uniswap_v3: uniswap::v3::Events,
    events_uniswap_v4: uniswap::v4::Events,

    // TVM stores
    store_justswap: StoreGetProto<proto::pb::justswap::v1::NewExchange>,
    store_sunswap: StoreGetProto<proto::pb::sunswap::v1::PairCreated>,
    store_sunpump: StoreGetProto<proto::pb::sunpump::v1::TokenCreate>,

    // EVM stores
    store_balancer: StoreGetProto<proto::pb::balancer::v1::PoolRegistered>,
    store_bancor: StoreGetProto<proto::pb::bancor::v1::Activation>,
    store_curvefi: StoreGetProto<proto::pb::curvefi::v1::PlainPoolDeployed>,

    // Uniswap stores
    store_uniswap_v1: StoreGetProto<uniswap::v1::NewExchange>,
    store_uniswap_v2: StoreGetProto<uniswap::v2::PairCreated>,
    store_uniswap_v3: StoreGetProto<uniswap::v3::PoolCreated>,
    store_uniswap_v4: StoreGetProto<uniswap::v4::Initialize>,
) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    // Handle support both EVM & TVM address encoding
    let encoding = if params == "tron_base58" { Encoding::TronBase58 } else { Encoding::Hex };

    // Process JustSwap events (TRON)
    justswap::process_events(&encoding, &mut tables, &clock, &events_justswap, &store_justswap);

    // Process SunSwap events (TRON)
    sunswap::process_events(&encoding, &mut tables, &clock, &events_sunswap, &store_sunswap);

    // Process SunPump events (TRON)
    sunpump::process_events(&encoding, &mut tables, &clock, &events_sunpump, &store_sunpump);

    // Process Uniswap V1 events (EVM)
    uniswap_v1::process_events(&encoding, &mut tables, &clock, &events_uniswap_v1, &store_uniswap_v1);

    // Process Uniswap V2 events (EVM)
    uniswap_v2::process_events(&encoding, &mut tables, &clock, &events_uniswap_v2, &store_uniswap_v2);

    // Process Uniswap V3 events (EVM)
    uniswap_v3::process_events(&encoding, &mut tables, &clock, &events_uniswap_v3, &store_uniswap_v3);

    // Process Uniswap V4 events (EVM)
    uniswap_v4::process_events(&encoding, &mut tables, &clock, &events_uniswap_v4, &store_uniswap_v4);

    // Process Balancer events (EVM)
    balancer::process_events(&encoding, &mut tables, &clock, &events_balancer, &store_balancer);

    // Process Bancor events (EVM)
    bancor::process_events(&encoding, &mut tables, &clock, &events_bancor, &store_bancor);

    // Process CoW Protocol events (EVM)
    cow::process_events(&encoding, &mut tables, &clock, &events_cow);

    // Process Curve.fi events (EVM)
    curvefi::process_events(&encoding, &mut tables, &clock, &events_curvefi, &store_curvefi);

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
