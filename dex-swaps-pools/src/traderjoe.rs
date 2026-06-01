use crate::PoolEntry;
use substreams_abis::dex::traderjoe;
use substreams_ethereum::pb::eth::v2::Log;
use substreams_ethereum::Event;

pub fn collect_traderjoe(log: &Log) -> Option<PoolEntry> {
    let event = traderjoe::lbfactory::events::LbPairCreated::match_and_decode(log)?;
    Some(PoolEntry { address: event.lb_pair.to_vec(), tokens: vec![event.token_x.to_vec(), event.token_y.to_vec()], factory: log.address.clone() })
}
