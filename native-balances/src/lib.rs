mod calls;
mod utils;
use proto::pb::evm::balances::v1::{Balance, Events};
use std::collections::{HashMap, HashSet};
use substreams::errors::Error;
use substreams_ethereum::pb::eth::v2::Block;

use crate::{
    calls::batch_eth_balance_of,
    utils::{get_balances, is_failed_transaction, is_gas_balance_change, is_valid_balance_change},
};

#[substreams::handlers::map]
pub fn map_events(params: String, block: Block) -> Result<Events, Error> {
    let mut events = Events::default();
    let chunk_size = params.parse::<usize>().expect("Failed to parse chunk_size");
    let mut balances = HashMap::new();

    // EXTENDED
    // balance changes at block level
    for balance_change in &block.balance_changes {
        // Block Rewards as balance change
        if is_valid_balance_change(balance_change) {
            let (_, new_balance) = get_balances(balance_change);
            // events.extended_balances_by_account_from_block_rewards.push(BalanceByAccount {
            //     tx_hash: None,
            //     account: balance_change.address.to_vec(),
            //     amount: new_balance.to_string(),
            // });
            balances.insert(balance_change.address.to_vec(), new_balance);
        }
    }

    // balance changes at system call level
    for call in &block.system_calls {
        for balance_change in &call.balance_changes {
            if is_valid_balance_change(balance_change) {
                let (_, new_balance) = get_balances(balance_change);
                balances.insert(balance_change.address.to_vec(), new_balance);
            }
        }
    }

    // EXTENDED
    // iterate over all transactions including failed ones
    for trx in &block.transaction_traces {
        for call_view in trx.calls() {
            for balance_change in &call_view.call.balance_changes {
                if is_valid_balance_change(balance_change) {
                    let (_, new_balance) = get_balances(balance_change);

                    // gas balance changes
                    if is_gas_balance_change(balance_change) {
                        balances.insert(balance_change.address.to_vec(), new_balance);
                    // non-gas successful balance changes
                    } else if !is_failed_transaction(trx) {
                        balances.insert(balance_change.address.to_vec(), new_balance);
                    }
                }
            }
        }
    }

    // BASE BLOCKS (NOT EXTENDED)
    // collect all unique accounts from transactions/calls/logs
    // - trx.from
    // - trx.to
    // - log.address
    // - call.address
    // - call.caller
    // - call.address_delegates_to
    let mut accounts = HashSet::new();
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

    // NATIVE ETH BALANCE OF
    for (account, balance) in &batch_eth_balance_of(block.number, &accounts.iter().collect::<Vec<_>>(), chunk_size) {
        balances.insert(account.to_vec(), balance.clone());
    }

    // prepare final events
    for (account, balance) in balances {
        {
            events.balances.push(Balance {
                contract: None,
                account,
                balance: balance.to_string(),
            });
        }
    }
    substreams::log::info!("Emitted {} balance events", events.balances.len());
    Ok(events)
}
