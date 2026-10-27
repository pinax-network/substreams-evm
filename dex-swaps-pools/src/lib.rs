//! One pool decoder map feeding independent Hosted Store and legacy stores.
//! The EVM Pool wire payload and legacy lowercase-hex keys remain unchanged.
use std::collections::HashSet;

use dex_pools::Block;
use prost::Message;
use prost_types::Any;
pub use proto::pb::dex::foundational_store::v1 as pb;
use substreams::errors::Error;
use substreams::pb::sf::substreams::foundational_store::model::v2::{Entry, Key, SinkEntries};
use substreams::pb::substreams::store_delta::Operation;
use substreams::store::{DeltaProto, Deltas, StoreSetIfNotExists};

pub const POOL_TYPE_URL: &str = "type.googleapis.com/dex.foundational_store.v1.Pool";

/// Decode once in the legacy scan order. Keep the first observation of an
/// identity in this block, matching the legacy store's first-write policy.
/// Execution/identity qualification retains the documented legacy limits.
pub fn map_pool_creations_impl(block: &Block) -> pb::Pools {
    let mut seen = HashSet::new();
    pb::Pools {
        pools: dex_pools::project(block)
            .into_iter()
            .filter(|entry| seen.insert(entry.address.clone()))
            .map(|entry| pb::PoolCreated {
                address: hex::encode(entry.address),
                pool: Some(pb::Pool {
                    tokens: entry.tokens,
                    factory: entry.factory,
                }),
            })
            .collect(),
    }
}

/// Convenience composition for native/wrapper parity checks. The manifest
/// routes the decoded Pools message to the two storage adapters independently.
pub fn map_pool_entries_impl(block: &Block) -> Result<SinkEntries, Error> {
    pool_entries(&map_pool_creations_impl(block))
}

/// Validate an already decoded batch before writing any entry. Addresses are
/// 20 bytes for contracts or 32 bytes for Uniswap V4 pool IDs; leading zeros
/// remain significant. Duplicate identities retain their first payload.
fn records(pools: &pb::Pools) -> Result<Vec<(&str, Vec<u8>, &pb::Pool)>, Error> {
    let mut seen = HashSet::new();
    let mut records = Vec::with_capacity(pools.pools.len());
    for entry in &pools.pools {
        let address = hex::decode(&entry.address).map_err(|_| Error::msg("invalid pool key: expected lowercase hex"))?;
        if !matches!(address.len(), 20 | 32) || hex::encode(&address) != entry.address {
            return Err(Error::msg("invalid pool key: expected a canonical 20-byte address or 32-byte pool ID"));
        }
        let pool = entry.pool.as_ref().ok_or_else(|| Error::msg("missing pool payload"))?;
        if seen.insert(entry.address.as_str()) {
            records.push((entry.address.as_str(), address, pool));
        }
    }
    Ok(records)
}

/// Hosted Store: raw pool keys, unchanged Pool bytes, first-write semantics.
/// Empty blocks still return if_not_exist=true so ingestion can advance.
pub fn pool_entries(pools: &pb::Pools) -> Result<SinkEntries, Error> {
    Ok(SinkEntries {
        entries: records(pools)?
            .into_iter()
            .map(|(_, address, pool)| Entry {
                key: Some(Key { bytes: address }),
                value: Some(Any {
                    type_url: POOL_TYPE_URL.to_owned(),
                    value: pool.encode_to_vec(),
                }),
            })
            .collect(),
        if_not_exist: true,
    })
}

/// Legacy branch: preserve canonical hex keys, Pool bytes and ordinal zero.
/// This immutable registry does not establish ordering within a block.
pub fn store_pools_impl<S: StoreSetIfNotExists<pb::Pool>>(pools: &pb::Pools, store: &S) -> Result<(), Error> {
    for (key, _, pool) in records(pools)? {
        store.set_if_not_exists(0, key, pool);
    }
    Ok(())
}

/// Preserve the existing public Create-only delta stream and wire schema.
pub fn map_pools_impl(deltas: Deltas<DeltaProto<pb::Pool>>) -> pb::Pools {
    pb::Pools {
        pools: deltas
            .deltas
            .into_iter()
            .filter(|delta| delta.operation == Operation::Create)
            .map(|delta| pb::PoolCreated {
                address: delta.key,
                pool: Some(delta.new_value),
            })
            .collect(),
    }
}

#[cfg(target_arch = "wasm32")]
mod handlers {
    use super::*;
    use substreams::store::{StoreNew, StoreSetIfNotExistsProto};

    #[substreams::handlers::map]
    fn map_pool_creations(block: Block) -> Result<pb::Pools, Error> {
        Ok(map_pool_creations_impl(&block))
    }

    #[substreams::handlers::map]
    fn map_pool_entries(pools: pb::Pools) -> Result<SinkEntries, Error> {
        pool_entries(&pools)
    }

    #[substreams::handlers::store]
    fn store_pools(pools: pb::Pools, store: StoreSetIfNotExistsProto<pb::Pool>) {
        store_pools_impl(&pools, &store).expect("valid decoded pool records");
    }

    #[substreams::handlers::map]
    fn map_pools(deltas: Deltas<DeltaProto<pb::Pool>>) -> Result<pb::Pools, Error> {
        Ok(map_pools_impl(deltas))
    }
}
