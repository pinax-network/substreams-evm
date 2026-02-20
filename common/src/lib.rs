pub mod clickhouse;
pub mod create;
pub mod debug;
use sha2::{Digest, Sha256};
use substreams::{hex, log, scalar::BigInt, Hex};

pub type Address = Vec<u8>;
pub type Hash = Vec<u8>;
pub const NULL_ADDRESS: [u8; 20] = hex!("0000000000000000000000000000000000000000");
pub const NULL_HASH: [u8; 32] = hex!("0000000000000000000000000000000000000000000000000000000000000000");

const TRON_VERSION_BYTE: u8 = 0x41; // 'T' addresses on Tron

/// Compute the 4-byte checksum for Base58Check (double SHA-256, first 4 bytes).
fn checksum4(data: &[u8]) -> [u8; 4] {
    let h1 = Sha256::digest(data);
    let h2 = Sha256::digest(&h1);
    let mut out = [0u8; 4];
    out.copy_from_slice(&h2[..4]);
    out
}

/// Convert a 20-byte payload (typically address body) into a Tron Base58Check string.
/// This prepends the Tron version byte (0x41) and appends the checksum.
pub fn tron_base58_from_20(bytes20: &[u8]) -> Result<String, &'static str> {
    if bytes20.len() != 20 {
        return Err("expected exactly 20 bytes");
    }
    let mut data = Vec::with_capacity(21 + 4);
    data.push(TRON_VERSION_BYTE);
    data.extend_from_slice(bytes20);
    let chk = checksum4(&data);
    data.extend_from_slice(&chk);
    Ok(bs58::encode(data).into_string())
}

/// Same as above, but accepts either:
/// - 20 bytes (address body) -> will prepend 0x41
/// - 21 bytes (already includes a leading version byte)
pub fn tron_base58_from_bytes(bytes: &[u8]) -> Result<String, &'static str> {
    match bytes.len() {
        20 => tron_base58_from_20(bytes),
        21 => {
            if bytes[0] != TRON_VERSION_BYTE {
                return Err("unexpected version byte; expected 0x41");
            }
            let mut data = bytes.to_vec();
            let chk = checksum4(&data);
            data.extend_from_slice(&chk);
            Ok(bs58::encode(data).into_string())
        }
        _ => Err("expected 20 or 21 bytes"),
    }
}

#[derive(PartialEq)]
pub enum Encoding {
    Hex,
    TronBase58,
}

pub fn handle_encoding_param(params: &String) -> Encoding {
    // Handle support both EVM & TVM address encoding
    if params.len() > 0 && params != "hex" && params != "tron_base58" {
        panic!("Invalid encoding parameter, supported: hex, tron_base58");
    }
    if params == "tron_base58" {
        return Encoding::TronBase58;
    }
    Encoding::Hex
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format! {"0x{}", Hex::encode(bytes)}.to_string()
}

pub fn bytes_to_string(bytes: &[u8], encoding: &Encoding) -> String {
    if encoding == &Encoding::TronBase58 {
        return tron_base58_from_bytes(bytes).unwrap_or_default();
    }
    bytes_to_hex(bytes)
}

/// Convenience: hex -> Tron Base58Check (accepts '41' + 20 bytes, or just 20 bytes).
pub fn tron_base58_from_hex(hexstr: &str) -> Result<String, String> {
    let raw = hex::decode(hexstr.trim_start_matches("0x")).map_err(|_| "invalid hex".to_string())?;
    tron_base58_from_bytes(&raw).map_err(|e| e.to_string())
}

/// Decode and validate a Tron Base58Check address string. Returns the 21-byte data:
/// [0x41, 20-byte-body]. Validates checksum and version.
pub fn tron_decode_verify(addr: &str) -> Result<[u8; 21], &'static str> {
    let decoded = bs58::decode(addr).into_vec().map_err(|_| "invalid base58")?;
    if decoded.len() != 25 {
        return Err("decoded length must be 25 bytes");
    }
    let (payload, chk_given) = decoded.split_at(21);
    let chk_calc = checksum4(payload);
    if chk_given != chk_calc {
        return Err("checksum mismatch");
    }
    if payload[0] != TRON_VERSION_BYTE {
        return Err("unexpected version byte; expected 0x41");
    }
    let mut out = [0u8; 21];
    out.copy_from_slice(payload);
    Ok(out)
}

// Used to enforce ERC-20 decimals to be between 0 and 255
pub fn bigint_to_u8(bigint: &substreams::scalar::BigInt) -> Option<i32> {
    if bigint.lt(&BigInt::zero()) {
        log::info!("bigint_to_u8: value is negative");
        return None;
    }
    if bigint.gt(&BigInt::from(255)) {
        log::info!("bigint_to_u8: value is greater than 255");
        return None;
    }
    Some(bigint.to_i32())
}

pub fn bigint_to_u64(bigint: &substreams::scalar::BigInt) -> Option<u64> {
    if bigint.lt(&BigInt::zero()) {
        log::info!("bigint_to_u64: value is negative");
        return None;
    }
    if bigint.gt(&BigInt::from(u64::MAX)) {
        log::info!("bigint_to_u64: value is greater than u64::MAX");
        return None;
    }
    Some(bigint.to_u64())
}

pub fn bigint_to_i32(bigint: &substreams::scalar::BigInt) -> Option<i32> {
    if bigint.lt(&BigInt::zero()) {
        log::info!("bigint_to_i32: value is negative");
        return None;
    }
    if bigint.gt(&BigInt::from(i32::MAX)) {
        log::info!("bigint_to_i32: value is greater than i32::MAX");
        return None;
    }
    Some(bigint.to_i32())
}

/// Validates if an address is a valid EVM address.
/// Returns true if the address is exactly 20 bytes and not the null address.
pub fn is_valid_evm_address(address: &[u8]) -> bool {
    address.len() == 20 && address != NULL_ADDRESS
}

use substreams_ethereum::pb::eth::v2::{block::DetailLevel, Block, Log, TransactionTrace};

pub fn logs_with_caller<'a>(block: &'a Block, trx: &'a TransactionTrace) -> Vec<(&'a Log, Option<Address>)> {
    let mut results = vec![];

    if block.detail_level() == DetailLevel::DetaillevelExtended {
        for (log, call_view) in trx.logs_with_calls() {
            results.push((log, Some(call_view.call.caller.to_vec())));
        }
    } else {
        for log in trx.receipt().logs() {
            results.push((log.log, None));
        }
    }

    results
}
