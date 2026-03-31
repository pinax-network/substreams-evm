use proto::pb::dex::swaps::v1 as pb;
use substreams_ethereum::pb::eth::v2::Block;

#[substreams::handlers::map]
pub fn map_events(_block: Block) -> Result<pb::Events, substreams::errors::Error> {
    Ok(pb::Events::default())
}
