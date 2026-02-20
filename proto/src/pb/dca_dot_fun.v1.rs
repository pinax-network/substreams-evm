// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Events {
    #[prost(message, repeated, tag="1")]
    pub transactions: ::prost::alloc::vec::Vec<Transaction>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Transaction {
    #[prost(bytes="vec", tag="1")]
    pub hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub from: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", optional, tag="3")]
    pub to: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(uint64, tag="5")]
    pub nonce: u64,
    #[prost(string, tag="6")]
    pub gas_price: ::prost::alloc::string::String,
    #[prost(uint64, tag="7")]
    pub gas_limit: u64,
    #[prost(uint64, tag="8")]
    pub gas_used: u64,
    #[prost(string, tag="9")]
    pub value: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="10")]
    pub logs: ::prost::alloc::vec::Vec<Log>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Log {
    #[prost(bytes="vec", tag="1")]
    pub address: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint64, tag="2")]
    pub ordinal: u64,
    #[prost(bytes="vec", repeated, tag="3")]
    pub topics: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes="vec", tag="4")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="5")]
    pub call: ::core::option::Option<Call>,
    #[prost(oneof="log::Log", tags="10, 11, 12")]
    pub log: ::core::option::Option<log::Log>,
}
pub mod log {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Log {
        #[prost(message, tag="10")]
        FillOrder(super::FillOrder),
        #[prost(message, tag="11")]
        CreateOrder(super::CreateOrder),
        #[prost(message, tag="12")]
        CancelOrder(super::CancelOrder),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Call {
    #[prost(bytes="vec", tag="1")]
    pub caller: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag="2")]
    pub index: u32,
    #[prost(uint32, tag="3")]
    pub depth: u32,
    #[prost(enumeration="CallType", tag="4")]
    pub call_type: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FillOrder {
    /// uint256
    #[prost(string, tag="1")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub caller: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub recipient: ::prost::alloc::vec::Vec<u8>,
    /// uint256 (token_in amount)
    #[prost(string, tag="4")]
    pub fill_amount: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="5")]
    pub amount_of_token_out: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="6")]
    pub protocol_fee: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="7")]
    pub token_in_price: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="8")]
    pub token_out_price: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="9")]
    pub scaling_factor: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateOrder {
    /// uint256
    #[prost(string, tag="1")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub creator: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="3")]
    pub recipient: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="4")]
    pub token_in: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="5")]
    pub token_out: ::prost::alloc::vec::Vec<u8>,
    /// uint256
    #[prost(string, tag="6")]
    pub spend_amount: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="7")]
    pub repeats: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="8")]
    pub slippage: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="9")]
    pub freq_interval: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="10")]
    pub scaling_interval: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="11")]
    pub last_run: ::prost::alloc::string::String,
    /// uint256
    #[prost(string, tag="12")]
    pub protocol_fee: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="13")]
    pub vault: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="14")]
    pub stake_asset_in: bool,
    #[prost(bool, tag="15")]
    pub stake_asset_out: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CancelOrder {
    /// uint256
    #[prost(string, tag="1")]
    pub order_id: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub vault: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CallType {
    Unspecified = 0,
    Call = 1,
    Callcode = 2,
    Delegate = 3,
    Static = 4,
    Create = 5,
}
impl CallType {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            CallType::Unspecified => "CALL_TYPE_UNSPECIFIED",
            CallType::Call => "CALL_TYPE_CALL",
            CallType::Callcode => "CALL_TYPE_CALLCODE",
            CallType::Delegate => "CALL_TYPE_DELEGATE",
            CallType::Static => "CALL_TYPE_STATIC",
            CallType::Create => "CALL_TYPE_CREATE",
        }
    }
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CALL_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "CALL_TYPE_CALL" => Some(Self::Call),
            "CALL_TYPE_CALLCODE" => Some(Self::Callcode),
            "CALL_TYPE_DELEGATE" => Some(Self::Delegate),
            "CALL_TYPE_STATIC" => Some(Self::Static),
            "CALL_TYPE_CREATE" => Some(Self::Create),
            _ => None,
        }
    }
}
