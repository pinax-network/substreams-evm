use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::{aerodrome, balancer, bancor, cow, curvefi, dca_dot_fun, dodo, kyber_elastic, sunpump, traderjoe, uniswap, woofi};
use substreams::pb::substreams::Clock;

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
    ($log_type:ty, $call_type:ty, $call_type_enum:ty) => {
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

// SunPump
impl_log_traits!(sunpump::v1::Log, sunpump::v1::Call, sunpump::v1::CallType);

// Uniswap V1
impl_log_traits!(uniswap::v1::Log, uniswap::v1::Call, uniswap::v1::CallType);

// Uniswap V2
impl_log_traits!(uniswap::v2::Log, uniswap::v2::Call, uniswap::v2::CallType);

// Uniswap V3
impl_log_traits!(uniswap::v3::Log, uniswap::v3::Call, uniswap::v3::CallType);

// Uniswap V4
impl_log_traits!(uniswap::v4::Log, uniswap::v4::Call, uniswap::v4::CallType);

// Balancer
impl_log_traits!(balancer::v1::Log, balancer::v1::Call, balancer::v1::CallType);

// Bancor
impl_log_traits!(bancor::v1::Log, bancor::v1::Call, bancor::v1::CallType);

// CoW Protocol
impl_log_traits!(cow::v1::Log, cow::v1::Call, cow::v1::CallType);

// Curve.fi
impl_log_traits!(curvefi::v1::Log, curvefi::v1::Call, curvefi::v1::CallType);

// New DEX protocols
impl_log_traits!(aerodrome::v1::Log, aerodrome::v1::Call, aerodrome::v1::CallType);
impl_log_traits!(dodo::v1::Log, dodo::v1::Call, dodo::v1::CallType);
impl_log_traits!(woofi::v1::Log, woofi::v1::Call, woofi::v1::CallType);
impl_log_traits!(traderjoe::v1::Log, traderjoe::v1::Call, traderjoe::v1::CallType);
impl_log_traits!(kyber_elastic::v1::Log, kyber_elastic::v1::Call, kyber_elastic::v1::CallType);
impl_log_traits!(dca_dot_fun::v1::Log, dca_dot_fun::v1::Call, dca_dot_fun::v1::CallType);
