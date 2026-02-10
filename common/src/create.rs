use substreams_ethereum::pb::eth::v2::{Call, Log, TransactionTrace};

/// Trait for creating a log entry from raw log data and an event
pub trait CreateLog<E> {
    fn create_log(log: &Log, event: E) -> Self;
    fn create_log_with_call(log: &Log, event: E, call: Option<&Call>) -> Self;
}

/// Trait for creating a transaction from a transaction trace
pub trait CreateTransaction {
    fn create_transaction(trx: &TransactionTrace) -> Self;
}

// Macro for modules WITHOUT call metadata fields in Log
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

            fn create_log_with_call(log: &Log, event: pb::log::Log, _call: Option<&Call>) -> Self {
                // Ignore call metadata for modules that don't support it
                Self::create_log(log, event)
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

// Macro for modules WITH call metadata fields in Log (e.g., erc20-transfers, dex)
macro_rules! impl_create_log_with_call_metadata {
    ($module:path) => {
        use $module as pb;

        impl CreateLog<pb::log::Log> for pb::Log {
            fn create_log(log: &Log, event: pb::log::Log) -> Self {
                Self::create_log_with_call(log, event, None)
            }

            fn create_log_with_call(log: &Log, event: pb::log::Log, call: Option<&Call>) -> Self {
                Self {
                    address: log.address.to_vec(),
                    ordinal: log.ordinal,
                    topics: log.topics.iter().map(|t| t.to_vec()).collect(),
                    data: log.data.to_vec(),
                    call: call.map(|c| pb::Call {
                        caller: c.caller.to_vec(),
                        index: c.index,
                        depth: c.depth,
                        call_type: c.call_type,
                    }),
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

mod uniswap_v1_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::uniswap::v1);
}

mod uniswap_v2_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::uniswap::v2);
}

mod uniswap_v3_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::uniswap::v3);
}

mod uniswap_v4_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::uniswap::v4);
}

mod balancer_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::balancer::v1);
}

mod bancor_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::bancor::v1);
}

mod cow_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::cow::v1);
}

mod curvefi_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::curvefi::v1);
}

mod sunpump_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::sunpump::v1);
}

mod erc20_transfers_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::erc20::transfers::v1);
}

mod steth_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::steth::v1);
}

mod erc20_tokens_impl {
    use super::*;
    impl_create_log_with_call_metadata!(proto::pb::erc20::tokens::v1);
}