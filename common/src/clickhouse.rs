use crate::{bytes_to_string, Encoding};
use proto::pb::{aerodrome, balancer, bancor, cow, curvefi, dex, dodo, erc1155, erc721, kyber_elastic, sunpump, traderjoe, uniswap, woofi};
use substreams::pb::substreams::Clock;
use substreams::Hex;
use substreams_database_change::tables::Row;

pub type Address = Vec<u8>;
pub type Hash = Vec<u8>;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", Hex::encode(bytes))
}

pub fn common_key(clock: &Clock, index: u64) -> [(&'static str, String); 5] {
    let seconds = clock.timestamp.as_ref().expect("clock.timestamp is required").seconds;
    [
        ("minute", (seconds / 60).to_string()),
        ("timestamp", seconds.to_string()),
        ("block_num", clock.number.to_string()),
        ("block_hash", format!("0x{}", &clock.id)),
        ("log_index", index.to_string()),
    ]
}

pub fn log_key(clock: &Clock, tx_index: usize, log_index: usize) -> [(&'static str, String); 6] {
    let seconds = clock.timestamp.as_ref().expect("clock.timestamp is required").seconds;
    [
        ("minute", (seconds / 60).to_string()),
        ("timestamp", seconds.to_string()),
        ("block_num", clock.number.to_string()),
        ("tx_index", tx_index.to_string()),
        ("log_index", log_index.to_string()),
        ("block_hash", format!("0x{}", &clock.id)),
    ]
}

pub fn set_clock(clock: &Clock, row: &mut Row) {
    row.set("block_num", clock.number.to_string())
        .set("block_hash", format!("0x{}", &clock.id))
        .set("timestamp", clock.timestamp.as_ref().expect("missing timestamp").seconds.to_string())
        .set("minute", clock.timestamp.as_ref().expect("missing timestamp").seconds / 60);
}

pub fn set_template_tx(encoding: &Encoding, tx: &impl TxTemplate, tx_index: usize, row: &mut Row) {
    let tx_to = match tx.get_to() {
        Some(addr) => bytes_to_string(addr, encoding),
        None => "".to_string(),
    };
    row.set("tx_index", tx_index as u32)
        .set("tx_hash", bytes_to_hex(tx.get_hash()))
        .set("tx_from", bytes_to_string(tx.get_from(), encoding))
        .set("tx_to", tx_to)
        .set("tx_nonce", tx.get_nonce())
        .set("tx_gas_price", tx.get_gas_price())
        .set("tx_gas_limit", tx.get_gas_limit())
        .set("tx_gas_used", tx.get_gas_used())
        .set("tx_value", tx.get_value());
}

pub fn set_template_log(encoding: &Encoding, log: &impl LogAddress, log_index: usize, row: &mut Row) {
    row.set("log_index", log_index as u32)
        .set("log_block_index", log.get_block_index())
        .set("log_address", bytes_to_string(log.get_address(), encoding))
        .set("log_ordinal", log.get_ordinal())
        .set("log_topics", {
            let topics: Vec<String> = log.get_topics().iter().map(|topic| bytes_to_hex(topic)).collect();
            topics.join(",")
        })
        .set("log_data", bytes_to_hex(log.get_data()));
}

pub fn set_template_call(encoding: &Encoding, call: &impl CallInfo, row: &mut Row) {
    row.set("call_caller", bytes_to_string(call.get_call_caller(), encoding))
        .set("call_index", call.get_call_index())
        .set("call_begin_ordinal", call.get_call_begin_ordinal())
        .set("call_end_ordinal", call.get_call_end_ordinal())
        .set("call_address", bytes_to_string(call.get_call_address(), encoding))
        .set("call_value", call.get_call_value())
        .set("call_gas_consumed", call.get_call_gas_consumed())
        .set("call_gas_limit", call.get_call_gas_limit())
        .set("call_depth", call.get_call_depth())
        .set("call_parent_index", call.get_call_parent_index())
        .set("call_type", call.get_call_type());
}

pub fn set_ordering(index: u64, ordinal: Option<u64>, block_index: Option<u32>, row: &mut Row) {
    row.set("log_index", index)
        .set("log_ordinal", ordinal.unwrap_or(0))
        .set("log_block_index", block_index.unwrap_or_default());
}

pub trait TxTemplate {
    fn get_hash(&self) -> &Vec<u8>;
    fn get_from(&self) -> &Vec<u8>;
    fn get_to(&self) -> &Option<Vec<u8>>;
    fn get_nonce(&self) -> u64;
    fn get_gas_price(&self) -> &str;
    fn get_gas_limit(&self) -> u64;
    fn get_gas_used(&self) -> u64;
    fn get_value(&self) -> &str;
}

pub trait LogAddress {
    fn get_address(&self) -> &Vec<u8>;
    fn get_block_index(&self) -> u32;
    fn get_ordinal(&self) -> u64;
    fn get_topics(&self) -> &Vec<Vec<u8>>;
    fn get_data(&self) -> &Vec<u8>;
}

pub trait CallInfo {
    fn get_call_caller(&self) -> &[u8];
    fn get_call_index(&self) -> u32;
    fn get_call_begin_ordinal(&self) -> u64;
    fn get_call_end_ordinal(&self) -> u64;
    fn get_call_address(&self) -> &[u8];
    fn get_call_value(&self) -> &str;
    fn get_call_gas_consumed(&self) -> u64;
    fn get_call_gas_limit(&self) -> u64;
    fn get_call_depth(&self) -> u32;
    fn get_call_parent_index(&self) -> u32;
    fn get_call_type(&self) -> &str;
}

pub fn set_bytes(bytes: Option<Hash>, name: &str, row: &mut Row) {
    match bytes {
        Some(data) => row.set(name, bytes_to_hex(&data)),
        None => row.set(name, "".to_string()),
    };
}

fn set_address(address: Option<&[u8]>, name: &str, encoding: &Encoding, row: &mut Row) {
    match address {
        Some(data) => row.set(name, bytes_to_string(&data, encoding)),
        None => row.set(name, "".to_string()),
    };
}

#[derive(Clone, Copy, Default)]
pub struct CallMetadata<'a> {
    pub caller: Option<&'a [u8]>,
    pub index: Option<u32>,
    pub begin_ordinal: Option<u64>,
    pub end_ordinal: Option<u64>,
    pub address: Option<&'a [u8]>,
    pub value: Option<&'a str>,
    pub gas_consumed: Option<u64>,
    pub gas_limit: Option<u64>,
    pub depth: Option<u32>,
    pub parent_index: Option<u32>,
    pub call_type: Option<&'a str>,
}

pub fn set_log(
    clock: &Clock,
    index: u64,
    tx_hash: Hash,
    contract: Address,
    ordinal: u64,
    block_index: Option<u32>,
    call: Option<CallMetadata<'_>>,
    encoding: &Encoding,
    row: &mut Row,
) {
    let call = call.unwrap_or_default();
    set_bytes(Some(tx_hash), "tx_hash", row);
    set_address(Some(contract.as_slice()), "log_address", encoding, row);
    set_address(call.caller, "call_caller", encoding, row);
    set_address(call.address, "call_address", encoding, row);
    row.set("call_index", call.index.unwrap_or_default())
        .set("call_begin_ordinal", call.begin_ordinal.unwrap_or_default())
        .set("call_end_ordinal", call.end_ordinal.unwrap_or_default())
        .set("call_value", call.value.unwrap_or_default())
        .set("call_gas_consumed", call.gas_consumed.unwrap_or_default())
        .set("call_gas_limit", call.gas_limit.unwrap_or_default())
        .set("call_depth", call.depth.unwrap_or_default())
        .set("call_parent_index", call.parent_index.unwrap_or_default())
        .set("call_type", call.call_type.unwrap_or_default());
    set_ordering(index, Some(ordinal), block_index, row);
    set_clock(clock, row);
}

macro_rules! impl_tx_template {
    ($tx_type:ty) => {
        impl TxTemplate for $tx_type {
            fn get_hash(&self) -> &Vec<u8> { &self.hash }
            fn get_from(&self) -> &Vec<u8> { &self.from }
            fn get_to(&self) -> &Option<Vec<u8>> { &self.to }
            fn get_nonce(&self) -> u64 { self.nonce }
            fn get_gas_price(&self) -> &str { &self.gas_price }
            fn get_gas_limit(&self) -> u64 { self.gas_limit }
            fn get_gas_used(&self) -> u64 { self.gas_used }
            fn get_value(&self) -> &str { &self.value }
        }
    };
}

macro_rules! impl_log_traits {
    ($log_type:ty, $call_type_enum:ty) => {
        impl LogAddress for $log_type {
            fn get_address(&self) -> &Vec<u8> { &self.address }
            fn get_block_index(&self) -> u32 { self.block_index }
            fn get_ordinal(&self) -> u64 { self.ordinal }
            fn get_topics(&self) -> &Vec<Vec<u8>> { &self.topics }
            fn get_data(&self) -> &Vec<u8> { &self.data }
        }

        impl CallInfo for $log_type {
            fn get_call_caller(&self) -> &[u8] {
                self.call.as_ref().map(|c| c.caller.as_slice()).unwrap_or_default()
            }
            fn get_call_index(&self) -> u32 {
                self.call.as_ref().map(|c| c.index).unwrap_or_default()
            }
            fn get_call_begin_ordinal(&self) -> u64 {
                self.call.as_ref().map(|c| c.begin_ordinal).unwrap_or_default()
            }
            fn get_call_end_ordinal(&self) -> u64 {
                self.call.as_ref().map(|c| c.end_ordinal).unwrap_or_default()
            }
            fn get_call_address(&self) -> &[u8] {
                self.call.as_ref().map(|c| c.address.as_slice()).unwrap_or_default()
            }
            fn get_call_value(&self) -> &str {
                self.call.as_ref().map(|c| c.value.as_str()).unwrap_or_default()
            }
            fn get_call_gas_consumed(&self) -> u64 {
                self.call.as_ref().map(|c| c.gas_consumed).unwrap_or_default()
            }
            fn get_call_gas_limit(&self) -> u64 {
                self.call.as_ref().map(|c| c.gas_limit).unwrap_or_default()
            }
            fn get_call_depth(&self) -> u32 {
                self.call.as_ref().map(|c| c.depth).unwrap_or_default()
            }
            fn get_call_parent_index(&self) -> u32 {
                self.call.as_ref().map(|c| c.parent_index).unwrap_or_default()
            }
            fn get_call_type(&self) -> &str {
                self.call
                    .as_ref()
                    .map(|c| <$call_type_enum>::try_from(c.call_type).unwrap_or_default().as_str_name())
                    .unwrap_or(<$call_type_enum>::default().as_str_name())
            }
        }
    };
}

impl_tx_template!(erc721::transfers::v1::Transaction);
impl_tx_template!(erc721::tokens::v1::Transaction);
impl_tx_template!(erc1155::v1::Transaction);
impl_tx_template!(sunpump::v1::Transaction);
impl_tx_template!(uniswap::v1::Transaction);
impl_tx_template!(uniswap::v2::Transaction);
impl_tx_template!(uniswap::v3::Transaction);
impl_tx_template!(uniswap::v4::Transaction);
impl_tx_template!(balancer::v1::Transaction);
impl_tx_template!(bancor::v1::Transaction);
impl_tx_template!(cow::v1::Transaction);
impl_tx_template!(curvefi::v1::Transaction);
impl_tx_template!(aerodrome::v1::Transaction);
impl_tx_template!(dodo::v1::Transaction);
impl_tx_template!(woofi::v1::Transaction);
impl_tx_template!(traderjoe::v1::Transaction);
impl_tx_template!(kyber_elastic::v1::Transaction);
impl_tx_template!(dex::swaps::v1::Transaction);

impl_log_traits!(erc721::transfers::v1::Log, erc721::transfers::v1::CallType);
impl_log_traits!(erc721::tokens::v1::Log, erc721::tokens::v1::CallType);
impl_log_traits!(erc1155::v1::Log, erc1155::v1::CallType);
impl_log_traits!(sunpump::v1::Log, sunpump::v1::CallType);
impl_log_traits!(uniswap::v1::Log, uniswap::v1::CallType);
impl_log_traits!(uniswap::v2::Log, uniswap::v2::CallType);
impl_log_traits!(uniswap::v3::Log, uniswap::v3::CallType);
impl_log_traits!(uniswap::v4::Log, uniswap::v4::CallType);
impl_log_traits!(balancer::v1::Log, balancer::v1::CallType);
impl_log_traits!(bancor::v1::Log, bancor::v1::CallType);
impl_log_traits!(cow::v1::Log, cow::v1::CallType);
impl_log_traits!(curvefi::v1::Log, curvefi::v1::CallType);
impl_log_traits!(aerodrome::v1::Log, aerodrome::v1::CallType);
impl_log_traits!(dodo::v1::Log, dodo::v1::CallType);
impl_log_traits!(woofi::v1::Log, woofi::v1::CallType);
impl_log_traits!(traderjoe::v1::Log, traderjoe::v1::CallType);
impl_log_traits!(kyber_elastic::v1::Log, kyber_elastic::v1::CallType);
impl_log_traits!(dex::swaps::v1::Log, dex::swaps::v1::CallType);

#[cfg(test)]
mod tests {
    use super::{CallInfo, LogAddress, set_template_call, set_template_log};
    use crate::Encoding;
    use substreams_database_change::tables::Row;

    struct TestLog;

    impl LogAddress for TestLog {
        fn get_address(&self) -> &Vec<u8> { static ADDRESS: Vec<u8> = Vec::new(); &ADDRESS }
        fn get_block_index(&self) -> u32 { 7 }
        fn get_ordinal(&self) -> u64 { 11 }
        fn get_topics(&self) -> &Vec<Vec<u8>> { static TOPICS: Vec<Vec<u8>> = Vec::new(); &TOPICS }
        fn get_data(&self) -> &Vec<u8> { static DATA: Vec<u8> = Vec::new(); &DATA }
    }

    impl CallInfo for TestLog {
        fn get_call_caller(&self) -> &[u8] { &[0x11] }
        fn get_call_index(&self) -> u32 { 2 }
        fn get_call_begin_ordinal(&self) -> u64 { 3 }
        fn get_call_end_ordinal(&self) -> u64 { 4 }
        fn get_call_address(&self) -> &[u8] { &[0x22] }
        fn get_call_value(&self) -> &str { "5" }
        fn get_call_gas_consumed(&self) -> u64 { 6 }
        fn get_call_gas_limit(&self) -> u64 { 7 }
        fn get_call_depth(&self) -> u32 { 8 }
        fn get_call_parent_index(&self) -> u32 { 9 }
        fn get_call_type(&self) -> &str { "CALL" }
    }

    #[test]
    #[allow(deprecated)]
    fn set_template_log_only_sets_log_fields() {
        let mut row = Row::new();

        set_template_log(&Encoding::Hex, &TestLog, 1, &mut row);

        assert_eq!(row.columns.get("log_index"), Some(&"1".to_string()));
        assert_eq!(row.columns.get("log_block_index"), Some(&"7".to_string()));
        assert_eq!(row.columns.get("log_ordinal"), Some(&"11".to_string()));
        assert!(!row.columns.contains_key("call_index"));
        assert!(!row.columns.contains_key("call_type"));
    }

    #[test]
    #[allow(deprecated)]
    fn set_template_call_only_sets_call_fields() {
        let mut row = Row::new();

        set_template_call(&Encoding::Hex, &TestLog, &mut row);

        assert_eq!(row.columns.get("call_index"), Some(&"2".to_string()));
        assert_eq!(row.columns.get("call_begin_ordinal"), Some(&"3".to_string()));
        assert_eq!(row.columns.get("call_end_ordinal"), Some(&"4".to_string()));
        assert_eq!(row.columns.get("call_value"), Some(&"5".to_string()));
        assert_eq!(row.columns.get("call_gas_consumed"), Some(&"6".to_string()));
        assert_eq!(row.columns.get("call_gas_limit"), Some(&"7".to_string()));
        assert_eq!(row.columns.get("call_depth"), Some(&"8".to_string()));
        assert_eq!(row.columns.get("call_parent_index"), Some(&"9".to_string()));
        assert_eq!(row.columns.get("call_type"), Some(&"CALL".to_string()));
        assert!(!row.columns.contains_key("log_index"));
        assert!(!row.columns.contains_key("log_data"));
    }
}
