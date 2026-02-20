use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams::Hex;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;
use substreams_ethereum::pb::eth::v2::{Block, CallType};

fn bytes_to_hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        String::new()
    } else {
        format!("0x{}", Hex::encode(bytes))
    }
}

#[substreams::handlers::map]
pub fn db_out(clock: Clock, block: Block) -> Result<DatabaseChanges, Error> {
    let mut tables = Tables::new();
    let block_hash = format!("0x{}", clock.id);
    let block_number = clock.number;
    let timestamp = clock.timestamp.as_ref().expect("missing block timestamp");

    for trace in &block.transaction_traces {
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
                    let address = bytes_to_hex(&code.address);

                    tables
                        .create_row(
                            "contracts",
                            [
                                ("address", address.as_str()),
                                ("block_hash", block_hash.as_str()),
                                ("transaction_index", &trace.index.to_string()),
                                ("ordinal", &code.ordinal.to_string()),
                            ],
                        )
                        .set("block_num", block_number)
                        .set("block_hash", &block_hash)
                        .set("timestamp", timestamp.seconds)
                        .set("transaction_hash", bytes_to_hex(&trace.hash))
                        .set("transaction_index", trace.index)
                        .set("ordinal", code.ordinal)
                        .set("address", &address)
                        .set("from", &from)
                        .set("to", &to)
                        .set("deployer", &deployer)
                        .set("factory", &factory)
                        .set("code", bytes_to_hex(&code.new_code))
                        .set("code_hash", bytes_to_hex(&code.new_hash))
                        .set("input", bytes_to_hex(&call.input));
                }
            }
        }
    }

    // ONLY include blocks if events are present
    if !tables.tables.is_empty() {
        let row = tables.create_row("blocks", [("block_num", clock.number.to_string().as_str())]);
        row.set("block_num", clock.number);
        row.set("block_hash", &block_hash);
        row.set("timestamp", timestamp.seconds);
    }

    Ok(tables.to_database_changes())
}
