use crate::PoolEntry;
use substreams_abis::dex::kyber::elastic;
use substreams_ethereum::pb::eth::v2::Log;
use substreams_ethereum::Event;

pub fn collect_kyber_elastic(log: &Log) -> Option<PoolEntry> {
    let event = elastic::factory::events::PoolCreated::match_and_decode(log)?;
    Some(PoolEntry { address: event.pool.to_vec(), tokens: vec![event.token0.to_vec(), event.token1.to_vec()], factory: log.address.clone() })
}
