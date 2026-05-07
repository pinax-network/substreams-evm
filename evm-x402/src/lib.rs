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

    for transaction in events.transactions.iter() {
        for (log_index, log) in transaction.logs.iter().enumerate() {
            let Some(pb::log::Log::Payment(payment)) = &log.log else {
                continue;
            };

            let key = [("timestamp", seconds.to_string()), ("block_num", clock.number.to_string())];
            let row = tables.create_row("x402_payments", key);

            row.set("block_num", clock.number);
            row.set("block_hash", format!("0x{}", clock.id));
            row.set("timestamp", seconds);
            row.set("tx_index", transaction.index);
            row.set("tx_hash", bytes_to_hex(&transaction.hash));
            row.set("tx_from", bytes_to_string(&transaction.from, &encoding));
            row.set(
                "tx_to",
                transaction.to.as_ref().map(|address| bytes_to_string(address, &encoding)).unwrap_or_default(),
            );
            row.set("tx_nonce", transaction.nonce);
            row.set("tx_gas_price", &transaction.gas_price);
            row.set("tx_gas_limit", transaction.gas_limit);
            row.set("tx_gas_used", transaction.gas_used);
            row.set("tx_value", &transaction.value);

            row.set("log_index", log_index as u32);
            row.set("log_block_index", log.block_index);
            row.set("log_address", bytes_to_string(&log.address, &encoding));
            row.set("log_ordinal", log.ordinal);
            row.set("log_topics", log.topics.iter().map(|topic| bytes_to_hex(topic)).collect::<Vec<_>>().join(","));
            row.set("log_data", bytes_to_hex(&log.data));

            let call = log.call.as_ref();
            row.set("call_caller", call.map(|call| bytes_to_string(&call.caller, &encoding)).unwrap_or_default());
            row.set("call_index", call.map(|call| call.index).unwrap_or_default());
            row.set("call_begin_ordinal", call.map(|call| call.begin_ordinal).unwrap_or_default());
            row.set("call_end_ordinal", call.map(|call| call.end_ordinal).unwrap_or_default());
            row.set("call_address", call.map(|call| bytes_to_string(&call.address, &encoding)).unwrap_or_default());
            row.set("call_value", call.map(|call| call.value.as_str()).unwrap_or_default());
            row.set("call_gas_consumed", call.map(|call| call.gas_consumed).unwrap_or_default());
            row.set("call_gas_limit", call.map(|call| call.gas_limit).unwrap_or_default());
            row.set("call_depth", call.map(|call| call.depth).unwrap_or_default());
            row.set("call_parent_index", call.map(|call| call.parent_index).unwrap_or_default());
            row.set(
                "call_type",
                call.map(|call| pb::CallType::try_from(call.call_type).unwrap_or_default().as_str_name())
                    .unwrap_or_default(),
            );

            row.set("asset", bytes_to_string(&payment.asset, &encoding));
            row.set("payer", bytes_to_string(&payment.payer, &encoding));
            row.set("recipient", bytes_to_string(&payment.recipient, &encoding));
            row.set("facilitator", bytes_to_string(&payment.facilitator, &encoding));
            row.set("amount", &payment.amount);
            row.set("nonce", bytes_to_hex(&payment.nonce));
            row.set(
                "transfer_method",
                pb::TransferMethod::try_from(payment.transfer_method).unwrap_or_default().as_str_name(),
            );
            row.set(
                "settlement_source",
                pb::SettlementSource::try_from(payment.settlement_source).unwrap_or_default().as_str_name(),
            );
            row.set("scheme", &payment.scheme);
            row.set("valid_after", payment.valid_after.as_deref().unwrap_or("0"));
            row.set("valid_before", payment.valid_before.as_deref().unwrap_or("0"));
            row.set("facilitator_allowlist_matched", payment.facilitator_allowlist_matched);
            row.set("confidence", &payment.confidence);
        }
    }

    if !tables.tables.is_empty() {
        let row = tables.create_row("blocks", [("block_num", clock.number.to_string())]);
        row.set("block_num", clock.number);
        row.set("block_hash", format!("0x{}", clock.id));
        row.set("timestamp", seconds);
    }

    Ok(tables.to_database_changes())
}
