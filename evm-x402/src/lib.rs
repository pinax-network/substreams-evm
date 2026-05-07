use common::{bytes_to_hex, bytes_to_string, handle_encoding_param};
use proto::pb::evm::x402::v1 as pb;
use substreams::errors::Error;
use substreams::pb::substreams::Clock;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables;

#[substreams::handlers::map]
pub fn db_out(params: String, clock: Clock, events: pb::Events) -> Result<DatabaseChanges, Error> {
    let encoding = handle_encoding_param(&params);
    let mut tables = Tables::new();

    let seconds = clock.timestamp.as_ref().map(|ts| ts.seconds).unwrap_or_default();

    for transaction in events.transactions {
        for (log_index, log) in transaction.logs.into_iter().enumerate() {
            let Some(pb::log::Log::Payment(payment)) = log.log else {
                continue;
            };

            let key = [
                ("block_num", clock.number.to_string()),
                ("transaction_index", transaction.index.to_string()),
                ("log_index", log_index.to_string()),
            ];
            let row = tables.create_row("x402_payments", key);
            row.set("block_num", clock.number);
            row.set("block_hash", format!("0x{}", clock.id));
            row.set("timestamp", seconds);
            row.set("minute", seconds / 60);
            row.set("transaction_hash", bytes_to_hex(&transaction.hash));
            row.set("transaction_index", transaction.index);
            row.set("transaction_from", bytes_to_string(&transaction.from, &encoding));
            row.set(
                "transaction_to",
                transaction.to.as_ref().map(|address| bytes_to_string(address, &encoding)).unwrap_or_default(),
            );
            row.set("log_index", log_index as u32);
            row.set("log_block_index", log.block_index);
            row.set("ordinal", log.ordinal);
            row.set("call_index", log.call.as_ref().map(|call| call.index).unwrap_or_default());
            row.set("asset", bytes_to_string(&payment.asset, &encoding));
            row.set("payer", bytes_to_string(&payment.payer, &encoding));
            row.set("recipient", bytes_to_string(&payment.recipient, &encoding));
            row.set("facilitator", bytes_to_string(&payment.facilitator, &encoding));
            row.set("amount", payment.amount);
            row.set("nonce", bytes_to_hex(&payment.nonce));
            row.set(
                "transfer_method",
                pb::TransferMethod::try_from(payment.transfer_method).unwrap_or_default().as_str_name(),
            );
            row.set(
                "settlement_source",
                pb::SettlementSource::try_from(payment.settlement_source).unwrap_or_default().as_str_name(),
            );
            row.set("scheme", payment.scheme);
            row.set("valid_after", payment.valid_after.unwrap_or_default());
            row.set("valid_before", payment.valid_before.unwrap_or_default());
            row.set("facilitator_allowlist_matched", payment.facilitator_allowlist_matched);
            row.set("confidence", payment.confidence);
        }
    }

    Ok(tables.to_database_changes())
}
