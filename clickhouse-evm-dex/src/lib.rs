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
    justswap_events: proto::pb::justswap::v1::Events,
    sunswap_events: proto::pb::sunswap::v1::Events,
    sunpump_events: proto::pb::sunpump::v1::Events,
    uniswap_v1_events: uniswap::v1::Events,
    uniswap_v2_events: uniswap::v2::Events,
    uniswap_v3_events: uniswap::v3::Events,
    uniswap_v4_events: uniswap::v4::Events,
    store_new_exchange_justswap: StoreGetProto<proto::pb::justswap::v1::NewExchange>,
    store_new_exchange_uniswap_v1: StoreGetProto<uniswap::v1::NewExchange>,
    store_pair_created_sunswap: StoreGetProto<proto::pb::sunswap::v1::PairCreated>,
    store_pair_created_uniswap_v2: StoreGetProto<uniswap::v2::PairCreated>,
    store_pool_created_uniswap_v3: StoreGetProto<uniswap::v3::PoolCreated>,
    store_initialize_uniswap_v4: StoreGetProto<uniswap::v4::Initialize>,
    store_token_create: StoreGetProto<proto::pb::sunpump::v1::TokenCreate>,
) -> Result<DatabaseChanges, Error> {
    let mut tables = substreams_database_change::tables::Tables::new();

    // Handle support both EVM & TVM address encoding
    let encoding = if params == "tron_base58" {
        Encoding::TronBase58
    } else {
        Encoding::Hex
    };

    // Process JustSwap events (TRON)
    justswap::process_events(&encoding, &mut tables, &clock, &justswap_events, &store_new_exchange_justswap);

    // Process SunSwap events (TRON)
    sunswap::process_events(&encoding, &mut tables, &clock, &sunswap_events, &store_pair_created_sunswap);

    // Process SunPump events (TRON)
    sunpump::process_events(&encoding, &mut tables, &clock, &sunpump_events, &store_token_create);

    // Process Uniswap V1 events (EVM)
    uniswap_v1::process_events(&encoding, &mut tables, &clock, &uniswap_v1_events, &store_new_exchange_uniswap_v1);

    // Process Uniswap V2 events (EVM)
    uniswap_v2::process_events(&encoding, &mut tables, &clock, &uniswap_v2_events, &store_pair_created_uniswap_v2);

    // Process Uniswap V3 events (EVM)
    uniswap_v3::process_events(&encoding, &mut tables, &clock, &uniswap_v3_events, &store_pool_created_uniswap_v3);

    // Process Uniswap V4 events (EVM)
    uniswap_v4::process_events(&encoding, &mut tables, &clock, &uniswap_v4_events, &store_initialize_uniswap_v4);

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
