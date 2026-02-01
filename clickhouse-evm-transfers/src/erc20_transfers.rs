use common::{bytes_to_string, Encoding};
use proto::pb::erc20::transfers::v1 as pb;
use substreams::pb::substreams::Clock;
use substreams_database_change::tables::Tables;

use crate::{log_key, logs::set_template_log, set_clock, transactions::set_template_erc20_tx};

pub fn process_events(encoding: &Encoding, tables: &mut Tables, clock: &Clock, events: &pb::Events) {
    for (tx_index, tx) in events.transactions.iter().enumerate() {
        for (log_index, log) in tx.logs.iter().enumerate() {
            // Transfer
            if let Some(pb::log::Log::Transfer(transfer)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("erc20_transfers", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("from", bytes_to_string(&transfer.from, encoding));
                row.set("to", bytes_to_string(&transfer.to, encoding));
                row.set("amount", &transfer.amount);
            }

            // Deposit
            if let Some(pb::log::Log::Deposit(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("weth_deposit", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("dst", bytes_to_string(&event.dst, encoding));
                row.set("wad", &event.wad);
            }

            // Withdrawal
            if let Some(pb::log::Log::Withdrawal(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("weth_withdrawal", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("src", bytes_to_string(&event.src, encoding));
                row.set("wad", &event.wad);
            }

            // Approval
            if let Some(pb::log::Log::Approval(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("erc20_approvals", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("owner", bytes_to_string(&event.owner, encoding));
                row.set("spender", bytes_to_string(&event.spender, encoding));
                row.set("value", &event.value.to_string());
            }

            // USDC Mint
            if let Some(pb::log::Log::UsdcMint(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdc_mint", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("minter", bytes_to_string(&event.minter, encoding));
                row.set("to", bytes_to_string(&event.to, encoding));
                row.set("amount", &event.amount);
            }

            // USDC Burn
            if let Some(pb::log::Log::UsdcBurn(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdc_burn", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("burner", bytes_to_string(&event.burner, encoding));
                row.set("amount", &event.amount);
            }

            // USDT Issue
            if let Some(pb::log::Log::UsdtIssue(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdt_issue", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("amount", &event.amount);
            }

            // USDT Redeem
            if let Some(pb::log::Log::UsdtRedeem(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("usdt_redeem", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("amount", &event.amount);
            }

            // stETH TokenRebased
            if let Some(pb::log::Log::StethTokenRebased(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_token_rebased", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("report_timestamp", &event.report_timestamp);
                row.set("time_elapsed", &event.time_elapsed);
                row.set("pre_total_shares", &event.pre_total_shares);
                row.set("pre_total_ether", &event.pre_total_ether);
                row.set("post_total_shares", &event.post_total_shares);
                row.set("post_total_ether", &event.post_total_ether);
                row.set("shares_minted_as_fees", &event.shares_minted_as_fees);
            }

            // stETH SharesBurnt
            if let Some(pb::log::Log::StethSharesBurnt(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_shares_burnt", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("account", bytes_to_string(&event.account, encoding));
                row.set("pre_rebase_token_amount", &event.pre_rebase_token_amount);
                row.set("post_rebase_token_amount", &event.post_rebase_token_amount);
                row.set("shares_amount", &event.shares_amount);
            }

            // stETH TransferShares
            if let Some(pb::log::Log::StethTransferShares(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_transfer_shares", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("from", bytes_to_string(&event.from, encoding));
                row.set("to", bytes_to_string(&event.to, encoding));
                row.set("shares_value", &event.shares_value);
            }

            // stETH ExternalSharesBurnt
            if let Some(pb::log::Log::StethExternalSharesBurnt(event)) = &log.log {
                let key = log_key(clock, tx_index, log_index);
                let row = tables.create_row("steth_external_shares_burnt", key);

                set_clock(clock, row);
                set_template_log(encoding, log, log_index, row);
                set_template_erc20_tx(encoding, tx, tx_index, row);

                row.set("amount_of_shares", &event.amount_of_shares);
            }
        }
    }
}
