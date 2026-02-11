mod calls;
mod utils;
use proto::pb::evm::balances::v1::{Balance, BalanceChange, BalanceChanges, Events};
use std::collections::HashSet;
use substreams::errors::Error;
use substreams_ethereum::pb::eth::v2::Block;

use crate::{
    calls::batch_eth_balance_of,
    utils::{is_failed_transaction, is_gas_balance_change, is_valid_balance_change},
};

#[substreams::handlers::map]
pub fn map_balance_changes(block: Block) -> Result<BalanceChanges, Error> {
    let mut accounts = HashSet::new();

    // EXTENDED
    // balance changes at block level
    for balance_change in &block.balance_changes {
        if is_valid_balance_change(balance_change) {
            accounts.insert(balance_change.address.to_vec());
        }
    }

    // balance changes at system call level
    for call in &block.system_calls {
        for balance_change in &call.balance_changes {
            if is_valid_balance_change(balance_change) {
                accounts.insert(balance_change.address.to_vec());
            }
        }
    }

    // EXTENDED
    // iterate over all transactions including failed ones
    for trx in &block.transaction_traces {
        for call_view in trx.calls() {
            for balance_change in &call_view.call.balance_changes {
                if is_valid_balance_change(balance_change) {
                    // gas balance changes
                    if is_gas_balance_change(balance_change) {
                        accounts.insert(balance_change.address.to_vec());
                    // non-gas successful balance changes
                    } else if !is_failed_transaction(trx) {
                        accounts.insert(balance_change.address.to_vec());
                    }
                }
            }
        }
    }

    // BASE BLOCKS (NOT EXTENDED)
    // collect all unique accounts from transactions/calls/logs
    // - coinbase (block miner)
    // - trx.from
    // - trx.to
    // - log.address
    // - call.address
    // - call.caller
    // - call.address_delegates_to

    // include coinbase (miner) address from block header
    if let Some(header) = &block.header {
        if !header.coinbase.is_empty() {
            accounts.insert(header.coinbase.to_vec());
        }
    }

    for trx in &block.transaction_traces {
        accounts.insert(trx.from.to_vec());
        accounts.insert(trx.to.to_vec());

        for call_view in trx.calls() {
            let call = call_view.call;
            accounts.insert(call.address.to_vec());
            accounts.insert(call.caller.to_vec());
            if let Some(address_delegates_to) = &call.address_delegates_to {
                accounts.insert(address_delegates_to.to_vec());
            }
            for log in call.logs.iter() {
                accounts.insert(log.address.to_vec());
            }
        }
    }

    let mut balance_changes = BalanceChanges::default();
    for address in accounts {
        balance_changes.balance_changes.push(BalanceChange { contract: None, address });
    }
    Ok(balance_changes)
}

#[substreams::handlers::map]
pub fn map_events(params: String, block: Block, balance_changes: BalanceChanges) -> Result<Events, Error> {
    let mut events = Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");

    let accounts: Vec<Vec<u8>> = balance_changes.balance_changes.into_iter().map(|bc| bc.address).collect();

    // NATIVE ETH BALANCE OF
    for (address, amount) in &batch_eth_balance_of(block.number, &accounts.iter().collect::<Vec<_>>(), chunk_size) {
        events.balances.push(Balance {
            contract: None,
            address: address.to_vec(),
            amount: amount.to_string(),
        });
    }

    substreams::log::info!("Emitted {} balance events", events.balances.len());
    Ok(events)
}
