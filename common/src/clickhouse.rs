use crate::{bytes_to_string, Encoding};
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

pub fn set_clock(clock: &Clock, row: &mut Row) {
    row.set("block_num", clock.number.to_string())
        .set("block_hash", format!("0x{}", &clock.id))
        .set("timestamp", clock.timestamp.as_ref().expect("missing timestamp").seconds.to_string());
}

pub fn set_ordering(index: u64, ordinal: Option<u64>, block_index: Option<u32>, row: &mut Row) {
    row.set("log_index", index)
        .set("log_ordinal", ordinal.unwrap_or(0))
        .set("log_block_index", block_index.unwrap_or_default());
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
