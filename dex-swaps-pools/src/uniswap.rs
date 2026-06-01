use crate::PoolEntry;
use substreams_abis::dex::uniswap;
use substreams_ethereum::pb::eth::v2::Log;
use substreams_ethereum::Event;

pub fn collect_uniswap_v1(log: &Log) -> Option<PoolEntry> {
    let event = uniswap::v1::factory::events::NewExchange::match_and_decode(log)?;
    Some(PoolEntry { address: event.exchange.to_vec(), tokens: vec![event.token.to_vec()], factory: log.address.clone() })
}

pub fn collect_uniswap_v2(log: &Log) -> Option<PoolEntry> {
    let event = uniswap::v2::factory::events::PairCreated::match_and_decode(log)?;
    Some(PoolEntry { address: event.pair.to_vec(), tokens: vec![event.token0.to_vec(), event.token1.to_vec()], factory: log.address.clone() })
}

pub fn collect_uniswap_v3(log: &Log) -> Option<PoolEntry> {
    let event = uniswap::v3::factory::events::PoolCreated::match_and_decode(log)?;
    Some(PoolEntry { address: event.pool.to_vec(), tokens: vec![event.token0.to_vec(), event.token1.to_vec()], factory: log.address.clone() })
}

pub fn collect_uniswap_v4(log: &Log) -> Option<PoolEntry> {
    let event = uniswap::v4::poolmanager::events::Initialize::match_and_decode(log)?;
    Some(PoolEntry { address: event.id.to_vec(), tokens: vec![event.currency0.to_vec(), event.currency1.to_vec()], factory: log.address.clone() })
}
