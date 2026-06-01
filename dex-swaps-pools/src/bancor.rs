use crate::PoolEntry;
use substreams_abis::dex::bancor::{bancorconverterfactory, contractfeatures, converterfactory};
use substreams_ethereum::pb::eth::v2::Log;
use substreams_ethereum::Event;

pub fn collect_bancor(log: &Log) -> Option<PoolEntry> {
    if let Some(event) = converterfactory::events::NewConverter::match_and_decode(log) {
        return Some(PoolEntry { address: event.converter.to_vec(), tokens: vec![], factory: log.address.clone() });
    }
    // Legacy BancorConverterFactory
    if let Some(event) = bancorconverterfactory::events::NewConverter::match_and_decode(log) {
        return Some(PoolEntry { address: event.converter.to_vec(), tokens: vec![], factory: log.address.clone() });
    }
    if let Some(event) = contractfeatures::events::FeaturesAddition::match_and_decode(log) {
        return Some(PoolEntry { address: event.address.to_vec(), tokens: vec![], factory: log.address.clone() });
    }
    None
}
