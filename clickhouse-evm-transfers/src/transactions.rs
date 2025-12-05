use common::{bytes_to_hex, bytes_to_string, Encoding};
use proto::pb::evm::transfers::v1 as pb;

pub fn set_template_tx(encoding: &Encoding, tx: &pb::Transaction, tx_index: usize, row: &mut substreams_database_change::tables::Row) {
    let tx_to = match tx.to.as_ref() {
        Some(addr) => bytes_to_string(addr, encoding),
        None => "".to_string(),
    };
    row.set("tx_index", tx_index as u32);
    row.set("tx_hash", bytes_to_hex(&tx.hash));
    row.set("tx_from", bytes_to_string(&tx.from, encoding));
    row.set("tx_to", tx_to);
    row.set("tx_nonce", tx.nonce);
    row.set("tx_gas_price", &tx.gas_price);
    row.set("tx_gas_limit", tx.gas_limit);
    row.set("tx_gas_used", tx.gas_used);
    row.set("tx_value", &tx.value);
}
