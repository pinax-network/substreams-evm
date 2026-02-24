use crate::{bytes_to_string, Encoding};
use substreams::pb::substreams::Clock;
use substreams::Hex;
use substreams_database_change::tables::Row;

pub type Address = Vec<u8>;
pub type Hash = Vec<u8>;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", Hex::encode(bytes))
}

pub fn to_global_sequence(clock: &Clock, index: u64) -> u64 {
    (clock.number << 32) + index
}

pub fn common_key(clock: &Clock, index: u64) -> [(&'static str, String); 3] {
    let seconds = clock.timestamp.as_ref().expect("clock.timestamp is required").seconds;
    [
        ("timestamp", seconds.to_string()),
        ("block_num", clock.number.to_string()),
        ("index", index.to_string()),
    ]
}

pub fn set_clock(clock: &Clock, row: &mut Row) {
    row.set("block_num", clock.number.to_string())
        .set("block_hash", format!("0x{}", &clock.id))
        .set("timestamp", clock.timestamp.as_ref().expect("missing timestamp").seconds.to_string());
}

pub fn set_ordering(index: u64, ordinal: Option<u64>, clock: &Clock, row: &mut Row) {
    row.set("index", index)
        .set("ordinal", ordinal.unwrap_or(0))
        .set("global_sequence", to_global_sequence(clock, index));
}

pub fn set_bytes(bytes: Option<Hash>, name: &str, row: &mut Row) {
    match bytes {
        Some(data) => row.set(name, bytes_to_hex(&data)),
        None => row.set(name, "".to_string()),
    };
}

fn set_address(bytes: Option<Address>, name: &str, encoding: &Encoding, row: &mut Row) {
    match bytes {
        Some(data) => row.set(name, bytes_to_string(&data, encoding)),
        None => row.set(name, "".to_string()),
    };
}

pub fn set_log(clock: &Clock, index: u64, tx_hash: Hash, contract: Address, ordinal: u64, caller: Option<Address>, encoding: &Encoding, row: &mut Row) {
    set_bytes(Some(tx_hash), "tx_hash", row);
    set_address(Some(contract), "contract", encoding, row);
    set_address(caller, "caller", encoding, row);
    set_ordering(index, Some(ordinal), clock, row);
    set_clock(clock, row);
}
