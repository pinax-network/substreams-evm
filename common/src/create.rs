use substreams_ethereum::pb::eth::v2::{Log, TransactionTrace};

/// Trait for creating a log entry from raw log data and an event
pub trait CreateLog<E> {
    fn create_log(log: &Log, event: E) -> Self;
}

/// Trait for creating a transaction from a transaction trace
pub trait CreateTransaction {
    fn create_transaction(trx: &TransactionTrace) -> Self;
}

macro_rules! impl_create_log_and_transaction {
    ($module:path) => {
        use $module as pb;

        impl CreateLog<pb::log::Log> for pb::Log {
            fn create_log(log: &Log, event: pb::log::Log) -> Self {
                Self {
                    address: log.address.to_vec(),
                    ordinal: log.ordinal,
                    topics: log.topics.iter().map(|t| t.to_vec()).collect(),
                    data: log.data.to_vec(),
                    log: Some(event),
                }
            }
        }

        impl CreateTransaction for pb::Transaction {
            fn create_transaction(trx: &TransactionTrace) -> Self {
                let gas_price = trx.clone().gas_price.unwrap_or_default().with_decimal(0).to_string();
                let value = trx.clone().value.unwrap_or_default().with_decimal(0);
                let to = if trx.to.is_empty() { None } else { Some(trx.to.to_vec()) };
                Self {
                    from: trx.from.to_vec(),
                    to,
                    hash: trx.hash.to_vec(),
                    nonce: trx.nonce,
                    gas_price,
                    gas_limit: trx.gas_limit,
                    gas_used: trx.receipt().receipt.cumulative_gas_used,
                    value: value.to_string(),
                    logs: vec![],
                }
            }
        }
    };
}

mod erc1155_impl {
    use super::*;
    impl_create_log_and_transaction!(proto::pb::erc1155::v1);
}
