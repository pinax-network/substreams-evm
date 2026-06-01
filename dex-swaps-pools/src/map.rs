use proto::pb::dex::foundational_store::v1::{Pool, PoolCreated, Pools};
use substreams::pb::substreams::store_delta::Operation;
use substreams::store::{DeltaProto, Deltas};

/// Surface every pool first seen in this block. Since `store_pools` uses `set_if_not_exists`,
/// only the first write per pool produces a `Create` delta — later blocks never re-emit it.
#[substreams::handlers::map]
pub fn map_pools(deltas: Deltas<DeltaProto<Pool>>) -> Pools {
    let pools = deltas
        .deltas
        .into_iter()
        .filter(|delta| delta.operation == Operation::Create)
        .map(|delta| PoolCreated { address: delta.key, pool: Some(delta.new_value) })
        .collect();
    Pools { pools }
}
