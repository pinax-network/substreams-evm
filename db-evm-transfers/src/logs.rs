use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::erc20::tokens::v1 as tokens_pb;
use proto::pb::erc20::transfers::v1 as pb;

pub fn set_template_log(encoding: &Encoding, log: &(impl LogAddress + CallInfo), log_index: usize, row: &mut substreams_database_change::tables::Row) {
    row.set("log_index", log_index as u32);
    row.set("log_address", bytes_to_string(log.get_address(), encoding));
    row.set("log_ordinal", log.get_ordinal());
    row.set("log_topics", {
        let topics: Vec<String> = log.get_topics().iter().map(|topic| bytes_to_hex(topic)).collect();
        topics.join(",")
    });
    row.set("log_data", bytes_to_hex(log.get_data()));

    // Call metadata (defaults to empty when not available)
    row.set("call_caller", bytes_to_string(log.get_call_caller(), encoding));
    row.set("call_index", log.get_call_index());
    row.set("call_depth", log.get_call_depth());
    row.set("call_type", log.get_call_type());
}

// Trait to abstract over different log types
pub trait LogAddress {
    fn get_address(&self) -> &Vec<u8>;
    fn get_ordinal(&self) -> u64;
    fn get_topics(&self) -> &Vec<Vec<u8>>;
    fn get_data(&self) -> &Vec<u8>;
}

// Trait to abstract over call metadata from different log types
pub trait CallInfo {
    fn get_call_caller(&self) -> &[u8];
    fn get_call_index(&self) -> u32;
    fn get_call_depth(&self) -> u32;
    fn get_call_type(&self) -> &str;
}

macro_rules! impl_log_traits {
    ($log_type:ty, $call_type_enum:ty) => {
        impl LogAddress for $log_type {
            fn get_address(&self) -> &Vec<u8> {
                &self.address
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
            fn get_call_depth(&self) -> u32 {
                self.call.as_ref().map(|c| c.depth).unwrap_or_default()
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
