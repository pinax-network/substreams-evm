use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use proto::pb::contracts::v1 as pb;
use substreams::Hex;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;

fn bytes_to_hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        String::new()
    } else {
        format!("0x{}", Hex::encode(bytes))
    }
}

#[substreams::handlers::map]
pub fn db_out(clock: Clock, events: pb::Events) -> Result<DatabaseChanges, Error> {
    let mut tables = Tables::new();
    let block_hash = format!("0x{}", clock.id);
    let block_number = clock.number;
    let timestamp = clock.timestamp.as_ref().expect("missing block timestamp");

    for (tx_index, transaction) in events.transactions.iter().enumerate() {
        let transaction_hash = bytes_to_hex(&transaction.hash);
        let tx_to = transaction
            .to
            .as_ref()
            .map(|addr| bytes_to_hex(addr))
            .unwrap_or_default();

        for contract in &transaction.contracts {
            let address = bytes_to_hex(&contract.address);
            let factory = contract
                .factory
                .as_ref()
                .map(|addr| bytes_to_hex(addr))
                .unwrap_or_default();

            tables
                .create_row(
                    "contracts",
                    [
                        ("address", address.as_str()),
                        ("block_hash", block_hash.as_str()),
                        ("transaction_index", &tx_index.to_string()),
                        ("ordinal", &contract.ordinal.to_string()),
                    ],
                )
                .set("block_num", block_number)
                .set("block_hash", &block_hash)
                .set("timestamp", timestamp.seconds)
                .set("transaction_hash", transaction_hash.as_str())
                .set("transaction_index", tx_index as u32)
                .set("ordinal", contract.ordinal)
                .set("address", &address)
                .set("from", bytes_to_hex(&contract.from))
                .set("to", tx_to.as_str())
                .set("deployer", bytes_to_hex(&contract.deployer))
                .set("factory", factory.as_str())
                .set("code", bytes_to_hex(&contract.code))
                .set("code_hash", bytes_to_hex(&contract.code_hash))
                .set("input", bytes_to_hex(&contract.input));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_hex_empty() {
        assert_eq!(bytes_to_hex(&[]), String::new());
    }

    #[test]
    fn test_bytes_to_hex_address() {
        let bytes: Vec<u8> = Hex::decode("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        assert_eq!(
            bytes_to_hex(&bytes),
            "0xd8da6bf26964af9d7eed9e03e53415d37aa96045"
        );
    }

    #[test]
    fn test_bytes_to_hex_single_byte() {
        assert_eq!(bytes_to_hex(&[0xff]), "0xff");
    }
}
