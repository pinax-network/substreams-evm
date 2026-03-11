use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::erc20::tokens::v1 as tokens_pb;
use proto::pb::erc20::transfers::v1 as pb;

pub fn set_template_log(encoding: &Encoding, log: &impl LogAddress, log_index: usize, row: &mut substreams_database_change::tables::Row) {
    row.set("log_index", log_index as u32);
    row.set("log_block_index", log.get_block_index());
    row.set("log_address", bytes_to_string(log.get_address(), encoding));
    row.set("log_ordinal", log.get_ordinal());
    row.set("log_topics", {
        let topics: Vec<String> = log.get_topics().iter().map(|topic| bytes_to_hex(topic)).collect();
        topics.join(",")
    });
    row.set("log_data", bytes_to_hex(log.get_data()));
}

pub fn set_template_call(encoding: &Encoding, call: &impl CallInfo, row: &mut substreams_database_change::tables::Row) {
    row.set("call_caller", bytes_to_string(call.get_call_caller(), encoding));
    row.set("call_index", call.get_call_index());
    row.set("call_begin_ordinal", call.get_call_begin_ordinal());
    row.set("call_end_ordinal", call.get_call_end_ordinal());
    row.set("call_address", bytes_to_string(call.get_call_address(), encoding));
    row.set("call_value", call.get_call_value());
    row.set("call_gas_consumed", call.get_call_gas_consumed());
    row.set("call_gas_limit", call.get_call_gas_limit());
    row.set("call_depth", call.get_call_depth());
    row.set("call_parent_index", call.get_call_parent_index());
    row.set("call_type", call.get_call_type());
}

// Trait to abstract over different log types
pub trait LogAddress {
    fn get_address(&self) -> &Vec<u8>;
    fn get_block_index(&self) -> u32;
    fn get_ordinal(&self) -> u64;
    fn get_topics(&self) -> &Vec<Vec<u8>>;
    fn get_data(&self) -> &Vec<u8>;
}

// Trait to abstract over call metadata from different log types
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

macro_rules! impl_log_traits {
    ($log_type:ty, $call_type_enum:ty) => {
        impl LogAddress for $log_type {
            fn get_address(&self) -> &Vec<u8> {
                &self.address
            }
            fn get_block_index(&self) -> u32 {
                self.block_index
            }
            fn get_ordinal(&self) -> u64 {
                self.ordinal
            }
            fn get_topics(&self) -> &Vec<Vec<u8>> {
                &self.topics
            }
            fn get_data(&self) -> &Vec<u8> {
                &self.data
            }
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
                self.call.as_ref()
                    .map(|c| <$call_type_enum>::try_from(c.call_type).unwrap_or_default().as_str_name())
                    .unwrap_or(<$call_type_enum>::default().as_str_name())
            }
        }
    };
}

impl_log_traits!(pb::Log, pb::CallType);
impl_log_traits!(tokens_pb::Log, tokens_pb::CallType);

#[cfg(test)]
mod tests {
    use super::{CallInfo, LogAddress, set_template_call, set_template_log};
    use common::Encoding;
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
