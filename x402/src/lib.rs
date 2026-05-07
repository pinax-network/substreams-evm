use proto::pb::evm::x402::v1 as pb;
use substreams::errors::Error;
use substreams::hex;
use substreams_abis::standard::erc20::events as erc20_events;
use substreams_abis::tokens::erc20::usdc::fiattoken_v2_2::{events as usdc_events, functions as usdc_functions};
use substreams_ethereum::pb::eth::v2::{Block, Call, Log, TransactionTrace};
use substreams_ethereum::Event;

const DEFAULT_X402_PERMIT2_PROXY: [u8; 20] = hex!("402085c248eea27d92e8b30b2c58ed07f9e20001");
const TRANSFER_TOPIC: [u8; 32] = hex!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
const SETTLED_TOPIC: [u8; 32] = hex!("97088ec3606cfe8cc112180570d03fcde05f9b8e1bfef8e27784eaf5dd5691b6");
const SETTLED_WITH_PERMIT_TOPIC: [u8; 32] = hex!("de5b89d10fc800c459329c382fabfcad0be0ed7e5328e01fae04e507b09ef5d8");
const TRANSFER_WITH_AUTHORIZATION_BYTES_SELECTOR: [u8; 4] = hex!("cf092995");
const TRANSFER_WITH_AUTHORIZATION_VRS_SELECTOR: [u8; 4] = hex!("e3ee160e");

struct DecodedAuthorization {
    from: Vec<u8>,
    to: Vec<u8>,
    value: String,
    valid_after: String,
    valid_before: String,
    nonce: Vec<u8>,
    call_index: u32,
}

struct TransferLog {
    from: Vec<u8>,
    to: Vec<u8>,
    amount: String,
}

#[substreams::handlers::map]
pub fn map_events(block: Block) -> Result<pb::Events, Error> {
    let mut events = pb::Events::default();
    let mut payment_count = 0;

    for (tx_index, trx) in block.transactions().enumerate() {
        let mut transaction = create_transaction(tx_index as u32, trx);
        collect_eip3009_payments(trx, &mut transaction);
        collect_permit2_payments(trx, &mut transaction);

        if !transaction.logs.is_empty() {
            payment_count += transaction.logs.len();
            events.transactions.push(transaction);
        }
    }

    substreams::log::info!("x402 payments: {}", payment_count);
    Ok(events)
}

fn collect_eip3009_payments(trx: &TransactionTrace, transaction: &mut pb::Transaction) {
    let decoded_authorizations = decoded_authorizations_by_call(trx);
    let logs_with_calls: Vec<(&Log, Option<&Call>)> = if trx.calls.is_empty() {
        trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
    } else {
        trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
    };

    for (log, call) in logs_with_calls.iter() {
        let Some(authorization_used) = usdc_events::AuthorizationUsed::match_and_decode(log) else {
            continue;
        };

        let decoded = call
            .as_ref()
            .and_then(|call| decode_transfer_with_authorization(call))
            .or_else(|| find_decoded_authorization(&decoded_authorizations, &log.address, &authorization_used.authorizer, &authorization_used.nonce));

        let transfer = decoded
            .as_ref()
            .and_then(|auth| find_matching_transfer(trx, &log.address, &auth.from, &auth.to, &auth.value))
            .or_else(|| find_matching_transfer_by_authorizer(trx, &log.address, &authorization_used.authorizer));

        let Some(transfer) = transfer else {
            continue;
        };

        let payer = decoded
            .as_ref()
            .map(|auth| auth.from.clone())
            .unwrap_or_else(|| authorization_used.authorizer.to_vec());
        let recipient = decoded.as_ref().map(|auth| auth.to.clone()).unwrap_or_else(|| transfer.to.clone());
        let amount = decoded.as_ref().map(|auth| auth.value.clone()).unwrap_or_else(|| transfer.amount.clone());
        let nonce = decoded
            .as_ref()
            .map(|auth| auth.nonce.clone())
            .unwrap_or_else(|| authorization_used.nonce.to_vec());

        let call = decoded
            .as_ref()
            .and_then(|auth| trx.calls.iter().find(|call| call.index == auth.call_index))
            .or(*call);

        transaction.logs.push(create_payment_log(
            log,
            call,
            pb::Payment {
                asset: log.address.to_vec(),
                payer,
                recipient,
                facilitator: trx.from.to_vec(),
                amount,
                nonce,
                transfer_method: pb::TransferMethod::Eip3009 as i32,
                settlement_source: pb::SettlementSource::AuthorizationUsed as i32,
                scheme: "exact".to_string(),
                valid_after: decoded.as_ref().map(|auth| auth.valid_after.clone()),
                valid_before: decoded.as_ref().map(|auth| auth.valid_before.clone()),
                facilitator_allowlist_matched: false,
                confidence: "heuristic".to_string(),
            },
        ));
    }
}

