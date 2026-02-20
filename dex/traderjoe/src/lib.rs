mod store;
use common::create::{CreateLog, CreateTransaction};
use proto::pb::traderjoe::v1 as pb;
use substreams::scalar::BigInt;
use substreams_abis::dex::traderjoe;
use substreams_ethereum::pb::eth::v2::Block;
use substreams_ethereum::Event;

/// Decode a packed bytes32 into two uint128 values (upper = x, lower = y).
/// Trader Joe LB uses this encoding: `uint128(x) << 128 | uint128(y)`
fn decode_packed_uint128(bytes: &[u8; 32]) -> (String, String) {
    let x = BigInt::from_unsigned_bytes_be(&bytes[..16]);
    let y = BigInt::from_unsigned_bytes_be(&bytes[16..]);
    (x.to_string(), y.to_string())
}

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<pb::Events, substreams::errors::Error> {
    let mut events = pb::Events::default();
    let mut total_swaps = 0;
    let mut total_deposits = 0;
    let mut total_withdrawals = 0;
    let mut total_composition_fees = 0;
    let mut total_lb_pair_created = 0;

    for trx in block.transactions() {
        let mut transaction = pb::Transaction::create_transaction(trx);

        for log_view in trx.receipt().logs() {
            let log = log_view.log;

            // Swap event (LBPair)
            if let Some(event) = traderjoe::lbpair::events::Swap::match_and_decode(log) {
                total_swaps += 1;
                let (amount_in_x, amount_in_y) = decode_packed_uint128(&event.amounts_in);
                let (amount_out_x, amount_out_y) = decode_packed_uint128(&event.amounts_out);
                let (total_fees_x, total_fees_y) = decode_packed_uint128(&event.total_fees);
                let (protocol_fees_x, protocol_fees_y) = decode_packed_uint128(&event.protocol_fees);
                let event = pb::log::Log::Swap(pb::Swap {
                    sender: event.sender.to_vec(),
                    to: event.to.to_vec(),
                    id: event.id.to_u64() as u32,
                    amount_in_x,
                    amount_in_y,
                    amount_out_x,
                    amount_out_y,
                    volatility_accumulator: event.volatility_accumulator.to_u64() as u32,
                    total_fees_x,
                    total_fees_y,
                    protocol_fees_x,
                    protocol_fees_y,
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // DepositedToBins event (LBPair)
            if let Some(event) = traderjoe::lbpair::events::DepositedToBins::match_and_decode(log) {
                total_deposits += 1;
                let event = pb::log::Log::DepositedToBins(pb::DepositedToBins {
                    sender: event.sender.to_vec(),
                    to: event.to.to_vec(),
                    ids: event.ids.iter().map(|id| id.to_u64()).collect(),
                    amounts: event.amounts.iter().map(|a| a.to_vec()).collect(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // WithdrawnFromBins event (LBPair)
            if let Some(event) = traderjoe::lbpair::events::WithdrawnFromBins::match_and_decode(log) {
                total_withdrawals += 1;
                let event = pb::log::Log::WithdrawnFromBins(pb::WithdrawnFromBins {
                    sender: event.sender.to_vec(),
                    to: event.to.to_vec(),
                    ids: event.ids.iter().map(|id| id.to_u64()).collect(),
                    amounts: event.amounts.iter().map(|a| a.to_vec()).collect(),
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // CompositionFees event (LBPair)
            if let Some(event) = traderjoe::lbpair::events::CompositionFees::match_and_decode(log) {
                total_composition_fees += 1;
                let (total_fees_x, total_fees_y) = decode_packed_uint128(&event.total_fees);
                let (protocol_fees_x, protocol_fees_y) = decode_packed_uint128(&event.protocol_fees);
                let event = pb::log::Log::CompositionFees(pb::CompositionFees {
                    sender: event.sender.to_vec(),
                    id: event.id.to_u64() as u32,
                    total_fees_x,
                    total_fees_y,
                    protocol_fees_x,
                    protocol_fees_y,
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }

            // LbPairCreated event (LBFactory)
            if let Some(event) = traderjoe::lbfactory::events::LbPairCreated::match_and_decode(log) {
                total_lb_pair_created += 1;
                let event = pb::log::Log::LbPairCreated(pb::LbPairCreated {
                    token_x: event.token_x.to_vec(),
                    token_y: event.token_y.to_vec(),
                    bin_step: event.bin_step.to_u64() as u32,
                    lb_pair: event.lb_pair.to_vec(),
                    pid: event.pid.to_u64() as u32,
                });
                transaction.logs.push(pb::Log::create_log(log, event));
            }
        }

        if !transaction.logs.is_empty() {
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("Total Transactions: {}", block.transaction_traces.len());
    substreams::log::info!("Total Events: {}", events.transactions.len());
    substreams::log::info!("Total Swap events: {}", total_swaps);
    substreams::log::info!("Total DepositedToBins events: {}", total_deposits);
    substreams::log::info!("Total WithdrawnFromBins events: {}", total_withdrawals);
    substreams::log::info!("Total CompositionFees events: {}", total_composition_fees);
    substreams::log::info!("Total LbPairCreated events: {}", total_lb_pair_created);
    Ok(events)
}
