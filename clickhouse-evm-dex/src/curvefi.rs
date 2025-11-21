use common::{bytes_to_string, Encoding};
use proto::pb::curvefi::v1::{self as curvefi};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{logs::set_template_log, set_clock, transactions::set_template_tx};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &curvefi::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(curvefi::log::Log::TokenExchange(event)) => {
                    process_token_exchange(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::AddLiquidity(event)) => {
                    process_add_liquidity(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidity(event)) => {
                    process_remove_liquidity(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidityOne(event)) => {
                    process_remove_liquidity_one(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::RemoveLiquidityImbalance(event)) => {
                    process_remove_liquidity_imbalance(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(curvefi::log::Log::Init(event)) => {
                    process_init(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {}
            }
        }
    }
}

fn log_key(clock: &Clock, tx_index: usize, log_index: usize) -> [(&'static str, String); 5] {
    [
        ("block_num", clock.number.to_string()),
        ("block_hash", format!("0x{}", clock.id)),
        ("tx_index", tx_index.to_string()),
        ("log_index", log_index.to_string()),
        ("timestamp", clock.timestamp.as_ref().map(|t| t.seconds.to_string()).unwrap_or_default()),
    ]
}

fn process_token_exchange(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::TokenExchange,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_token_exchange", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("buyer", bytes_to_string(&event.buyer, encoding));
    row.set("sold_id", &event.sold_id);
    row.set("tokens_sold", &event.tokens_sold);
    row.set("bought_id", &event.bought_id);
    row.set("tokens_bought", &event.tokens_bought);
}

fn process_add_liquidity(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::AddLiquidity,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_add_liquidity", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amounts", event.token_amounts.join(","));
    row.set("fees", event.fees.join(","));
    row.set("invariant", &event.invariant);
    row.set("token_supply", &event.token_supply);
}

fn process_remove_liquidity(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidity,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_remove_liquidity", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amounts", event.token_amounts.join(","));
    row.set("fees", event.fees.join(","));
    row.set("token_supply", &event.token_supply);
}

fn process_remove_liquidity_one(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityOne,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_remove_liquidity_one", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amount", &event.token_amount);
    row.set("coin_amount", &event.coin_amount);
}

fn process_remove_liquidity_imbalance(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::RemoveLiquidityImbalance,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_remove_liquidity_imbalance", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("provider", bytes_to_string(&event.provider, encoding));
    row.set("token_amounts", event.token_amounts.join(","));
    row.set("fees", event.fees.join(","));
    row.set("invariant", &event.invariant);
    row.set("token_supply", &event.token_supply);
}

fn process_init(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &curvefi::Transaction,
    log: &curvefi::Log,
    tx_index: usize,
    log_index: usize,
    event: &curvefi::Init,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("curvefi_init", key);

    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    row.set("owner", bytes_to_string(&event.owner, encoding));
    // Note: Coins are stored as comma-separated addresses for consistency with other
    // array fields in this codebase (e.g., token_amounts, fees in AddLiquidity).
    // CurveFi pools typically have 2-4 coins, so this approach is acceptable.
    row.set("coins", event.coins.iter().map(|c| bytes_to_string(c, encoding)).collect::<Vec<_>>().join(","));
    row.set("pool_token", bytes_to_string(&event.pool_token, encoding));
    row.set("a", &event.a);
    row.set("fee", &event.fee);
    row.set("admin_fee", &event.admin_fee);
}
