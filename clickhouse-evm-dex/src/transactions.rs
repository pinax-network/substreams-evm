use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::{balancer, bancor, cow, curvefi, polymarket, sunpump, uniswap};

pub fn set_template_tx(encoding: &Encoding, tx: &impl TxTemplate, tx_index: usize, row: &mut substreams_database_change::tables::Row) {
    let tx_to = match tx.get_to() {
        Some(addr) => bytes_to_string(addr, encoding),
        None => "".to_string(),
    };
    row.set("tx_index", tx_index as u32);
    row.set("tx_hash", bytes_to_hex(tx.get_hash()));
    row.set("tx_from", bytes_to_string(tx.get_from(), encoding));
    row.set("tx_to", tx_to);
    row.set("tx_nonce", tx.get_nonce());
    row.set("tx_gas_price", tx.get_gas_price());
    row.set("tx_gas_limit", tx.get_gas_limit());
    row.set("tx_gas_used", tx.get_gas_used());
    row.set("tx_value", tx.get_value());
}

// Trait to abstract over different transaction types
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

// SunPump
impl TxTemplate for sunpump::v1::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Uniswap V1
impl TxTemplate for uniswap::v1::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Uniswap V2
impl TxTemplate for uniswap::v2::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Uniswap V3
impl TxTemplate for uniswap::v3::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Uniswap V4
impl TxTemplate for uniswap::v4::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Balancer
impl TxTemplate for balancer::v1::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Bancor
impl TxTemplate for bancor::v1::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// CoW Protocol
impl TxTemplate for cow::v1::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Curve.fi
impl TxTemplate for curvefi::v1::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}

// Polymarket
impl TxTemplate for polymarket::v1::Transaction {
    fn get_hash(&self) -> &Vec<u8> {
        &self.hash
    }
    fn get_from(&self) -> &Vec<u8> {
        &self.from
    }
    fn get_to(&self) -> &Option<Vec<u8>> {
        &self.to
    }
    fn get_nonce(&self) -> u64 {
        self.nonce
    }
    fn get_gas_price(&self) -> &str {
        &self.gas_price
    }
    fn get_gas_limit(&self) -> u64 {
        self.gas_limit
    }
    fn get_gas_used(&self) -> u64 {
        self.gas_used
    }
    fn get_value(&self) -> &str {
        &self.value
    }
}
