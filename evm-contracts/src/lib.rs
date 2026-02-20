use proto::pb::evm::contracts::v1::Events;
use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;

#[substreams::handlers::map]
pub fn db_out(clock: Clock, events: Events) -> Result<DatabaseChanges, Error> {
    let mut tables = Tables::new();

    for contract in events.contract_creations {
        tables
            .create_row(
                "contracts",
                [
                    ("address", contract.address.as_str()),
                    ("block_hash", contract.block_hash.as_str()),
                    ("transaction_index", &contract.transaction_index.to_string()),
                    ("ordinal", &contract.ordinal.to_string()),
                ],
            )
            .set("block_num", contract.block_number)
            .set("block_hash", &contract.block_hash)
            .set("block_date", &contract.block_date)
            .set("timestamp", contract.block_time.unwrap_or_default().seconds)
            .set("transaction_hash", &contract.transaction_hash)
            .set("transaction_index", contract.transaction_index)
            .set("ordinal", contract.ordinal)
            .set("address", &contract.address)
            .set("from", &contract.from)
            .set("to", &contract.to)
            .set("deployer", &contract.deployer)
            .set("factory", contract.factory.unwrap_or_default())
            .set("code", contract.code.unwrap_or_default())
            .set("code_hash", contract.code_hash.unwrap_or_default())
            .set("input", contract.input.unwrap_or_default());
    }

    // ONLY include blocks if events are present
    if !tables.tables.is_empty() {
        let row = tables.create_row("blocks", [("block_num", clock.number.to_string().as_str())]);
        row.set("block_num", clock.number);
        row.set("block_hash", format!("0x{}", clock.id));
        if let Some(timestamp) = &clock.timestamp {
            row.set("timestamp", timestamp.seconds);
        }
    }

    Ok(tables.to_database_changes())
}
