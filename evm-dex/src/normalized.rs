use common::clickhouse::{bytes_to_hex, log_key, set_clock, set_template_call, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::dex::swaps::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(pb::log::Log::Swap(swap)) => process_swap(encoding, tables, clock, tx, log, tx_index, log_index, swap),
                Some(pb::log::Log::SwapFee(swap_fee)) => {
                    process_swap_fee(encoding, tables, clock, tx, log, tx_index, log_index, swap_fee)
                }
                Some(pb::log::Log::Initialize(initialize)) => {
                    process_initialize(encoding, tables, clock, tx, log, tx_index, log_index, initialize)
                }
                None => {}
            }
        }
    }
}

fn process_swap(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &pb::Transaction,
    log: &pb::Log,
    tx_index: usize,
    log_index: usize,
    swap: &pb::Swap,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("swaps", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_call(encoding, log, row);

    row.set("log_index", log_index as u32)
        .set("log_block_index", log.block_index)
        .set("log_address", bytes_to_string(&log.address, encoding))
        .set("log_ordinal", log.ordinal)
        .set(
            "log_topic0",
            log.topics.first().map(|topic| bytes_to_hex(topic)).unwrap_or_default(),
        )
        .set("protocol", protocol_name(swap.protocol))
        .set("factory", bytes_to_string(&swap.factory, encoding))
        .set("pool", bytes_to_string(&swap.pool, encoding))
        .set("user", bytes_to_string(&swap.user, encoding))
        .set("input_contract", bytes_to_string(&swap.input_token, encoding))
        .set("input_amount", &swap.input_amount)
        .set("output_contract", bytes_to_string(&swap.output_token, encoding))
        .set("output_amount", &swap.output_amount);
}

fn process_swap_fee(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &pb::Transaction,
    _log: &pb::Log,
    tx_index: usize,
    log_index: usize,
    swap_fee: &pb::SwapFee,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("state_pools_fees", key);

    set_clock(clock, row);
    row.set("tx_hash", bytes_to_hex(&tx.hash))
        .set("factory", bytes_to_string(&swap_fee.factory, encoding))
        .set("pool", bytes_to_string(&swap_fee.pool, encoding))
        .set("protocol", protocol_name(swap_fee.protocol))
        .set("fee", swap_fee.fee);
}

fn process_initialize(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &pb::Transaction,
    _log: &pb::Log,
    tx_index: usize,
    log_index: usize,
    initialize: &pb::Initialize,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("state_pools_initialize", key);

    set_clock(clock, row);
    row.set("tx_hash", bytes_to_hex(&tx.hash))
        .set("factory", bytes_to_string(&initialize.factory, encoding))
        .set("pool", bytes_to_string(&initialize.pool, encoding))
        .set("protocol", protocol_name(initialize.protocol));
}

fn protocol_name(protocol: i32) -> &'static str {
    match pb::Protocol::try_from(protocol).unwrap_or(pb::Protocol::Unspecified) {
        pb::Protocol::Aerodrome => "aerodrome",
        pb::Protocol::Balancer => "balancer",
        pb::Protocol::Bancor => "bancor",
        pb::Protocol::Cow => "cow",
        pb::Protocol::Curvefi => "curvefi",
        pb::Protocol::Dodo => "dodo",
        pb::Protocol::KyberElastic => "kyber_elastic",
        pb::Protocol::Sunpump => "sunpump",
        pb::Protocol::Traderjoe => "traderjoe",
        pb::Protocol::UniswapV1 => "uniswap_v1",
        pb::Protocol::UniswapV2 => "uniswap_v2",
        pb::Protocol::UniswapV3 => "uniswap_v3",
        pb::Protocol::UniswapV4 => "uniswap_v4",
        pb::Protocol::Woofi => "woofi",
        pb::Protocol::Unspecified => "unspecified",
    }
}
