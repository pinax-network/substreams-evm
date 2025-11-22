use common::{bytes_to_string, Encoding};
use proto::pb::sunpump::v1::{self as sunpump, TokenCreate};
use substreams::{pb::substreams::Clock, store::StoreGetProto};
use substreams_database_change::tables::Tables;

use crate::{
    logs::{log_key, set_template_log},
    set_clock,
    store::get_store_by_address,
    transactions::set_template_tx,
};

// SunPump Processing
pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &sunpump::Events, store: &StoreGetProto<TokenCreate>) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(sunpump::log::Log::TokenPurchased(purchase)) => {
                    process_sunpump_token_purchased(encoding, store, tables, clock, tx, log, tx_index, log_index, purchase);
                }
                Some(sunpump::log::Log::TokenSold(sold)) => {
                    process_sunpump_token_sold(encoding, store, tables, clock, tx, log, tx_index, log_index, sold);
                }
                Some(sunpump::log::Log::LaunchPending(event)) => {
                    process_sunpump_launch_pending(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::LauncherChanged(event)) => {
                    process_sunpump_launcher_changed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::MinTxFeeSet(event)) => {
                    process_sunpump_min_tx_fee_set(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::MintFeeSet(event)) => {
                    process_sunpump_mint_fee_set(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::OperatorChanged(event)) => {
                    process_sunpump_operator_changed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::OwnerChanged(event)) => {
                    process_sunpump_owner_changed(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::PendingOwnerSet(event)) => {
                    process_sunpump_pending_owner_set(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::PurchaseFeeSet(event)) => {
                    process_sunpump_purchase_fee_set(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::SaleFeeSet(event)) => {
                    process_sunpump_sale_fee_set(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::TokenCreate(event)) => {
                    process_sunpump_token_create(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::TokenCreateLegacy(event)) => {
                    process_sunpump_token_create_legacy(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                Some(sunpump::log::Log::TokenLaunched(event)) => {
                    process_sunpump_token_launched(encoding, tables, clock, tx, log, tx_index, log_index, event);
                }
                _ => {} // Ignore other event types
            }
        }
    }
}

pub fn set_pool(encoding: &Encoding, value: TokenCreate, row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(&value.factory, encoding));
    row.set("creator", bytes_to_string(&value.creator, encoding));
    row.set("token_index", &value.token_index);
}

fn process_sunpump_token_purchased(
    encoding: &Encoding,
    store: &StoreGetProto<TokenCreate>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    purchase: &sunpump::TokenPurchased,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("sunpump_token_purchased", key);

        // Block and transaction info
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        // Swap info - TRX -> Token purchase
        row.set("buyer", bytes_to_string(&purchase.buyer, encoding));
        row.set("trx_amount", &purchase.trx_amount);
        row.set("token", bytes_to_string(&purchase.token, encoding));
        row.set("token_amount", &purchase.token_amount);
        row.set("fee", &purchase.fee);
        row.set("token_reserve", &purchase.token_reserve);
    }
}

fn process_sunpump_token_sold(
    encoding: &Encoding,
    store: &StoreGetProto<TokenCreate>,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    sold: &sunpump::TokenSold,
) {
    if let Some(pool) = get_store_by_address(store, &log.address) {
        let key = log_key(clock, tx_index, log_index);
        let row = tables.create_row("sunpump_token_sold", key);

        // Block and transaction info
        set_clock(clock, row);
        set_template_tx(encoding, tx, tx_index, row);
        set_template_log(encoding, log, log_index, row);
        set_pool(encoding, pool, row);

        // Swap info - Token -> TRX sale
        row.set("seller", bytes_to_string(&sold.seller, encoding));
        row.set("token", bytes_to_string(&sold.token, encoding));
        row.set("token_amount", &sold.token_amount);
        row.set("trx_amount", &sold.trx_amount);
        row.set("fee", &sold.fee);
    }
}

fn process_sunpump_launch_pending(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::LaunchPending,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_launch_pending", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("token", bytes_to_string(&event.token, encoding));
}

fn process_sunpump_launcher_changed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::LauncherChanged,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_launcher_changed", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_launcher", bytes_to_string(&event.old_launcher, encoding));
    row.set("new_launcher", bytes_to_string(&event.new_launcher, encoding));
}

fn process_sunpump_min_tx_fee_set(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::MinTxFeeSet,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_min_tx_fee_set", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_fee", &event.old_fee);
    row.set("new_fee", &event.new_fee);
}

fn process_sunpump_mint_fee_set(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::MintFeeSet,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_mint_fee_set", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_fee", &event.old_fee);
    row.set("new_fee", &event.new_fee);
}

fn process_sunpump_operator_changed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::OperatorChanged,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_operator_changed", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_operator", bytes_to_string(&event.old_operator, encoding));
    row.set("new_operator", bytes_to_string(&event.new_operator, encoding));
}

fn process_sunpump_owner_changed(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::OwnerChanged,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_owner_changed", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_owner", bytes_to_string(&event.old_owner, encoding));
    row.set("new_owner", bytes_to_string(&event.new_owner, encoding));
}

fn process_sunpump_pending_owner_set(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::PendingOwnerSet,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_pending_owner_set", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_pending_owner", bytes_to_string(&event.old_pending_owner, encoding));
    row.set("new_pending_owner", bytes_to_string(&event.new_pending_owner, encoding));
}

fn process_sunpump_purchase_fee_set(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::PurchaseFeeSet,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_purchase_fee_set", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_fee", &event.old_fee);
    row.set("new_fee", &event.new_fee);
}

fn process_sunpump_sale_fee_set(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::SaleFeeSet,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_sale_fee_set", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("old_fee", &event.old_fee);
    row.set("new_fee", &event.new_fee);
}

fn process_sunpump_token_create(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::TokenCreate,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_token_create", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("token_address", bytes_to_string(&event.token_address, encoding));
    row.set("token_index", &event.token_index);
    row.set("creator", bytes_to_string(&event.creator, encoding));
}

fn process_sunpump_token_create_legacy(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::TokenCreateLegacy,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_token_create_legacy", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("token_address", bytes_to_string(&event.token_address, encoding));
    row.set("creator", bytes_to_string(&event.creator, encoding));
    row.set("nft_max_supply", event.nft_max_supply);
    row.set("nft_threshold", event.nft_threshold);
    row.set("name", &event.name);
    row.set("symbol", &event.symbol);
}

fn process_sunpump_token_launched(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    event: &sunpump::TokenLaunched,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_token_launched", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);

    // Event info
    row.set("token", bytes_to_string(&event.token, encoding));
}
