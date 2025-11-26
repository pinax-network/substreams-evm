mod balancer;
mod bancor;
mod cow;
mod curvefi;
mod logs;
mod store;
mod sunpump;
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
    let encoding = if params == "tron_base58" { Encoding::TronBase58 } else { Encoding::Hex };

    // Tron DEX Substreams
    sunpump::process_events(&encoding, &mut tables, &clock, &events_sunpump, &store_sunpump);

    // Ethereum DEX Substreams
    balancer::process_events(&encoding, &mut tables, &clock, &events_balancer, &store_balancer);
    bancor::process_events(&encoding, &mut tables, &clock, &events_bancor, &store_bancor);
    cow::process_events(&encoding, &mut tables, &clock, &events_cow);
    curvefi::process_events(&encoding, &mut tables, &clock, &events_curvefi, &store_curvefi);

    // Uniswap DEX Substreams
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
