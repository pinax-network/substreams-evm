use proto::pb::evm::contracts::v1 as pb;
use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams_ethereum::pb::eth::v2::{Block, CallType};
use substreams::Hex;

fn bytes_to_hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        String::new()
    } else {
        format!("0x{}", Hex::encode(bytes))
    }
}

fn to_date(clock: &Clock) -> String {
    let timestamp = clock.timestamp.as_ref().expect("missing clock timestamp");
    timestamp.to_string().split('T').next().expect("missing date").to_string()
}

/// Filters the block to only include transaction traces with CREATE calls.
/// This is a performance optimization for Substreams caching.
#[substreams::handlers::map]
fn map_block_index(block: Block) -> Result<Block, Error> {
    let mut indexed_block = Block::default();
    indexed_block.header = block.header;
    indexed_block.code_changes = block.code_changes;

    for trace in block.transaction_traces.into_iter() {
        for call in trace.calls.iter() {
            if call.call_type() == CallType::Create {
                indexed_block.transaction_traces.push(trace);
                break;
            }
        }
    }
    Ok(indexed_block)
}

#[substreams::handlers::map]
fn map_events(clock: Clock, block: Block) -> Result<pb::Events, Error> {
    let mut events = pb::Events::default();
    let block_hash = format!("0x{}", clock.id);
    let block_number = clock.number;
    let timestamp = clock.timestamp.as_ref().expect("missing block timestamp");
    let block_date = to_date(&clock);

    for trace in block.transaction_traces {
        for call in &trace.calls {
            if call.call_type() == CallType::Create {
                for code in &call.code_changes {
                    let from = bytes_to_hex(&trace.from);
                    let to = bytes_to_hex(&trace.to);
                    let factory = if trace.to == code.address {
                        String::new()
                    } else {
                        to.clone()
                    };
                    let deployer = if factory.is_empty() {
                        from.clone()
                    } else {
                        to.clone()
                    };

                    events.contract_creations.push(pb::ContractCreation {
                        block_hash: block_hash.clone(),
                        block_number,
                        block_time: Some(timestamp.clone()),
                        block_date: block_date.clone(),
                        transaction_hash: bytes_to_hex(&trace.hash),
                        transaction_index: trace.index,
                        ordinal: code.ordinal,
                        address: bytes_to_hex(&code.address),
                        from,
                        to,
                        deployer,
                        factory: Some(factory),
                        code: Some(bytes_to_hex(&code.new_code)),
                        code_hash: Some(bytes_to_hex(&code.new_hash)),
                        input: Some(bytes_to_hex(&call.input)),
                    });
                }
            }
        }
    }
    Ok(events)
}
