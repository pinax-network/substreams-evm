use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::{justswap, sunpump, sunswap, uniswap};
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

pub fn set_template_log(encoding: &Encoding, log: &impl LogAddress, log_index: usize, row: &mut substreams_database_change::tables::Row) {
    row.set("log_index", log_index as u32);
    row.set("log_address", bytes_to_string(log.get_address(), encoding));
    row.set("log_ordinal", log.get_ordinal());
    row.set("log_topics", {
        let topics: Vec<String> = log.get_topics().iter().map(|topic| bytes_to_hex(topic)).collect();
        topics.join(",")
    });
    row.set("log_data", bytes_to_hex(log.get_data()));
}

// Trait to abstract over different log types
pub trait LogAddress {
    fn get_address(&self) -> &Vec<u8>;
    fn get_ordinal(&self) -> u64;
    fn get_topics(&self) -> &Vec<Vec<u8>>;
    fn get_data(&self) -> &Vec<u8>;
}

// JustSwap
impl LogAddress for justswap::v1::Log {
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

// SunSwap
impl LogAddress for sunswap::v1::Log {
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

// SunPump
impl LogAddress for sunpump::v1::Log {
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

// Uniswap V1
impl LogAddress for uniswap::v1::Log {
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

// Uniswap V2
impl LogAddress for uniswap::v2::Log {
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


// Uniswap V3
impl LogAddress for uniswap::v3::Log {
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

// Uniswap V4
impl LogAddress for uniswap::v4::Log {
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
