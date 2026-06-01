use crate::PoolEntry;
use substreams_abis::dex::sunpump;
use substreams_ethereum::pb::eth::v2::Log;
use substreams_ethereum::Event;

pub fn collect_sunpump(log: &Log) -> Option<PoolEntry> {
    if let Some(event) = sunpump::v1::launchpadproxy::events::TokenCreate::match_and_decode(log) {
        return Some(PoolEntry { address: event.token_address.to_vec(), tokens: vec![], factory: log.address.clone() });
    }
    if let Some(event) = sunpump::legacy::launchpad::events::TokenCreate::match_and_decode(log) {
        return Some(PoolEntry { address: event.token_address.to_vec(), tokens: vec![], factory: log.address.clone() });
    }
    None
}
