use std::collections::HashMap;

use common::Address;
use substreams::{log, scalar::BigInt, Hex};
use substreams_abis::evm::token::erc20;
use substreams_abis::evm::token::extensions::erc20capped;
use substreams_ethereum::rpc::RpcBatch;

/// Batch fetch totalSupply() for a list of contracts.
pub fn batch_total_supply<'a>(contracts: &'a [&Address], chunk_size: usize) -> HashMap<&'a Address, BigInt> {
    let mut results: HashMap<&Address, BigInt> = HashMap::with_capacity(contracts.len());

    for chunk in contracts.chunks(chunk_size) {
        let batch = chunk.iter().fold(RpcBatch::new(), |batch, contract| {
            batch.add(erc20::functions::TotalSupply {}, contract.to_vec())
        });
        let responses = batch.execute().expect("failed to execute erc20::functions::TotalSupply batch").responses;
        for (i, contract) in chunk.iter().enumerate() {
            if let Some(value) = RpcBatch::decode::<BigInt, erc20::functions::TotalSupply>(&responses[i]) {
                results.insert(contract, value);
            } else {
                substreams::log::info!(
                    "Failed to decode erc20::TotalSupply for contract={:?}",
                    Hex::encode(contract),
                );
            }
        }
    }
    log::info!(
        "\nTotalSupply={}\nRpcBatch={}\nMissing={}",
        contracts.len(),
        contracts.chunks(chunk_size).len(),
        contracts.len() - results.len()
    );
    results
}

/// Batch fetch maxSupply() for a list of contracts, with fallback to cap().
/// First tries maxSupply(), then falls back to cap() for contracts that don't support maxSupply.
pub fn batch_max_supply<'a>(contracts: &'a [&Address], chunk_size: usize) -> HashMap<&'a Address, BigInt> {
    let mut results: HashMap<&Address, BigInt> = HashMap::with_capacity(contracts.len());

    // First pass: try maxSupply()
    let mut missing: Vec<&'a &Address> = Vec::new();
    for chunk in contracts.chunks(chunk_size) {
        let batch = chunk.iter().fold(RpcBatch::new(), |batch, contract| {
            batch.add(MaxSupply {}, contract.to_vec())
        });
        let responses = batch.execute().expect("failed to execute MaxSupply batch").responses;
        for (i, contract) in chunk.iter().enumerate() {
            if let Some(value) = RpcBatch::decode::<BigInt, MaxSupply>(&responses[i]) {
                results.insert(contract, value);
            } else {
                missing.push(contract);
            }
        }
    }

    // Second pass: fallback to cap() for contracts missing maxSupply
    if !missing.is_empty() {
        for chunk in missing.chunks(chunk_size) {
            let batch = chunk.iter().fold(RpcBatch::new(), |batch, contract| {
                batch.add(erc20capped::functions::Cap {}, contract.to_vec())
            });
            let responses = batch.execute().expect("failed to execute erc20capped::functions::Cap batch").responses;
            for (i, contract) in chunk.iter().enumerate() {
                if let Some(value) = RpcBatch::decode::<BigInt, erc20capped::functions::Cap>(&responses[i]) {
                    results.insert(contract, value);
                }
            }
        }
    }

    log::info!(
        "\nMaxSupply={}\nFound={}\nMissing={}",
        contracts.len(),
        results.len(),
        contracts.len() - results.len()
    );
    results
}

/// maxSupply() function ABI definition.
/// Selector: 0xd5abeb01 = keccak256("maxSupply()")
/// Takes no parameters, returns uint256.
pub struct MaxSupply {}

impl MaxSupply {
    const METHOD_ID: [u8; 4] = [213u8, 171u8, 235u8, 1u8];

    pub fn encode(&self) -> Vec<u8> {
        // No parameters, just the 4-byte selector
        Self::METHOD_ID.to_vec()
    }

    pub fn output(data: &[u8]) -> Result<BigInt, String> {
        if data.len() < 32 {
            return Err(format!("expected at least 32 bytes, got {}", data.len()));
        }
        // Decode uint256 from the first 32 bytes
        let mut v = [0u8; 32];
        v.copy_from_slice(&data[..32]);
        Ok(substreams::scalar::BigInt::from_unsigned_bytes_be(&v))
    }

    pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
        match call.input.get(0..4) {
            Some(signature) => Self::METHOD_ID == signature,
            None => false,
        }
    }
}

impl substreams_ethereum::Function for MaxSupply {
    const NAME: &'static str = "maxSupply";
    fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
        Self::match_call(call)
    }
    fn decode(_call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
        Ok(Self {})
    }
    fn encode(&self) -> Vec<u8> {
        self.encode()
    }
}

impl substreams_ethereum::rpc::RPCDecodable<BigInt> for MaxSupply {
    fn output(data: &[u8]) -> Result<BigInt, String> {
        Self::output(data)
    }
}
