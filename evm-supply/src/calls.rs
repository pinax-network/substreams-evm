use std::collections::HashMap;

use common::Address;
use substreams::{log, scalar::BigInt, Hex};
use substreams_abis::evm::token::erc20;
use substreams_ethereum::rpc::RpcBatch;

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
                substreams::log::info!("Failed to decode erc20::TotalSupply for contract={:?}", Hex::encode(contract));
            }
        }
    }
    log::info!(
        "\nSupply={}\nRpcBatch={}\nMissing={}",
        contracts.len(),
        contracts.chunks(chunk_size).len(),
        contracts.len() - results.len()
    );
    results
}
