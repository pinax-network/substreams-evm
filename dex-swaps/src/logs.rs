use std::collections::{HashMap, HashSet};

use proto::pb::dex::foundational_store::v1::Pool;
use substreams::{
    store::{StoreGet, StoreGetProto},
    Hex,
};
use substreams_abis::dex::balancer;
use substreams_abis::dex::sunpump;
use substreams_abis::dex::uniswap::v4 as uniswap_v4;
use substreams_ethereum::{pb::eth::v2::{Block, Log}, Event};

pub(crate) type PoolMetadata = Pool;
pub(crate) type PoolMetadataMap = HashMap<Vec<u8>, PoolMetadata>;

pub(crate) fn collect_log_addresses(block: &Block) -> HashSet<Vec<u8>> {
    let mut addresses = HashSet::new();

    for log in block.logs() {
        collect_log_address(log.log, &mut addresses);
    }

    addresses
}

fn collect_log_address(log: &Log, addresses: &mut HashSet<Vec<u8>>) {
    if !log.address.is_empty() {
        addresses.insert(log.address.clone());
    }

    if let Some(event) = uniswap_v4::poolmanager::events::Swap::match_and_decode(log) {
        addresses.insert(event.id.to_vec());
    }

    if let Some(event) = balancer::v3::vault::events::Swap::match_and_decode(log) {
        addresses.insert(event.pool.to_vec());
    }

    if let Some(event) = sunpump::v1::launchpadproxy::events::TokenPurchased::match_and_decode(log) {
        addresses.insert(event.token.to_vec());
    }

    if let Some(event) = sunpump::v1::launchpadproxy::events::TokenSold::match_and_decode(log) {
        addresses.insert(event.token.to_vec());
    }
}

/// Look up pool metadata from the legacy `store_pools` key-value store. Keys are the
/// hex-encoded pool address (lowercase, no `0x`), matching how `store_pools` writes them.
pub(crate) fn get_pools_by_address(store: &StoreGetProto<Pool>, addresses: &HashSet<Vec<u8>>) -> PoolMetadataMap {
    addresses
        .iter()
        .filter_map(|address| store.get_last(Hex::encode(address)).map(|pool| (address.clone(), pool)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_matches_store_pools_encoding() {
        // `store_pools` keys entries via `Hex::encode(address)` — lowercase, no `0x`.
        assert_eq!(Hex::encode([0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }
}
