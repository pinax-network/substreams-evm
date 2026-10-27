use crate::PoolEntry;
use substreams_abis::dex::balancer;
use substreams_ethereum::pb::eth::v2::Log;
use substreams_ethereum::Event;

pub fn collect_balancer(log: &Log) -> Option<PoolEntry> {
    let event = balancer::v3::vault::events::PoolRegistered::match_and_decode(log)?;
    let tokens = event.token_config.iter().map(|(token, ..)| token.to_vec()).collect();
    Some(PoolEntry { address: event.pool.to_vec(), tokens, factory: event.factory.to_vec() })
}
