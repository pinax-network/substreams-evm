use common::clickhouse::{log_key, set_clock, set_template_call, set_template_log, set_template_tx};
use common::{bytes_to_string, Encoding};
use proto::pb::sunpump::v1::{self as sunpump};
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;
use substreams_ethereum::NULL_ADDRESS;

// SunPump Processing
pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &sunpump::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            match &log.log {
                Some(sunpump::log::Log::TokenPurchased(purchase)) => {
                    process_sunpump_token_purchased(encoding, tables, clock, tx, log, tx_index, log_index, purchase);
                }
                Some(sunpump::log::Log::TokenSold(sold)) => {
                    process_sunpump_token_sold(encoding, tables, clock, tx, log, tx_index, log_index, sold);
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

fn set_factory(encoding: &Encoding, factory: &[u8], row: &mut substreams_database_change::tables::Row) {
    row.set("factory", bytes_to_string(factory, encoding));
}

fn set_native_trx(encoding: &Encoding, row: &mut substreams_database_change::tables::Row) {
    row.set("eth", bytes_to_string(&NULL_ADDRESS, encoding));
}

fn process_sunpump_token_purchased(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    purchase: &sunpump::TokenPurchased,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_token_purchased", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);
    set_factory(encoding, &log.address, row);
    set_native_trx(encoding, row);

    // Swap info - TRX -> Token purchase
    row.set("buyer", bytes_to_string(&purchase.buyer, encoding));
    row.set("trx_amount", &purchase.trx_amount);
    row.set("token", bytes_to_string(&purchase.token, encoding));
    row.set("token_amount", &purchase.token_amount);
    row.set("fee", &purchase.fee);
    row.set("token_reserve", &purchase.token_reserve);
}

fn process_sunpump_token_sold(
    encoding: &Encoding,
    tables: &mut Tables,
    clock: &Clock,
    tx: &sunpump::Transaction,
    log: &sunpump::Log,
    tx_index: usize,
    log_index: usize,
    sold: &sunpump::TokenSold,
) {
    let key = log_key(clock, tx_index, log_index);
    let row = tables.create_row("sunpump_token_sold", key);

    // Block and transaction info
    set_clock(clock, row);
    set_template_tx(encoding, tx, tx_index, row);
    set_template_log(encoding, log, log_index, row);
    set_template_call(encoding, log, row);
    set_factory(encoding, &log.address, row);
    set_native_trx(encoding, row);

    // Swap info - Token -> TRX sale
    row.set("seller", bytes_to_string(&sold.seller, encoding));
    row.set("token", bytes_to_string(&sold.token, encoding));
    row.set("token_amount", &sold.token_amount);
    row.set("trx_amount", &sold.trx_amount);
    row.set("fee", &sold.fee);
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
    set_template_call(encoding, log, row);

    // Event info
    row.set("token", bytes_to_string(&event.token, encoding));
    row.set("eth", bytes_to_string(&NULL_ADDRESS, encoding));
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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);
    set_factory(encoding, &log.address, row);

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
    set_template_call(encoding, log, row);
    set_factory(encoding, &log.address, row);

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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);

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
    set_template_call(encoding, log, row);

    // Event info
    row.set("token", bytes_to_string(&event.token, encoding));
    row.set("eth", bytes_to_string(&NULL_ADDRESS, encoding));
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::Timestamp;
    use substreams_database_change::pb::database::TableChange;

    fn sample_clock() -> Clock {
        Clock {
            id: "abcd".into(),
            number: 69_911_427,
            timestamp: Some(Timestamp {
                seconds: 1_726_000_000,
                nanos: 0,
            }),
        }
    }

    fn sample_tx() -> sunpump::Transaction {
        sunpump::Transaction {
            hash: vec![0x01; 32],
            from: vec![0x02; 20],
            to: Some(vec![0x03; 20]),
            nonce: 7,
            gas_price: "1".into(),
            gas_limit: 21_000,
            gas_used: 20_000,
            value: "0".into(),
            logs: vec![],
        }
    }

    fn sample_log() -> sunpump::Log {
        sunpump::Log {
            address: vec![0x11; 20],
            ordinal: 99,
            topics: vec![vec![0xaa; 32]],
            data: vec![0xbb],
            call: None,
            block_index: 3,
            log: None,
        }
    }

    fn table_change<'a>(changes: &'a substreams_database_change::pb::database::DatabaseChanges, table: &str) -> &'a TableChange {
        changes.table_changes.iter().find(|change| change.table == table).unwrap()
    }

    fn field<'a>(change: &'a TableChange, name: &str) -> &'a str {
        change.fields.iter().find(|field| field.name == name).unwrap().new_value.as_str()
    }

    #[test]
    fn token_purchased_rows_are_created_without_foundational_store() {
        let mut tables = Tables::new();
        let tx = sample_tx();
        let log = sample_log();
        let purchase = sunpump::TokenPurchased {
            token: vec![0x22; 20],
            buyer: vec![0x33; 20],
            trx_amount: "42".into(),
            fee: "1".into(),
            token_amount: "84".into(),
            token_reserve: "168".into(),
        };

        process_sunpump_token_purchased(&Encoding::Hex, &mut tables, &sample_clock(), &tx, &log, 0, 0, &purchase);

        let changes = tables.to_database_changes();
        let change = table_change(&changes, "sunpump_token_purchased");
        assert_eq!(field(change, "factory"), "0x1111111111111111111111111111111111111111");
        assert_eq!(field(change, "token"), "0x2222222222222222222222222222222222222222");
        assert_eq!(field(change, "eth"), "0x0000000000000000000000000000000000000000");
    }

    #[test]
    fn purchase_fee_rows_are_created_without_foundational_store() {
        let mut tables = Tables::new();
        let tx = sample_tx();
        let log = sample_log();
        let event = sunpump::PurchaseFeeSet {
            old_fee: "5".into(),
            new_fee: "7".into(),
        };

        process_sunpump_purchase_fee_set(&Encoding::Hex, &mut tables, &sample_clock(), &tx, &log, 0, 0, &event);

        let changes = tables.to_database_changes();
        let change = table_change(&changes, "sunpump_purchase_fee_set");
        assert_eq!(field(change, "factory"), "0x1111111111111111111111111111111111111111");
        assert_eq!(field(change, "old_fee"), "5");
        assert_eq!(field(change, "new_fee"), "7");
    }
}