fn collect_permit2_payments(trx: &TransactionTrace, transaction: &mut pb::Transaction) {
    let logs_with_calls: Vec<(&Log, Option<&Call>)> = if trx.calls.is_empty() {
        trx.receipt().logs().map(|log_view| (log_view.log, None)).collect()
    } else {
        trx.logs_with_calls().map(|(log, call_view)| (log, Some(call_view.call))).collect()
    };

    for (log, call) in logs_with_calls {
        if log.address.as_slice() != DEFAULT_X402_PERMIT2_PROXY || log.topics.is_empty() {
            continue;
        }

        let source = if log.topics[0] == SETTLED_TOPIC {
            pb::SettlementSource::Permit2Settled
        } else if log.topics[0] == SETTLED_WITH_PERMIT_TOPIC {
            pb::SettlementSource::Permit2SettledWithPermit
        } else {
            continue;
        };

        let Some((asset, transfer)) = find_last_erc20_transfer_before(trx, log.ordinal) else {
            continue;
        };

        transaction.logs.push(create_payment_log(
            log,
            call,
            pb::Payment {
                asset,
                payer: transfer.from,
                recipient: transfer.to,
                facilitator: trx.from.to_vec(),
                amount: transfer.amount,
                nonce: vec![],
                transfer_method: pb::TransferMethod::Permit2 as i32,
                settlement_source: source as i32,
                scheme: "exact".to_string(),
                valid_after: None,
                valid_before: None,
                facilitator_allowlist_matched: false,
                confidence: "high".to_string(),
            },
        ));
    }
}

fn create_transaction(tx_index: u32, trx: &TransactionTrace) -> pb::Transaction {
    let gas_price = trx.clone().gas_price.unwrap_or_default().with_decimal(0).to_string();
    let value = trx.clone().value.unwrap_or_default().with_decimal(0).to_string();

    pb::Transaction {
        hash: trx.hash.to_vec(),
        from: trx.from.to_vec(),
        to: if trx.to.is_empty() { None } else { Some(trx.to.to_vec()) },
        index: tx_index,
        nonce: trx.nonce,
        gas_price,
        gas_limit: trx.gas_limit,
        gas_used: trx.receipt().receipt.cumulative_gas_used,
        value,
        logs: vec![],
    }
}

fn create_payment_log(log: &Log, call: Option<&Call>, payment: pb::Payment) -> pb::Log {
    pb::Log {
        address: log.address.to_vec(),
        ordinal: log.ordinal,
        topics: log.topics.iter().map(|topic| topic.to_vec()).collect(),
        data: log.data.to_vec(),
        call: call.map(create_call),
        block_index: log.block_index,
        log: Some(pb::log::Log::Payment(payment)),
    }
}

fn create_call(call: &Call) -> pb::Call {
    pb::Call {
        index: call.index,
        begin_ordinal: call.begin_ordinal,
        end_ordinal: call.end_ordinal,
        caller: call.caller.to_vec(),
        address: call.address.to_vec(),
        value: call.value.clone().unwrap_or_default().with_decimal(0).to_string(),
        gas_consumed: call.gas_consumed,
        gas_limit: call.gas_limit,
        depth: call.depth,
        parent_index: call.parent_index,
        call_type: call.call_type,
    }
}

fn decoded_authorizations_by_call(trx: &TransactionTrace) -> Vec<(Vec<u8>, DecodedAuthorization)> {
    trx.calls
        .iter()
        .filter_map(|call| decode_transfer_with_authorization(call).map(|decoded| (call.address.to_vec(), decoded)))
        .collect()
}

fn decode_transfer_with_authorization(call: &Call) -> Option<DecodedAuthorization> {
    if is_transfer_with_authorization_vrs_call(call) {
        let decoded = usdc_functions::TransferWithAuthorization2::decode(call).ok()?;
        return Some(DecodedAuthorization {
            from: decoded.from,
            to: decoded.to,
            value: decoded.value.to_string(),
            valid_after: decoded.valid_after.to_string(),
            valid_before: decoded.valid_before.to_string(),
            nonce: decoded.nonce.to_vec(),
            call_index: call.index,
        });
    }

    if is_transfer_with_authorization_bytes_call(call) {
        let decoded = usdc_functions::TransferWithAuthorization1::decode(call).ok()?;
        return Some(DecodedAuthorization {
            from: decoded.from,
            to: decoded.to,
            value: decoded.value.to_string(),
            valid_after: decoded.valid_after.to_string(),
            valid_before: decoded.valid_before.to_string(),
            nonce: decoded.nonce.to_vec(),
            call_index: call.index,
        });
    }

    None
}

