use common::create::{CreateCall, CreateTransaction};
use proto::pb::contracts::v1 as pb;
use substreams_ethereum::pb::eth::v2::{Block, CallType};

#[substreams::handlers::map]
pub fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();

    for transaction_trace in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(transaction_trace);
        let tx_from = transaction_trace.from.to_vec();
        let tx_to = if transaction_trace.to.is_empty() {
            None
        } else {
            Some(transaction_trace.to.to_vec())
        };

        for call in &transaction_trace.calls {
            if call.call_type() != CallType::Create {
                continue;
            }

            for code_change in &call.code_changes {
                let factory = match tx_to.as_ref() {
                    Some(address) if *address != code_change.address => Some(address.clone()),
                    _ => None,
                };
                let deployer = factory.clone().unwrap_or_else(|| tx_from.clone());

                transaction.contracts.push(pb::Contract {
                    address: code_change.address.to_vec(),
                    ordinal: code_change.ordinal,
                    from: tx_from.clone(),
                    to: tx_to.clone(),
                    deployer,
                    factory,
                    code: code_change.new_code.to_vec(),
                    code_hash: code_change.new_hash.to_vec(),
                    input: call.input.to_vec(),
                    call: Some(pb::Call::create_call(call)),
                    log: None,
                });
            }
        }

        if !transaction.contracts.is_empty() {
            events.transactions.push(transaction);
        }
    }

    Ok(events)
}