fn find_decoded_authorization(
    decoded_authorizations: &[(Vec<u8>, DecodedAuthorization)],
    asset: &[u8],
    authorizer: &[u8],
    nonce: &[u8],
) -> Option<DecodedAuthorization> {
    decoded_authorizations
        .iter()
        .find(|(call_address, auth)| call_address.as_slice() == asset && auth.from.as_slice() == authorizer && auth.nonce.as_slice() == nonce)
        .map(|(_, auth)| DecodedAuthorization {
            from: auth.from.clone(),
            to: auth.to.clone(),
            value: auth.value.clone(),
            valid_after: auth.valid_after.clone(),
            valid_before: auth.valid_before.clone(),
            nonce: auth.nonce.clone(),
            call_index: auth.call_index,
        })
}

fn find_matching_transfer(trx: &TransactionTrace, asset: &[u8], from: &[u8], to: &[u8], amount: &str) -> Option<TransferLog> {
    trx.receipt().logs().find_map(|log_view| {
        let log = log_view.log;
        if log.address.as_slice() != asset {
            return None;
        }
        let transfer = decode_erc20_transfer(log)?;
        if transfer.from.as_slice() == from && transfer.to.as_slice() == to && transfer.value.to_string() == amount {
            Some(TransferLog {
                from: transfer.from.to_vec(),
                to: transfer.to.to_vec(),
                amount: transfer.value.to_string(),
            })
        } else {
            None
        }
    })
}

fn find_matching_transfer_by_authorizer(trx: &TransactionTrace, asset: &[u8], authorizer: &[u8]) -> Option<TransferLog> {
    trx.receipt().logs().find_map(|log_view| {
        let log = log_view.log;
        if log.address.as_slice() != asset {
            return None;
        }
        let transfer = decode_erc20_transfer(log)?;
        if transfer.from.as_slice() == authorizer {
            Some(TransferLog {
                from: transfer.from.to_vec(),
                to: transfer.to.to_vec(),
                amount: transfer.value.to_string(),
            })
        } else {
            None
        }
    })
}

fn find_last_erc20_transfer_before(trx: &TransactionTrace, ordinal: u64) -> Option<(Vec<u8>, TransferLog)> {
    trx.receipt()
        .logs()
        .filter_map(|log_view| {
            let log = log_view.log;
            if log.ordinal > ordinal {
                return None;
            }
            let transfer = decode_erc20_transfer(log)?;
            Some((
                log.address.to_vec(),
                TransferLog {
                    from: transfer.from.to_vec(),
                    to: transfer.to.to_vec(),
                    amount: transfer.value.to_string(),
                },
            ))
        })
        .last()
}

fn decode_erc20_transfer(log: &Log) -> Option<erc20_events::Transfer> {
    if log.topics.len() < 3 || log.topics[0].as_slice() != TRANSFER_TOPIC || log.data.len() < 32 {
        return None;
    }
    erc20_events::Transfer::match_and_decode(log)
}

fn is_transfer_with_authorization_vrs_call(call: &Call) -> bool {
    call.input.starts_with(&TRANSFER_WITH_AUTHORIZATION_VRS_SELECTOR) && call.input.len() >= 4 + (9 * 32)
}

fn is_transfer_with_authorization_bytes_call(call: &Call) -> bool {
    if !call.input.starts_with(&TRANSFER_WITH_AUTHORIZATION_BYTES_SELECTOR) || call.input.len() < 4 + (7 * 32) {
        return false;
    }

    let args = &call.input[4..];
    let Some(offset) = read_usize_word(args, 6 * 32) else {
        return false;
    };
    if offset < 7 * 32 || offset % 32 != 0 {
        return false;
    }
    let Some(length) = read_usize_word(args, offset) else {
        return false;
    };
    let Some(end) = offset.checked_add(32).and_then(|start| start.checked_add(round_up_to_word(length))) else {
        return false;
    };
    end <= args.len()
}

fn read_usize_word(bytes: &[u8], start: usize) -> Option<usize> {
    let end = start.checked_add(32)?;
    let word = bytes.get(start..end)?;
    if word[..24].iter().any(|byte| *byte != 0) {
        return None;
    }

    let mut raw = [0u8; 8];
    raw.copy_from_slice(&word[24..32]);
    Some(u64::from_be_bytes(raw) as usize)
}

fn round_up_to_word(value: usize) -> usize {
    value.checked_add(31).map(|value| value / 32 * 32).unwrap_or(usize::MAX)
}
